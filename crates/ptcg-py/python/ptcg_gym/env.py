"""PTCG Gymnasium Environment.

Single-agent RL environment wrapping the PTCG Rust engine.
The agent plays as Player 0; the opponent is controlled by a bot.
"""

import random as _random
import numpy as np

try:
    import gymnasium as gym
    from gymnasium import spaces
    GYM_AVAILABLE = True
except ImportError:
    GYM_AVAILABLE = False

from .opponent import RandomOpponent, HeuristicOpponent


class PTCGEnv(gym.Env if GYM_AVAILABLE else object):
    """Gymnasium-compatible environment for Pokemon TCG.

    The agent plays as Player 0 (Miraidon by default).
    The opponent (Player 1, Charizard/Pidgeot by default) is auto-played
    by a bot.

    During Setup/Mulligan phases where both players can act, the
    environment auto-plays the opponent's setup actions interleaved
    with the agent's.

    Observation space: Box(float32, shape=(obs_dim,))
    Action space: Discrete(1024) with invalid action masking via info dict.
    """

    metadata = {"render_modes": ["text"]}

    def __init__(
        self,
        deck1: str = "miraidon",
        deck2: str = "charizard_pidgeot",
        seed: int | None = None,
        opponent: str = "random",
        render_mode: str | None = None,
        max_turns: int = 50,
    ):
        """Initialize the PTCG environment.

        Args:
            deck1: Deck preset for Player 0 (agent). "miraidon" or "charizard_pidgeot".
            deck2: Deck preset for Player 1 (opponent).
            seed: Random seed for reproducibility.
            opponent: Opponent bot type: "random" or "heuristic".
            render_mode: "text" for human-readable output.
            max_turns: Maximum turns before truncation.
        """
        self._seed = seed if seed is not None else np.random.randint(0, 2**31 - 1)
        self._deck1 = deck1
        self._deck2 = deck2
        self._max_turns = max_turns
        self._render_mode = render_mode

        # Lazy-import Rust module (built separately with maturin)
        try:
            from ptcg_py import PyEngine, create_engine
        except ImportError:
            raise ImportError(
                "ptcg_py Rust module not found. "
                "Build it with: maturin develop --release -m crates/ptcg-py/Cargo.toml"
            )

        self._engine = create_engine(self._seed)

        # Opponent bot
        if opponent == "heuristic":
            self._opponent = HeuristicOpponent()
        else:
            self._opponent = RandomOpponent()

        # Gym spaces
        if GYM_AVAILABLE:
            obs_dim = self._engine.observation_dim()
            self.observation_space = spaces.Box(
                low=-1.0, high=1.0, shape=(obs_dim,), dtype=np.float32
            )
            self.action_space = spaces.Discrete(self._engine.action_space_size())
        else:
            self.observation_space = None
            self.action_space = None

        # Seed Python random for opponent reproducibility
        _random.seed(self._seed)

        self._step_count = 0
        self._total_reward = 0.0

    # ── Gym API ────────────────────────────────────────────────────

    def reset(self, *, seed=None, options=None):
        """Reset the environment for a new episode.

        Returns:
            (observation, info) tuple.
        """
        if seed is not None:
            self._seed = seed
            _random.seed(self._seed)

        self._engine.reset(self._seed)
        self._step_count = 0
        self._total_reward = 0.0

        # Auto-play through setup/mulligan until agent (Player 0) can act
        self._auto_advance_to_agent_turn()

        obs = self._get_obs()
        info = {"action_mask": self._get_action_mask()}
        return obs, info

    def step(self, action: int):
        """Execute an action for the agent (Player 0).

        The opponent's moves are auto-played after the agent's action.

        Args:
            action: Integer action index (0..1023). Must be legal.

        Returns:
            (observation, reward, terminated, truncated, info) tuple.
        """
        self._step_count += 1
        truncated = self._step_count >= self._max_turns

        # Validate action
        mask = self._action_mask
        if mask is not None and (action >= len(mask) or not mask[action]):
            # Invalid action: return small penalty, game continues
            obs = self._get_obs()
            return obs, -0.05, False, False, {
                "action_mask": self._get_action_mask(),
                "error": "invalid_action",
            }

        # Execute agent's action
        obs_list, reward, done, winner, turn, phase, events = self._engine.step(0, action)
        self._total_reward += reward

        if done:
            obs = np.array(obs_list, dtype=np.float32)
            return obs, reward, True, False, {
                "action_mask": self._get_action_mask(),
                "winner": winner,
                "turn": turn,
                "phase": phase,
                "total_reward": self._total_reward,
            }

        # Auto-play opponent moves until agent can act or game ends
        self._auto_advance_to_agent_turn()

        if self._engine.is_done():
            obs_list, _, _, winner, turn, phase, _ = self._engine.step(0, 0)
            obs = np.array(obs_list, dtype=np.float32)
            return obs, reward, True, False, {
                "action_mask": self._get_action_mask(),
                "winner": winner,
                "turn": turn,
                "phase": phase,
                "total_reward": self._total_reward,
            }

        obs = self._get_obs()
        info = {
            "action_mask": self._get_action_mask(),
            "turn": self._engine.turn(),
            "phase": self._engine.phase(),
            "total_reward": self._total_reward,
        }

        return obs, reward, False, truncated, info

    def action_masks(self) -> np.ndarray:
        """Return boolean action mask for MaskablePPO."""
        return self._get_action_mask()

    def render(self):
        """Render the current game state."""
        if self._render_mode == "text":
            from .render import format_game_state
            print(format_game_state(self._engine))

    def close(self):
        """Cleanup. The Rust engine is dropped when the Python object is GC'd."""
        pass

    # ── Helpers ─────────────────────────────────────────────────────

    def _auto_advance_to_agent_turn(self, max_iterations: int = 200):
        """Auto-play opponent moves until the agent (Player 0) can act.

        During Setup/Mulligan phases, both players act simultaneously.
        We auto-play the opponent's setup actions fully, then return control
        to the agent for their setup actions.
        """
        for _ in range(max_iterations):
            if self._engine.is_done():
                break

            acting = self._engine.acting_players()
            is_multiactor = len(acting) > 1  # Setup/Mulligan phase

            if is_multiactor:
                # During setup: fully auto-play P1's setup, then let P0 act
                if 1 in acting:
                    p1_mask = self._engine.action_mask(1)
                    p1_legal = [j for j, m in enumerate(p1_mask) if m]
                    if p1_legal:
                        action = self._opponent.select_action(self._engine, player_id=1)
                        self._engine.step(1, action)
                        continue  # Keep going until P1 is done or phase changes
                # P1 done with setup, let P0 act
                if 0 in acting:
                    break
                break
            else:
                # Normal turn-based play
                if 0 in acting:
                    break  # Agent can act now
                if 1 in acting:
                    action = self._opponent.select_action(self._engine, player_id=1)
                    self._engine.step(1, action)
                else:
                    break  # Nobody can act

    def _get_obs(self) -> np.ndarray:
        obs_list = self._engine.observe(0)
        return np.array(obs_list, dtype=np.float32)

    def _get_action_mask(self) -> np.ndarray | None:
        mask = self._engine.action_mask(0)
        return np.array(mask, dtype=bool)

    @property
    def _action_mask(self) -> np.ndarray | None:
        """Cached action mask (lazy)."""
        return self._get_action_mask()

    # ── Diagnostics ─────────────────────────────────────────────────

    def seed_value(self) -> int:
        """Return the current seed."""
        return self._seed

    @property
    def engine(self):
        """Access the underlying PyEngine for debugging."""
        return self._engine


# Register as Gymnasium env if gymnasium is available
if GYM_AVAILABLE:

    gym.register(
        id="PTCG-v0",
        entry_point="ptcg_gym.env:PTCGEnv",
        max_episode_steps=500,
        kwargs={},
    )

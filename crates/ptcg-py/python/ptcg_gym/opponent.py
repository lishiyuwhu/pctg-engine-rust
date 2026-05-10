"""Opponent bots for PTCG Gym environment.

Provides bot implementations that auto-play the opponent (Player 1)
during RL training.
"""

import random as _random
from abc import ABC, abstractmethod


class OpponentBot(ABC):
    """Base class for opponent bots."""

    @abstractmethod
    def select_action(self, engine, player_id: int) -> int:
        """Select an action index (0..action_space_size) for the given player."""
        ...


class RandomOpponent(OpponentBot):
    """Randomly selects among legal actions."""

    def select_action(self, engine, player_id: int) -> int:
        mask = engine.action_mask(player_id)
        legal = [i for i, m in enumerate(mask) if m]
        if not legal:
            return 0  # fallback — will be rejected if truly illegal
        return _random.choice(legal)


class HeuristicOpponent(OpponentBot):
    """Simple priority-based heuristic opponent.

    Priority order (high to low):
        Attack > Evolve > AttachEnergy > PlayTrainer >
        PlayBasicToBench > UseAbility > AttachTool >
        Retreat > EndTurn > Pass
    """

    _PRIORITY = {
        "Attack": 10,
        "Evolve": 9,
        "AttachEnergy": 8,
        "PlayTrainer": 7,
        "PlayStadium": 7,
        "AttachTool": 6,
        "PlayBasicToBench": 5,
        "SetupChooseActive": 8,
        "SetupBenchBasics": 5,
        "UseAbility": 4,
        "MulliganDraw": 3,
        "Retreat": 2,
        "EndTurn": 1,
        "Pass": 0,
    }

    def select_action(self, engine, player_id: int) -> int:
        dicts = engine.legal_actions_dicts(player_id)
        mask = engine.action_mask(player_id)
        legal = [i for i, m in enumerate(mask) if m]

        if not legal:
            return 0

        if len(legal) == 1:
            return legal[0]

        # Score each legal action by its type priority
        best_idx = legal[0]
        best_score = -1
        for idx in legal:
            if idx < len(dicts):
                import json
                try:
                    d = json.loads(dicts[idx])
                except (json.JSONDecodeError, IndexError):
                    d = {}
                action_type = d.get("type", "Pass")
            else:
                action_type = "Pass"
            score = self._PRIORITY.get(action_type, 0)
            if score > best_score:
                best_score = score
                best_idx = idx

        return best_idx

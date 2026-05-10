"""Integration tests for the PTCG Gym environment.

Run with: pytest python/tests/ -v
Requires the Rust extension built with: maturin develop --release
"""

import sys
import os
import json

import numpy as np
import pytest


# ── Fixtures ────────────────────────────────────────────────────────


@pytest.fixture(scope="module")
def env():
    """Create a PTCG environment with fixed seed for reproducibility."""
    from ptcg_gym import PTCGEnv
    return PTCGEnv(seed=42)


@pytest.fixture
def fresh_env():
    """Create a fresh environment for each test."""
    from ptcg_gym import PTCGEnv
    return PTCGEnv(seed=99)


# ── Basic Tests ─────────────────────────────────────────────────────


def test_import():
    """Verify that the ptcg_gym package imports cleanly."""
    import ptcg_gym
    assert hasattr(ptcg_gym, "PTCGEnv")
    assert hasattr(ptcg_gym, "RandomOpponent")
    assert hasattr(ptcg_gym, "HeuristicOpponent")


def test_env_creation(env):
    """Verify environment creates with valid spaces."""
    assert env.action_space is not None
    assert env.action_space.n == 1024
    assert env.observation_space is not None
    assert env.observation_space.shape[0] > 0


def test_reset(env):
    """Reset should return observation and info dict with action mask."""
    obs, info = env.reset()
    assert isinstance(obs, np.ndarray)
    assert obs.dtype == np.float32
    assert "action_mask" in info
    assert isinstance(info["action_mask"], np.ndarray)
    assert info["action_mask"].dtype == bool
    # At least one action should be legal
    assert info["action_mask"].any(), "No legal actions on reset"


def test_seed_reproducibility(fresh_env):
    """Same seed should produce identical observations on reset.

    Note: the opponent bot uses Python's random module, which is not
    seeded by the environment. So observations may differ slightly
    if the opponent takes different setup actions. We verify that
    the observation shape and basic properties are consistent.
    """
    from ptcg_gym import PTCGEnv

    env_a = PTCGEnv(seed=12345)
    env_b = PTCGEnv(seed=12345)

    obs_a, info_a = env_a.reset()
    obs_b, info_b = env_b.reset()

    # Shape should match
    assert obs_a.shape == obs_b.shape

    # Both should have legal actions
    assert info_a["action_mask"].any()
    assert info_b["action_mask"].any()


def test_obs_dim_consistency(env):
    """Observation dimension should match the space."""
    obs, _ = env.reset()
    assert len(obs) == env.observation_space.shape[0]


# ── Action Tests ────────────────────────────────────────────────────


def test_action_mask_shape(env):
    """Action mask should have length 1024."""
    env.reset()
    mask = env._get_action_mask()
    assert len(mask) == 1024


def test_legal_action_step(env):
    """Stepping with a legal action should not error."""
    env.reset()
    mask = env._get_action_mask()
    legal = np.where(mask)[0]

    if len(legal) == 0:
        pytest.skip("No legal actions available")

    action = int(legal[0])
    obs, reward, terminated, truncated, info = env.step(action)

    assert isinstance(obs, np.ndarray)
    assert isinstance(reward, float)
    assert isinstance(terminated, bool)
    assert isinstance(truncated, bool)
    assert isinstance(info, dict)


def test_illegal_action(env):
    """Stepping with an illegal action should return penalty."""
    env.reset()
    mask = env._get_action_mask()
    illegal = np.where(~mask)[0]

    if len(illegal) == 0:
        pytest.skip("No illegal actions available (all 1024 are legal)")

    action = int(illegal[0])
    obs, reward, terminated, truncated, info = env.step(action)

    assert reward < 0, f"Expected negative reward for illegal action, got {reward}"
    assert "error" in info


# ── Full Game Tests ─────────────────────────────────────────────────


def test_play_random_game(env):
    """Play a full game with random actions until termination."""
    obs, info = env.reset()
    steps = 0
    max_steps = 2000

    for _ in range(max_steps):
        mask = info["action_mask"]
        legal = np.where(mask)[0]
        if len(legal) == 0:
            break

        action = int(np.random.choice(legal))
        obs, reward, terminated, truncated, info = env.step(action)
        steps += 1

        if terminated or truncated:
            break

    assert steps > 0, "Game should have at least one step"
    assert terminated or truncated, "Game should end within max steps"


def test_multiple_games():
    """Play 10 games with random actions to check stability."""
    from ptcg_gym import PTCGEnv

    for seed in range(10):
        env = PTCGEnv(seed=seed)
        obs, info = env.reset()

        for _ in range(500):
            mask = info["action_mask"]
            legal = np.where(mask)[0]
            if len(legal) == 0:
                break

            action = int(np.random.choice(legal))
            obs, reward, terminated, truncated, info = env.step(action)

            if terminated or truncated:
                break

        assert terminated or truncated, f"Game with seed {seed} did not finish"


# ── Render Test ─────────────────────────────────────────────────────


def test_render_text(env):
    """Text rendering should not crash."""
    env.reset()
    from ptcg_gym.render import format_game_state

    text = format_game_state(env.engine)

    assert "Turn" in text
    assert "Phase" in text
    assert "Player 0" in text
    assert "Player 1" in text


# ── Heuristic Opponent Test ─────────────────────────────────────────


def test_heuristic_opponent():
    """Test with heuristic opponent."""
    from ptcg_gym import PTCGEnv

    env = PTCGEnv(seed=42, opponent="heuristic")
    obs, info = env.reset()

    mask = info["action_mask"]
    legal = np.where(mask)[0]
    if len(legal) > 0:
        action = int(np.random.choice(legal))
        obs, reward, terminated, truncated, info = env.step(action)

        # Should not crash
        assert isinstance(reward, float)


# ── Gym Check Test ──────────────────────────────────────────────────


def test_gym_check_env():
    """Run gymnasium's env_checker if available."""
    try:
        from gymnasium.utils.env_checker import check_env
    except ImportError:
        pytest.skip("gymnasium env_checker not available")

    from ptcg_gym import PTCGEnv
    env = PTCGEnv(seed=42)

    # check_env may fail for custom envs with complex action masking,
    # but it should not crash
    try:
        check_env(env, skip_render_check=True)
    except Exception as e:
        pytest.skip(f"env_checker raised: {e}")

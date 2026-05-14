"""RL training configuration for PTCG Phase 1."""

# ── Environment ──
ENV_CONFIG = {
    "deck1": "miraidon",
    "deck2": "charizard_pidgeot",
    "opponent": "random",      # "random" | "heuristic" | "charizard_bot"
    "max_turns": 100,
}

# ── PPO Hyperparameters ──
PPO_CONFIG = {
    "policy": "MlpPolicy",
    "learning_rate": 3e-4,
    "n_steps": 2048,
    "batch_size": 64,
    "n_epochs": 10,
    "gamma": 0.99,
    "gae_lambda": 0.95,
    "clip_range": 0.2,
    "ent_coef": 0.01,
    "vf_coef": 0.5,
    "max_grad_norm": 0.5,
    "verbose": 1,
    "tensorboard_log": "./logs/",
}

# ── Training Loop ──
TRAIN_CONFIG = {
    "total_timesteps": 10_000_000,
    "eval_frequency": 100_000,
    "save_frequency": 500_000,
    "eval_episodes": 100,
    "seed": 42,
}

# ── Reward Design ──
REWARD_CONFIG = {
    "win": 1.0,
    "loss": -1.0,
    "draw": 0.0,
    "prize_taken": 0.05,
    "opponent_ko": 0.05,
    "own_ko": -0.05,
}

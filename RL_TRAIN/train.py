#!/usr/bin/env python3
"""PTCG RL Training — Phase 1: Fixed Miraidon vs Charizard.

Trains a MaskablePPO agent to play Miraidon (Player 0)
against a Charizard opponent (Player 1).

Usage:
    python train.py                    # Start training
    python train.py --eval-only PATH   # Evaluate a saved model
"""

import argparse
import json
import os
import sys
import time
from pathlib import Path

import numpy as np

# Add parent dir for ptcg_gym imports
sys.path.insert(0, str(Path(__file__).parent.parent / "crates" / "ptcg-py" / "python"))

from config import ENV_CONFIG, PPO_CONFIG, TRAIN_CONFIG, REWARD_CONFIG
from evaluate import evaluate


def make_env(seed=None):
    """Create a PTCGEnv with configured opponent."""
    from ptcg_gym import PTCGEnv

    return PTCGEnv(
        deck1=ENV_CONFIG["deck1"],
        deck2=ENV_CONFIG["deck2"],
        seed=seed or TRAIN_CONFIG["seed"],
        opponent=ENV_CONFIG["opponent"],
        max_turns=ENV_CONFIG["max_turns"],
    )


class RewardWrapper:
    """Converts PTCGEnv rewards using the reward config."""

    def __init__(self, env):
        self.env = env
        self._prev_prizes = 6
        self._prev_opp_ko = False

    def step(self, action):
        obs, reward, done, truncated, info = self.env.step(action)

        # Add auxiliary rewards
        aux = 0.0
        # Prize change (approximated from observation)
        # We don't have direct prize event access in step result,
        # so auxiliary rewards are simplified for now.

        total_reward = reward  # Base terminal reward
        if done:
            if info.get("winner") == 0:
                total_reward = REWARD_CONFIG["win"]
            elif info.get("winner") == 1:
                total_reward = REWARD_CONFIG["loss"]
            else:
                total_reward = REWARD_CONFIG["draw"]

        # Small living bonus to encourage not timing out
        if not done and not truncated:
            total_reward += 0.0  # neutral step

        return obs, total_reward, done, truncated, info

    def reset(self, **kwargs):
        obs, info = self.env.reset(**kwargs)
        self._prev_prizes = 6
        return obs, info

    def __getattr__(self, name):
        return getattr(self.env, name)


def main():
    parser = argparse.ArgumentParser(description="PTCG RL Training")
    parser.add_argument("--eval-only", type=str, help="Path to model to evaluate")
    parser.add_argument("--timesteps", type=int, default=TRAIN_CONFIG["total_timesteps"])
    parser.add_argument("--seed", type=int, default=TRAIN_CONFIG["seed"])
    args = parser.parse_args()

    # ── Evaluation-only mode ──
    if args.eval_only:
        from sb3_contrib import MaskablePPO

        model = MaskablePPO.load(args.eval_only)
        env = make_env()
        result = evaluate(env, model, episodes=TRAIN_CONFIG["eval_episodes"])
        print(json.dumps({k: round(v, 3) if isinstance(v, float) else v
                          for k, v in result.items() if k != "raw"}, indent=2))
        return

    # ── Training mode ──
    print("=" * 60)
    print("PTCG RL Training — Phase 1: Miraidon vs Charizard")
    print(f"  Algorithm: MaskablePPO")
    print(f"  Total timesteps: {args.timesteps:,}")
    print(f"  Eval frequency: {TRAIN_CONFIG['eval_frequency']:,}")
    print(f"  Opponent: {ENV_CONFIG['opponent']}")
    print(f"  Observation dim: 140")
    print(f"  Action space: Discrete(1024) + mask")
    print("=" * 60)

    # Create environment
    env = make_env(seed=args.seed)
    env = RewardWrapper(env)

    # Create model
    from sb3_contrib import MaskablePPO

    model = MaskablePPO(
        PPO_CONFIG["policy"],
        env,
        learning_rate=PPO_CONFIG["learning_rate"],
        n_steps=PPO_CONFIG["n_steps"],
        batch_size=PPO_CONFIG["batch_size"],
        n_epochs=PPO_CONFIG["n_epochs"],
        gamma=PPO_CONFIG["gamma"],
        gae_lambda=PPO_CONFIG["gae_lambda"],
        clip_range=PPO_CONFIG["clip_range"],
        ent_coef=PPO_CONFIG["ent_coef"],
        vf_coef=PPO_CONFIG["vf_coef"],
        max_grad_norm=PPO_CONFIG["max_grad_norm"],
        verbose=PPO_CONFIG["verbose"],
        tensorboard_log=PPO_CONFIG["tensorboard_log"],
        seed=args.seed,
    )

    # Training loop
    ckpt_dir = Path("checkpoints")
    ckpt_dir.mkdir(exist_ok=True)
    eval_env = make_env(seed=999)  # Fixed seed for evaluation

    total_steps = 0
    iteration = 0
    t0 = time.time()

    while total_steps < args.timesteps:
        steps_this_iter = min(TRAIN_CONFIG["eval_frequency"],
                              args.timesteps - total_steps)

        model.learn(total_timesteps=steps_this_iter, reset_num_timesteps=False)
        total_steps += steps_this_iter
        iteration += 1

        elapsed = time.time() - t0
        fps = total_steps / max(elapsed, 0.1)

        # Evaluate
        result = evaluate(eval_env, model, episodes=TRAIN_CONFIG["eval_episodes"])
        print(f"\n[Iter {iteration}] {total_steps:>10,} steps | "
              f"Win: {result['win_rate']:.1%} | "
              f"Loss: {result['loss_rate']:.1%} | "
              f"Draw: {result['draw_rate']:.1%} | "
              f"Turns: {result['avg_turns']:.0f} | "
              f"{fps:,.0f} steps/s")

        # Save checkpoint
        if total_steps % TRAIN_CONFIG["save_frequency"] == 0:
            path = ckpt_dir / f"ppo_step_{total_steps // 1_000_000}M"
            model.save(str(path))
            print(f"  -> Saved {path}")

    # Final save
    model.save(str(ckpt_dir / "ppo_final"))
    print(f"\nTraining complete. Total time: {elapsed:.0f}s")


if __name__ == "__main__":
    main()

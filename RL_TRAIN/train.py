#!/usr/bin/env python3
"""PTCG RL Training — Phase 1: Fixed Miraidon vs Charizard.

Trains a MaskablePPO agent to play Miraidon (Player 0)
against a Charizard opponent (Player 1).

Logs all metrics to Weights & Biases (wandb).

Usage:
    python train.py                      # Start training (requires wandb login)
    python train.py --no-wandb           # Train without wandb
    python train.py --eval-only PATH     # Evaluate a saved model
    python train.py --wandb-offline      # Use wandb offline mode
"""

import argparse
import datetime
import json
import os
import sys
import time
from pathlib import Path

import numpy as np

# Add parent dir for ptcg_gym imports
sys.path.insert(0, str(Path(__file__).parent.parent / "crates" / "ptcg-py" / "python"))

from config import ENV_CONFIG, PPO_CONFIG, TRAIN_CONFIG, REWARD_CONFIG, WANDB_CONFIG


# ═══════════════════════════════════════════════════════════════════════
# Environment helpers
# ═══════════════════════════════════════════════════════════════════════

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


# ═══════════════════════════════════════════════════════════════════════
# Evaluation
# ═══════════════════════════════════════════════════════════════════════

def evaluate(env, model, episodes=100):
    """Run fixed evaluation and return metrics dict."""
    results = {"wins": 0, "losses": 0, "draws": 0, "turns": 0, "steps": 0}

    for ep in range(episodes):
        obs, info = env.reset(seed=env.seed_value() + ep)
        done, truncated = False, False

        while not done and not truncated:
            mask = info.get("action_mask")
            if mask is not None and mask.any():
                action, _ = model.predict(obs, action_masks=mask, deterministic=True)
            else:
                legal = np.where(mask)[0] if mask is not None else [0]
                action = int(np.random.choice(legal)) if len(legal) > 0 else 0

            obs, reward, done, truncated, info = env.step(int(action))

        winner = info.get("winner")
        if winner == 0:
            results["wins"] += 1
        elif winner == 1:
            results["losses"] += 1
        else:
            results["draws"] += 1

        results["turns"] += info.get("turn", 0)
        results["steps"] += 1

    n = max(episodes, 1)
    return {
        "win_rate": results["wins"] / n,
        "loss_rate": results["losses"] / n,
        "draw_rate": results["draws"] / n,
        "avg_turns": results["turns"] / n,
        "raw": results,
    }


# ═══════════════════════════════════════════════════════════════════════
# Wandb Eval Callback
# ═══════════════════════════════════════════════════════════════════════

class WandbEvalCallback:
    """SB3-compatible callback that logs eval metrics to wandb."""

    def __init__(self, eval_env, eval_freq, eval_episodes, use_wandb, model_save_dir):
        self.eval_env = eval_env
        self.eval_freq = eval_freq
        self.eval_episodes = eval_episodes
        self.use_wandb = use_wandb
        self.model_save_dir = Path(model_save_dir)
        self.model_save_dir.mkdir(exist_ok=True)
        self._step = 0
        self._best_win_rate = 0.0
        self._t0 = time.time()

    def _on_step(self, current_steps) -> bool:
        if current_steps - self._step < self.eval_freq:
            return True
        self._step = current_steps

        result = evaluate(self.eval_env, self.model, episodes=self.eval_episodes)
        elapsed = time.time() - self._t0

        print(f"\n[Eval @ {current_steps:>10,} steps] "
              f"Win: {result['win_rate']:.1%} | "
              f"Loss: {result['loss_rate']:.1%} | "
              f"Draw: {result['draw_rate']:.1%} | "
              f"Turns: {result['avg_turns']:.0f} | "
              f"Elapsed: {elapsed:.0f}s")

        if self.use_wandb:
            import wandb
            wandb.log({
                "eval/win_rate": result["win_rate"],
                "eval/loss_rate": result["loss_rate"],
                "eval/draw_rate": result["draw_rate"],
                "eval/avg_turns": result["avg_turns"],
                "eval/total_steps": current_steps,
                "system/elapsed_seconds": elapsed,
                "system/steps_per_second": current_steps / max(elapsed, 0.1),
            }, step=current_steps)

        # Save best model
        if result["win_rate"] > self._best_win_rate:
            self._best_win_rate = result["win_rate"]
            path = self.model_save_dir / "ppo_best"
            self.model.save(str(path))
            print(f"  -> New best model (win_rate={result['win_rate']:.3f})")

        return True

    def init_callback(self, model):
        self.model = model


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════

def main():
    parser = argparse.ArgumentParser(description="PTCG RL Training")
    parser.add_argument("--eval-only", type=str, help="Path to model .zip to evaluate")
    parser.add_argument("--timesteps", type=int, default=TRAIN_CONFIG["total_timesteps"])
    parser.add_argument("--seed", type=int, default=TRAIN_CONFIG["seed"])
    parser.add_argument("--no-wandb", action="store_true", help="Disable wandb logging")
    parser.add_argument("--wandb-offline", action="store_true", help="Wandb offline mode")
    parser.add_argument("--wandb-name", type=str, default=None, help="Wandb run name")
    parser.add_argument("--wandb-tags", type=str, nargs="*", help="Extra wandb tags")
    args = parser.parse_args()

    use_wandb = not args.no_wandb

    # ── Evaluation-only mode ──
    if args.eval_only:
        from sb3_contrib import MaskablePPO
        model = MaskablePPO.load(args.eval_only)
        env = make_env()
        result = evaluate(env, model, episodes=TRAIN_CONFIG["eval_episodes"])
        print(json.dumps({k: round(v, 3) if isinstance(v, float) else v
                          for k, v in result.items() if k != "raw"}, indent=2))
        return

    # ── Wandb init ──
    if use_wandb:
        import wandb
        wandb_mode = "offline" if args.wandb_offline else "online"
        tags = list(WANDB_CONFIG.get("tags", []))
        if args.wandb_tags:
            tags.extend(args.wandb_tags)

        run_name = args.wandb_name or WANDB_CONFIG.get("name")
        if run_name is None:
            run_name = f"phase1-{datetime.datetime.now().strftime('%Y%m%d-%H%M%S')}"

        wandb.init(
            project=WANDB_CONFIG["project"],
            name=run_name,
            entity=WANDB_CONFIG.get("entity"),
            tags=tags,
            notes=WANDB_CONFIG.get("notes", ""),
            mode=wandb_mode,
            config={
                "env": ENV_CONFIG,
                "ppo": {k: v for k, v in PPO_CONFIG.items() if k != "tensorboard_log"},
                "train": TRAIN_CONFIG,
                "reward": REWARD_CONFIG,
                "observation_dim": 140,
                "action_space": 1024,
            },
        )
        print(f"wandb run: {wandb.run.url}" if wandb.run else "wandb offline")

    # ── Print config ──
    print("=" * 60)
    print("PTCG RL Training — Phase 1: Miraidon vs Charizard")
    print(f"  Wandb: {'enabled' if use_wandb else 'disabled'}")
    print(f"  Algorithm: MaskablePPO")
    print(f"  Policy: {PPO_CONFIG['policy']}")
    print(f"  Total timesteps: {args.timesteps:,}")
    print(f"  Eval frequency: {TRAIN_CONFIG['eval_frequency']:,}")
    print(f"  Opponent: {ENV_CONFIG['opponent']}")
    print("=" * 60)

    # ── Create env & model ──
    env = make_env(seed=args.seed)

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
        tensorboard_log=PPO_CONFIG["tensorboard_log"] if not use_wandb else None,
        seed=args.seed,
    )

    # ── Training loop ──
    eval_env = make_env(seed=999)
    eval_callback = WandbEvalCallback(
        eval_env=eval_env,
        eval_freq=TRAIN_CONFIG["eval_frequency"],
        eval_episodes=TRAIN_CONFIG["eval_episodes"],
        use_wandb=use_wandb,
        model_save_dir="checkpoints",
    )
    eval_callback.init_callback(model)

    ckpt_dir = Path("checkpoints")
    ckpt_dir.mkdir(exist_ok=True)
    total_steps = 0
    iteration = 0

    while total_steps < args.timesteps:
        steps_this_iter = min(TRAIN_CONFIG["eval_frequency"],
                              args.timesteps - total_steps)

        model.learn(total_timesteps=steps_this_iter, reset_num_timesteps=False)
        total_steps += steps_this_iter
        iteration += 1

        # Eval + wandb logging
        eval_callback._on_step(total_steps)

        # Periodic checkpoint save
        if total_steps % TRAIN_CONFIG["save_frequency"] == 0:
            path = ckpt_dir / f"ppo_step_{total_steps // 1_000_000}M"
            model.save(str(path))
            print(f"  -> Saved checkpoint {path}")

    # Final save
    model.save(str(ckpt_dir / "ppo_final"))
    print(f"\nTraining complete.")

    if use_wandb:
        import wandb
        wandb.finish()


if __name__ == "__main__":
    main()

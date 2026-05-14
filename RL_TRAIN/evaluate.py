"""Fixed evaluation for PTCG RL agent."""

import json
import numpy as np
from ptcg_gym import PTCGEnv


def evaluate(env, model, episodes=100, render=False):
    """Evaluate an RL model against the environment's opponent.

    Args:
        env: PTCGEnv instance.
        model: SB3 model with .predict(obs, action_masks=mask) method.
        episodes: Number of evaluation episodes.
        render: If True, print game states.

    Returns:
        dict with win_rate, loss_rate, draw_rate, avg_turns.
    """
    results = {"wins": 0, "losses": 0, "draws": 0, "turns": 0, "steps": 0}

    for ep in range(episodes):
        obs, info = env.reset(seed=env.seed_value() + ep)
        done = False
        truncated = False
        ep_steps = 0

        while not done and not truncated:
            mask = info.get("action_mask", None)
            if mask is not None and mask.any():
                action, _ = model.predict(obs, action_masks=mask, deterministic=True)
            else:
                # Fallback: random legal action
                legal = np.where(mask)[0] if mask is not None else [0]
                action = int(np.random.choice(legal)) if len(legal) > 0 else 0

            obs, reward, done, truncated, info = env.step(int(action))
            ep_steps += 1

            if render and ep < 3:
                print(f"  Eval ep {ep}, step {ep_steps}: reward={reward:.2f}, "
                      f"done={done}, phase={info.get('phase', '?')}")

        winner = info.get("winner")
        if winner is not None and winner >= 0:
            if winner == 0:
                results["wins"] += 1
            else:
                results["losses"] += 1
        else:
            results["draws"] += 1

        results["turns"] += info.get("turn", 0)
        results["steps"] += ep_steps

    n = max(episodes, 1)
    return {
        "win_rate": results["wins"] / n,
        "loss_rate": results["losses"] / n,
        "draw_rate": results["draws"] / n,
        "avg_turns": results["turns"] / n,
        "avg_steps": results["steps"] / n,
        "raw": results,
    }


if __name__ == "__main__":
    # Quick test: evaluate random policy
    import sys
    sys.path.insert(0, "..")
    from ptcg_gym import PTCGEnv

    class RandomModel:
        def predict(self, obs, action_masks=None, deterministic=False):
            mask = action_masks
            legal = np.where(mask)[0] if mask is not None else [0]
            return int(np.random.choice(legal)), None

    env = PTCGEnv(seed=42, opponent="random")
    result = evaluate(env, RandomModel(), episodes=50)
    print(json.dumps({k: round(v, 3) for k, v in result.items() if k != "raw"}, indent=2))

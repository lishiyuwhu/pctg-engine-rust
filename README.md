# PTCG Rust Engine

> **参考来源**: 本项目从 Godot/GDScript 项目 [PtcgDeckAgent](https://github.com/lishiyuwhu/PtcgDeckAgent) 迁移而来，将原有的 GDScript 实现重写为 Rust，并扩展了 Python Gym 接口以支持强化学习训练。

Pokemon TCG game engine written in Rust, designed for reinforcement learning training at scale.

## Features

- **54 card definitions** — Miraidon ex, Charizard ex, Pidgeot ex, and supporting Pokemon/Trainers/Tools/Stadiums
- **2x 60-card deck templates** — Competitive Miraidon ex vs Charizard ex / Pidgeot ex matchup
- **Full game engine** — Setup, mulligan, draw, play, attack, damage, weakness/resistance, KO, prize cards
- **Python Gym interface** — Drop-in `gym.Env` compatible with Stable-Baselines3
- **High performance** — 3,500+ games/second parallel batch simulation
- **140-dim observation** vector for neural network input
- **1024-dim action space** with invalid action masking

## Architecture

```
ptcg-rust-engine/
├── Cargo.toml              # Workspace configuration
├── manifest/               # Card and deck YAML definitions
│   ├── cards.yaml
│   └── decks/
├── crates/
│   ├── ptcg-core/          # Core engine library
│   │   └── src/
│   │       ├── engine.rs       # Game loop, phase transitions
│   │       ├── state.rs        # GameState, PlayerState, PokemonSlot
│   │       ├── card.rs         # CardDef, CardRegistry (54 cards)
│   │       ├── deck.rs         # Deck, MatchConfig, templates
│   │       ├── action.rs       # Action enum, Choices
│   │       ├── rules.rs        # Rule validation
│   │       ├── damage.rs       # Damage calculation (weakness/resistance/modifiers)
│   │       ├── observe.rs      # Observation encoding (140-dim vector)
│   │       ├── effects/        # Card effect implementations
│   │       │   ├── pokemon.rs  # Abilities & attacks
│   │       │   ├── trainers.rs # Trainer card effects
│   │       │   ├── tools.rs    # Tool card effects
│   │       │   ├── stadiums.rs # Stadium card effects
│   │       │   └── dispatch.rs # Effect dispatch
│   │       ├── rng.rs          # Deterministic RNG (ChaCha8)
│   │       └── replay.rs       # Game replay system
│   ├── ptcg-sim/           # CLI benchmark tool
│   │   └── src/main.rs
│   └── ptcg-py/            # Python bindings (PyO3)
│       ├── src/
│       │   ├── lib.rs          # PyEngine, PyBatchRunner
│       │   └── action_codec.rs # Action encoding/decoding
│       └── python/
│           ├── ptcg_gym/       # Gymnasium environment
│           │   ├── env.py      # PTCGEnv
│           │   ├── opponent.py # Random/Heuristic bots
│           │   └── render.py   # Text renderer
│           └── tests/
```

## Quick Start

### Prerequisites

- Rust 1.70+
- Python 3.10+
- maturin (`pip install maturin`)

### Build & Test (Rust)

```bash
# Run all tests
cargo test -p ptcg-core
cargo test -p ptcg-py

# Run benchmark (10,000 games, parallel)
cargo run -p ptcg-sim --release -- -g 10000
```

### Build & Test (Python)

```bash
# Build Python extension
maturin develop --release -m crates/ptcg-py/Cargo.toml

# Install dependencies
pip install gymnasium numpy

# Test Gym environment
PYTHONPATH=crates/ptcg-py/python python3 -c "
from ptcg_gym import PTCGEnv
import numpy as np

env = PTCGEnv(seed=42)
obs, info = env.reset()
mask = info['action_mask']
legal = np.where(mask)[0]
action = int(np.random.choice(legal))
obs, reward, terminated, truncated, info = env.step(action)
print(f'Obs shape: {obs.shape}, Reward: {reward}')
"
```

## Usage

### Python Gym Environment

```python
from ptcg_gym import PTCGEnv
import numpy as np

# Create environment
env = PTCGEnv(
    seed=42,                    # Reproducibility
    opponent="random",          # "random" or "heuristic"
    max_turns=100,              # Truncation limit
)

# Reset
obs, info = env.reset()
action_mask = info["action_mask"]  # Boolean array (1024,) — legal actions

# Step
legal_actions = np.where(action_mask)[0]
action = int(np.random.choice(legal_actions))
obs, reward, terminated, truncated, info = env.step(action)

# Render
env.render()
```

### Python Batch Simulation

```python
from ptcg_py import run_batch
import json

# Run 1000 games in parallel
result_json = run_batch(1000, seed=42, threads=None)
stats = json.loads(result_json)
print(f"Player 0 wins: {stats['player0_wins']}")
print(f"Player 1 wins: {stats['player1_wins']}")
print(f"Draws: {stats['draws']}")
```

### Rust CLI Benchmark

```bash
# Basic
cargo run -p ptcg-sim --release -- -g 1000

# Parallel threads control
cargo run -p ptcg-sim --release -- -g 10000 -t 8

# Save results to file
cargo run -p ptcg-sim --release -- -g 10000 -o results.json
```

### Rust Library

```rust
use ptcg_core::{
    deck::{MatchConfig, templates, StartingPlayer},
    engine::Engine,
    state::PlayerId,
};

let config = MatchConfig {
    player_deck: templates::miraidon_deck(),
    opponent_deck: templates::charizard_pidgeot_deck(),
    player_name: "Alice".into(),
    opponent_name: "Bob".into(),
    starting_player: StartingPlayer::Random,
};

let mut engine = Engine::new(config, 42);

// Check legal actions
let actions = engine.legal_actions(PlayerId(0));

// Execute an action
let result = engine.step(PlayerId(0), actions[0].clone());

// Check game state
println!("Turn: {}", engine.state().turn.turn_number);
println!("Winner: {:?}", engine.state().winner);
```

## Observation Space (140-dim)

| Feature Group | Dims | Description |
|---|---|---|
| Turn/phase | 10 | Turn number, phase one-hot (8), is active player |
| Deck/hand/prizes | 6 | Sizes for both players |
| Active Pokemon (self) | 8 | HP, max HP, damage, energy count, tool, is EX, is V, attacks |
| Active Pokemon (opponent) | 8 | Same fields |
| Bench (self, 5 slots) | 30 | Per slot: occupied, HP, damage, energy, is EX, stage |
| Bench (opponent, 5 slots) | 30 | Same fields |
| Energy (self active) | 10 | Energy count + type one-hot (9 types) |
| Energy (opponent active) | 10 | Same fields |
| Bench energy | 4 | Total bench energy + avg per slot, both players |
| Hand composition | 8 | Placeholder for card type breakdown |
| Stadium | 3 | Stadium present flags |
| Action flags | 6 | Can attack, attack locked, can retreat, etc. |
| Discard/lost zone | 4 | Discard and lost zone sizes |
| Padding | 3 | Reserved for future expansion |

## Action Space (1024-dim Discrete + Mask)

Actions are sorted into canonical order and mapped to flat integer indices 0..1023.
A boolean mask marks which indices are legal in the current state.

**Action types (sorted priority):**
EndTurn, Pass, MulliganDraw, SetupChooseActive, SetupBenchBasics,
PlayBasicToBench, Evolve, AttachEnergy, AttachTool, PlayTrainer,
PlayStadium, UseAbility, Retreat, Attack

## Performance

| Benchmark | Value |
|---|---|
| Rust single-game | ~200 games/s |
| Rust parallel (11 threads) | **1,560 games/s** |
| Python batch (parallel) | **3,518 games/s** |
| 50,000 games | 36s, 0 crashes |

*Measured on Apple M2 Pro, release build with LTO.*

## Known Limitations

- Random play does not produce KOs efficiently (all draws)
- Some Trainer card effects are simplified (data defined, runtime no-op)
- Manaphy's Awaken ability not integrated into damage calculation
- Retreat energy cost check is basic
- Deck-out not implemented as win condition

## License

Apache-2.0

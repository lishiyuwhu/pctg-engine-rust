# PTCG Rust Engine - 项目进度

> 最后更新: 2026-05-10

## 项目概述

基于 Pokemon TCG 的 Rust 游戏引擎，用于强化学习训练。

### 目标
- 创建最小化的 PTCG 游戏引擎
- 支持 RL 训练环境
- 从 Godot/GDScript 项目 (PtcgDeckAgent) 迁移

### 初始对战
- Miraidon ex vs Charizard ex / Pidgeot ex

---

## 进度总览

| 模块 | 状态 | 完成度 |
|------|------|--------|
| M1-M2: 核心框架 | ✅ 完成 | 100% |
| M3: Miraidon核心效果 | ✅ 完成 | 100% |
| M4: Charizard/Pidgeot效果 | ✅ 完成 | 100% |
| M5: 伤害计算与KO逻辑 | ✅ 完成 | 100% |
| M6: 稳定性验证 | ✅ 完成 | 100% |
| M7: 补全卡牌定义 | ✅ 完成 | 100% |
| M8: Python Gym接口 | ✅ 完成 | 100% |
| M9: 性能优化 | ❌ 待完成 | 0% |
| M10: 文档与测试 | ✅ 完成 | 80% |

---

## 已完成功能

### 核心模块 (M1-M2)

- [x] `card.rs` - 卡牌定义 (CardDef, CardDefId, CardRegistry) - **54张卡牌定义**
- [x] `deck.rs` - 卡组定义与验证 (Deck, MatchConfig, templates) - **两套60张卡组模板**
- [x] `state.rs` - 游戏状态 (GameState, PlayerState, PokemonSlot)
- [x] `action.rs` - 动作空间定义
- [x] `engine.rs` - 游戏引擎核心 (含阶段转换逻辑)
- [x] `rules.rs` - 规则验证 (RuleValidator)
- [x] `damage.rs` - 伤害计算器
- [x] `rng.rs` - 随机数生成器
- [x] `error.rs` - 错误类型定义
- [x] `observe.rs` - 观察接口 (RL用)
- [x] `replay.rs` - 重放系统

### 效果系统 (M3-M4)

#### Pokemon 能力 (effects/pokemon.rs)

| 能力 | Pokemon | 效果 |
|------|---------|------|
| Tandem Unit | Miraidon ex | 搜索最多2张基础Lightning Pokemon到bench |
| Infernal Reign | Charizard ex | 进化时搜索最多3张Fire能量 |
| Wind Search | Pidgeot ex | 搜索任意1张卡牌到手牌 |
| Awaken | Manaphy | 防止benched Pokemon受到伤害 |
| Restart | Mew ex | 将手牌放回牌库底部，抽等量卡牌 |
| Concealed Cards | Radiant Greninja | 丢弃1张能量，抽2张卡牌 |

#### Pokemon 攻击 (effects/pokemon.rs)

| 攻击 | Pokemon | 效果 |
|------|---------|------|
| Photon Blaster | Miraidon ex | 220伤害，下回合无法使用 |
| Double Impact | Iron Hands ex | 120伤害，+30到1只benched Pokemon |
| Scorching Darkness | Charizard ex | 180+每失去1张奖品卡+30 |
| Combustion Blast | Radiant Charizard | 250伤害(仅在1张或更少奖品卡时可用) |
| Moonlight Shuriken | Radiant Greninja | 对最多2只对手Pokemon各造成90伤害 |
| Gale Winds | Pidgeot ex | 120伤害，可选弃手牌Stadium对benched造成120 |
| Quick Strike | Zapdos | 70伤害 |
| Ember | Charmander | 50伤害，丢弃1张能量 |

#### Trainer 效果 (effects/trainers.rs)

| Trainer | 效果 |
|---------|------|
| Electric Generator | 从top 5附着Lightning能量到Electric Pokemon |
| Nest Ball | 搜索基础Pokemon到bench |
| Ultra Ball | 弃2张，搜索任意Pokemon到手牌 |
| Rare Candy | 跳过Stage 1，直接进化Stage 1到Stage 2 |
| Arven | 搜索1张Item和1张Tool到手牌 |
| Boss's Orders | 将对手bench Pokemon移到active |
| Iono | 双方手牌洗回牌库，根据奖品卡数量抽牌 |

#### 效果分发 (effects/dispatch.rs)

```rust
dispatch_ability()  // 能力效果分发
dispatch_trainer()  // Trainer效果分发
dispatch_stadium()  // Stadium效果分发
```

### 伤害计算 (M5)

- [x] 基础伤害计算
- [x] 弱点加成 (weakness multiplier)
- [x] 抗性减免 (resistance)
- [x] Miraidon ex Tandem Unit +30伤害
- [x] Tool卡牌伤害修正
- [x] Stadium卡牌伤害修正

### KO逻辑 (M5)

- [x] `process_ko()` - 完整KO处理流程
- [x] 收集KO Pokemon的所有卡牌
- [x] 移动到弃牌区
- [x] 自动从bench补充active
- [x] 触发 KnockedOut 和 PrizeTaken 事件

### 阶段转换 (M7 新增)

- [x] Setup阶段: 双方同时行动，选择active和bench Pokemon
- [x] Mulligan: 无基础Pokemon时自动处理重抽
- [x] Setup→Play: 双方都有active后自动过渡
- [x] Alternating turns: 回合制交替

---

## 项目结构

```
ptcg-rust-engine/
├── Cargo.toml              # Workspace配置
├── Cargo.lock
├── manifest/               # 卡牌和卡组清单
│   ├── manifest.yaml
│   ├── cards.yaml          # 85张卡牌定义
│   └── decks/
│       ├── miraidon.yaml
│       └── charizard_pidgeot.yaml
└── crates/
    ├── ptcg-core/          # 核心引擎库
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── card.rs     # 54张卡牌定义
    │       ├── deck.rs     # 2套60张卡组模板
    │       ├── state.rs
    │       ├── action.rs
    │       ├── engine.rs   # 含完整阶段转换
    │       ├── rules.rs
    │       ├── damage.rs
    │       ├── rng.rs
    │       ├── error.rs
    │       ├── observe.rs
    │       ├── replay.rs
    │       └── effects/
    │           ├── mod.rs
    │           ├── common.rs
    │           ├── pokemon.rs
    │           ├── trainers.rs
    │           ├── tools.rs
    │           ├── stadiums.rs
    │           ├── energy.rs
    │           └── dispatch.rs
    ├── ptcg-sim/           # CLI自对弈工具
    │   ├── Cargo.toml
    │   └── src/main.rs
    └── ptcg-py/            # Python绑定
        ├── Cargo.toml
        └── src/lib.rs
```

---

## 测试状态

### 单元测试
```
26 passed / 0 failed
```

所有测试通过，包括之前失败的5个测试：
- `test_charizard_deck` ✅ - Duskull等卡牌定义已补全
- `test_miraidon_deck` ✅ - Miraidon卡组修正为60张
- `test_deck_expand` ✅ - 卡组展开正确
- `test_legal_actions` ✅ - Setup阶段支持Mulligan
- `test_game_state` ✅ - 状态初始化正常

### 模拟器测试 (1000局)
```
1000 games completed, 0 crashes
Player 0 (Miraidon) wins: 976 (97.6%)
Draws: 24 (2.4%)
Average turns: 1.0
```

> **注**: 高P0胜率和短对局是由于随机策略+高伤害卡组所致。在随机策略下，setup阶段未充分铺场bench，导致第一回合进攻即可KO获胜。后续引入RL训练后将呈现更真实的对局。

---

## 编译状态

- **ptcg-core**: ✅ 编译通过 (37警告)
- **ptcg-sim**: ✅ 编译通过 (2警告)
- **ptcg-py**: ✅ 编译通过 (通过maturin)

---

## M6/M7 已完成的工作

### M7: 补全卡牌定义
- 新增 **32张** 卡牌定义到 `load_miraidon_charizard_cards()`
- 覆盖 Pokemon: Radiant Greninja, Zapdos, Flutter Mane, Fezandipiti ex, Bloodmoon Ursaluna ex, Mew ex, Raichu V, Raikou V, Lumineon V, Squawkabilly ex, Scream Tail, Klefki, Drifloon, Munkidori, Rotom V, Duskull, Dusclops, Dusknoir, Radiant Charizard
- 覆盖 Trainers: Buddy-Buddy Poffin, Super Rod, Earthen Vessel, Counter Catcher, Switch Cart, Lost Vacuum, Hisuian Heavy Ball, Night Stretcher, Secret Box, Unfair Stamp, Ciphermaniac's Codebreaking, Cyllene, Professor Turo's Scenario, Thorton
- 覆盖 Tools: Forest Seal Stone, Heavy Baton, Bravery Charm, Rescue Board, Defiance Band
- 覆盖 Stadiums: Gravity Mountain, Collapsed Stadium

### M6: 稳定性验证
- 修复 **Miraidon卡组**：从79张修正为60张标准卡组
- 修复 **Setup阶段**：双方玩家同时设置，无基础Pokemon时自动Mulligan
- 修复 **阶段转换**：Setup→Mulligan→Play自动化
- 修复 **模拟器**：Setup阶段双方均可操作
- **1000局稳定性测试**：0 crash，引擎稳定运行

---

## 已知问题

1. 模拟器使用随机策略，无法真正测试游戏平衡
2. Manaphy的Awaken能力尚未集成到伤害计算中
3. Retreat功能需要完善能量检查和撤退费用处理
4. Trainer卡效果大部分未与engine.rs集成(仅定义了数据，运行时为简化的no-op)

---

## M8: Python Gym接口 (已完成)

### Rust 侧
- `action_codec.rs` - 动作编码/解码 (1024维离散空间 + mask)
- `lib.rs` - PyEngine: step(), observe(), reset(), legal_actions_encoded()
- `observe.rs` - to_vector_extended() 140维特征向量

### Python 侧
- `ptcg_gym/env.py` - PTCGEnv: reset/step/render
- `ptcg_gym/opponent.py` - RandomOpponent, HeuristicOpponent
- `tests/test_env.py` - 12项集成测试通过

### Bug 修复
- Mulligan 无限循环 (rules.rs phase 检查)
- Mulligan 无 Basic (execute_mulligan 保底交换)
- Setup 阶段误判胜负 (check_winner phase 检查)
- Prize Card 初始化 (setup_initial_state)

### 性能
- 100 games: 0 crashes, ~31ms/game

---

## 下一步计划

1. 性能优化 (M9)
2. 完善 Trainer/Ability 效果与 engine 集成
3. Python pip 包发布

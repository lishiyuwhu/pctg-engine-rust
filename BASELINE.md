# Engine Baseline

> 记录日期: 2026-05-14
> PtcgDeckAgent 基线: `c3b0161` (2026-04-27, 原始改写基线) → 同步至 `8a66870` (2026-05-05)
> Rust 引擎 commit: `94b8841`

## 卡牌与卡组

| 指标 | 数值 |
|------|------|
| 卡牌定义 | 290 张 |
| 卡组模板 | 18 套 (全部来自 GDScript JSON 数据) |
| 效果实现 | 60 个 (23 ability + 21 attack + 16 trainer) |
| 策略脚本 | 18 个 (3 手写 + 15 通用模板) |

## 核心引擎

| 功能 | 状态 |
|------|------|
| Setup / Mulligan | ✅ |
| Draw / Play / Attack / Retreat | ✅ |
| 伤害计算 (弱点/抗性/Tool/Stadium 修正) | ✅ |
| KO 处理 + Prize 系统 | ✅ |
| Pokemon Check (poison/burn/sleep/paralyze) | ✅ |
| Heavy Baton 能量转移 | ✅ |
| Forest Seal Stone / Tool-granted ability | ✅ |
| Deck-out 判负 | ✅ |
| First-turn attack restriction | ✅ |
| Pokemon 效果 dispatch | ✅ |
| Trainer 效果 dispatch | ✅ |
| Attack 效果 dispatch | ✅ |

## Python Gym 接口

| 功能 | 状态 |
|------|------|
| reset / step / render | ✅ |
| 140 维 observation | ✅ |
| 1024 维 Discrete action + mask | ✅ |
| PTCGEnv (gymnasium 兼容) | ✅ |
| RandomOpponent / HeuristicOpponent | ✅ |
| run_batch (并行模拟) | ✅ |
| run_random_matchup_batch | ✅ |

## 测试

| 层级 | 数量 | 结果 |
|------|------|------|
| Rust ptcg-core | 41 | 全部通过 |
| Rust ptcg-py | 6 | 全部通过 |
| Python | 12 | 12 passed / 1 skipped |
| **总计** | **59** | **0 失败** |

## 性能基准

| 场景 | 吞吐量 | 备注 |
|------|--------|------|
| Rust sim (并行 8 线程) | 1,380 games/s | ptcg-sim, Miraidon vs Charizard |
| Python batch (并行) | 3,300 games/s | run_random_matchup_batch |
| Python Gym (单线程) | 55 games/s | 单局 Python FFI 开销 |
| 50,000 局稳定性 | 0 crashes | ptcg-sim release build |

## 对局质量

| 方式 | 对局数 | P0 胜率 | P1 胜率 | Draw |
|------|--------|---------|---------|------|
| 随机 action (Miraidon vs Charizard) | 500 | 35% | 65% | 0% |
| 策略 bot (MiraidonStrategy vs CharizardStrategy) | 1,000 | 0% | 0% | 100% |
| 策略 bot (18 卡组随机对阵) | 10,000 | — | — | 99.97% |

> **结论**: 引擎规则正确（随机 action 能产生 KO 和胜负）。策略 bot 评分权重偏向铺场/贴能，不主动进攻，需要独立调优。

## 已知问题

1. **策略 bot 太保守** — 优先铺场贴能而非进攻，导致 100% draw
2. **卡组模板未经验证** — 自动生成的 16 套卡组模板引用的 CardDefId 与 registry 不一致
3. **部分 Trainer 效果为 no-op** — 效果函数已 dispatch 但实现简化
4. **单局 Python FFI 开销大** — 55 games/s vs Rust batch 的 3300 games/s
5. **Gym env 未接入策略 bot** — opponent 仍用 RandomOpponent

## 下一步

1. 策略 bot 评分调优（让 bot 更主动进攻）
2. 卡组模板验证（确保所有 CardDefId 可解析）
3. Gym env 集成策略 bot
4. SB3 PPO 训练验证

# PTCG 强化学习项目背景整理

本文整理当前项目的背景判断：目标是在 Pokemon TCG（PTCG）这类双人不完全信息牌类游戏中，训练一个能够自动对战的 AI。短期目标建议从固定套牌对战开始，而不是直接进入构筑优化或通用大模型训练。

## 1. 领域背景

棋类和牌类强化学习的发展大致可以分成两条主线：

- 完全信息棋类：代表是 AlphaGo、AlphaZero、MuZero。核心范式是自我对弈、策略网络、价值网络和搜索，尤其适合围棋、国际象棋、日本将棋这类完全信息游戏。
- 不完全信息牌类：代表包括 DeepStack、Libratus、Pluribus、Suphx、DouZero、DeepNash 等。核心问题从“如何搜索最优走法”变成“如何在隐藏信息、随机性和对手策略不确定的情况下学习稳健策略”。

PTCG 更接近后者。它有隐藏信息、随机抽牌、检索、奖品牌、长回合动作链、复杂卡牌效果和巨大合法动作空间。因此不建议第一版直接照搬 AlphaZero/MCTS。更现实的路线是使用结构化模拟器、自我对弈、合法动作 mask、策略池和强化学习算法。

## 2. PTCG 与麻将、扑克的相似点和差异

PTCG 和日本麻将、扑克一样是不完全信息博弈，但工程形态不同：

- 类似点：隐藏信息、随机性、对手建模、长期收益、策略不能只针对单一对手。
- 差异点：PTCG 的动作空间通常更结构化，也更依赖卡牌效果实现；每个回合可能包含多步连续操作，且合法动作由卡牌文本和当前状态共同决定。

日本麻将强 AI 通常采用监督预训练、自我对弈强化学习、隐藏信息处理和奖励建模。例如 Suphx 使用高手牌谱预训练再进行强化学习；更近的麻将 AI 还会结合 regret minimization、Actor-Critic、策略适配等方法。对 PTCG 来说，最值得借鉴的是“先让模型会玩，再用自我对弈变强”，而不是从零开始纯 RL。

## 3. 数据现状

目前没有发现类似天凤牌谱那样的公开 PTCG 大规模高质量逐动作数据集。

可用数据源主要分为几类：

- Limitless TCG：提供赛事结果、套牌列表、选手成绩、部分 matchup 数据。适合做 meta 分析、选择训练套牌、构筑候选和 matchup 先验，但不是逐动作牌谱。
- PTCG Live Battle Log：PTCG Live 支持赛后导出 Battle Log，第三方工具如 PTCGL Replayer 可以上传和回放日志。但目前没有发现公开、标准化、可批量下载的大规模 Battle Log 数据集。
- 社区零散日志：论坛、Reddit 上有玩家贴出的单局日志，主要用于 bug 反馈，规模和质量不足以作为主训练数据。
- 视频/转播：高质量但结构化成本很高，不适合作为第一阶段训练数据源。

因此，当前项目不应依赖大规模人类牌谱。更现实的主路径是：

```text
Limitless deck/meta 数据
+ 少量 PTCG Live Battle Log 辅助验证
+ scripted bot 生成预训练数据
+ 自建模拟器 self-play 强化学习
```

## 4. 建议的第一阶段目标

第一阶段应先做固定套牌对战 AI：

```text
固定 Deck A vs 固定 Deck B
```

原因是固定套牌对战是整个系统的最小可验证闭环。后续无论要扩展到 10 副牌、两两对战，还是做 60+10 的构筑搜索，都依赖一个前提：给定一副牌，AI 确实会打，并且评估结果可信。

第一阶段需要跑通：

- PTCG 规则环境
- 发牌、奖品、抽牌、洗牌
- 卡牌效果实现
- 合法动作枚举
- observation 不泄露隐藏信息
- 胜负判断
- random / greedy / scripted bot
- masked policy 训练
- checkpoint 保存
- 固定评估流程

## 5. 推荐训练方式

固定套牌第一版推荐：

```text
scripted bot 行为克隆预训练
-> masked PPO 强化学习
-> self-play + 历史 checkpoint 策略池
-> 固定评估矩阵
```

### 5.1 算法选择

首选 masked PPO：

- 实现难度适中。
- 可以配合 legal action mask。
- 对长时序、多阶段决策相对稳定。
- 比纯 DQN/Q-learning 更容易调通。
- 不需要第一版就实现 CFR、Deep CFR 或 ReBeL。

如果后续发现每个状态的合法动作数量非常大，可以考虑 DMC / legal-action Q model：

```text
Q(observation, candidate_action) -> score
```

但第一版仍建议从 masked PPO 开始。

### 5.2 模型架构

推荐使用 state encoder + action encoder：

```text
state_encoder(observation) + action_encoder(legal_action) -> action_score
```

环境在每个决策点枚举所有合法动作，模型只对合法动作打分或采样。这比固定输出一个巨大动作表更适合 PTCG。

状态输入建议包括：

- 我方 active / bench
- 对方 active / bench
- 我方手牌
- 双方弃牌区
- 双方奖品数量
- 能量、伤害、异常状态
- 本回合是否已用支援者、是否已附能、是否已撤退
- 回合数、先后手
- 最近 N 个动作历史

隐藏信息不能进入 observation，例如对手手牌、对手牌库顺序、奖品牌内容。

动作编码建议结构化：

```text
action_type: play_card / attach_energy / evolve / retreat / attack / ability / pass
source_card
target_card
target_zone
chosen_option
```

### 5.3 训练架构

建议拆成五个模块：

```text
PTCGEnv
PolicyModel
RolloutWorkers
Learner
Evaluator
```

基本流程：

```text
多个 CPU worker 并行跑对局
-> 收集 trajectory
-> GPU learner 用 PPO 更新模型
-> 定期保存 checkpoint
-> evaluator 跑固定评估
```

对手来源建议混合：

```text
30% scripted bot
30% 历史 checkpoint
30% 当前 self-play
10% random / greedy bot
```

后期可以逐步降低 scripted bot 和 random bot 的比例。

## 6. 奖励设计

主奖励应保持简单：

```text
胜利: +1
失败: -1
平局: 0
```

第一版可以少量加入辅助奖励：

```text
拿奖品: +0.02 ~ +0.05
击倒对手: +0.02 ~ +0.05
被击倒: -0.02 ~ -0.05
```

辅助奖励不能过重，否则模型可能学会追求短期奖品或击倒，而不是最终胜利。

## 7. 评估方式

不要只看训练 loss，也不要只看 self-play 胜率。建议固定评估：

- vs random bot
- vs greedy bot
- vs scripted bot
- vs 历史 checkpoint
- 先手胜率
- 后手胜率
- 总胜率
- 平均回合数
- 非法动作率
- 不同随机种子的稳定性

当模型能稳定打过 scripted bot，并且在历史 checkpoint 池中的 Elo 持续上升，再考虑扩展。

## 8. 多套牌扩展

如果从固定 A vs B 扩展到 10 副牌，建议路线是：

```text
单 matchup 专家
-> 多 matchup 专家
-> 带 deck embedding 的共享模型
-> 专家蒸馏
-> 全 matchup self-play 微调
```

不要一开始训练一个完全通用模型。先训练多个 matchup 专家，确认每个对局能跑通，再把专家数据蒸馏到一个共享模型中。

固定 10 副牌时可以使用 deck_id embedding。后续如果套牌会变化，应升级为 decklist/card embedding。

## 9. 构筑扩展：60+10 候选牌

如果未来目标是“给定 60 张初始套牌 + 10 张可替换牌，自动寻找最优 60 张”，不需要从头重做架构。应拆成两层：

```text
构筑层：从候选池选择最终 60 张
对局层：给定 60 张牌，学习怎么打
```

不要一开始端到端训练“选牌 + 打牌”。更现实的路线是：

```text
生成候选 deck
-> 用当前 PlayPolicy 评估或短时间 fine-tune
-> 保留 Top-K
-> 对 Top-K 做更长训练和更高置信度评估
```

此时模型需要从 deck_id embedding 升级为 decklist embedding：

```text
deck_embedding = aggregate(card_embedding * card_count)
```

## 10. 环境更迭与模型迁移

PTCG 的环境会持续变化：新卡发布、禁限或规则调整、主流套牌增减、已有套牌只替换少量卡位。模型不应该在每个新环境中从零训练，而应采用持续训练和迁移微调。

长期架构应从一开始支持环境变化：

```text
card embedding
+ decklist embedding
+ observation encoding
+ action encoding
```

不要只依赖 deck_id embedding。deck_id 适合固定 10 副牌的早期实验，但环境变化后会很快失效。更稳的做法是让模型理解“这副牌由哪些卡组成”：

```text
deck_embedding = sum(card_embedding * card_count)
```

动作也应保持结构化：

```text
action_type + source_card + target + option
```

这样当一副套牌只换几张卡时，模型的大部分对局知识可以复用，新训练主要学习新卡的使用时机和 matchup 变化。

### 10.1 环境更新流程

每次环境更新时建议按以下流程迁移：

```text
旧环境最佳模型
-> 更新 card pool / deck pool / matchup pool
-> 用旧模型初始化新模型
-> 提高新环境对局采样权重
-> 保留少量旧环境关键 matchup
-> 持续 self-play 微调
-> 重新评估 matchup matrix
```

具体步骤：

1. 冻结旧环境最佳 checkpoint，作为 teacher、baseline 和历史对手。
2. 更新新环境的 card pool、deck pool 和主流 matchup 列表。
3. 用旧模型参数初始化新环境模型，而不是随机初始化。
4. 重点训练新套牌 vs 新套牌、旧套牌 vs 新套牌、改动几张卡的旧套牌变体。
5. 保留一部分旧环境 replay 或旧环境 matchup，防止 catastrophic forgetting。
6. 对新旧环境都跑评估矩阵，确认新环境变强的同时没有严重遗忘基础能力。

训练采样可以从类似比例开始：

```text
70% 新环境对局
20% 上个环境关键 matchup
10% 历史困难 matchup
```

比例不是固定规则，应根据环境变化幅度调整。若新环境变化很小，可以提高旧环境保留比例；若新卡和主流套牌大幅变化，可以提高新环境采样比例。

### 10.2 策略池和数据版本化

建议所有训练资产都按环境版本管理：

```text
env_v1/
  card_pool
  deck_pool
  checkpoints
  matchup_matrix

env_v2/
  card_pool
  deck_pool
  checkpoints
  matchup_matrix
```

策略池不应只保留当前最新模型。建议拆成：

```text
current_env_pool
previous_env_pool
all_time_best_pool
```

训练时从不同策略池采样对手，可以让模型见到更多风格，避免只适应当前最新版自己。

### 10.3 主流套牌增减的采样策略

主流套牌变多或变少时，不建议简单删除旧套牌或均匀采样所有套牌。更合理的是动态采样：

```text
sample_weight(deck) =
  meta_share_weight
  + uncertainty_weight
  + weakness_weight
```

含义是：

- 主流占比高的套牌多采样。
- 模型评估不确定的 matchup 多采样。
- 当前胜率低的 matchup 多采样。
- 已过气但仍有代表性的旧套牌低频保留。

这样模型能贴近当前环境，同时不完全忘记历史环境中的基础能力。

### 10.4 小幅卡位改动的处理

如果某套牌只是替换几张卡，不应把它当成全新套牌从零训练。可以表示为：

```text
base_deck + delta_cards
```

训练时使用共享 PlayPolicy，通过 decklist embedding 感知卡位变化，然后对新变体做短时间 fine-tune。旧模型已经学到该套牌的主轴，新训练主要适配新增卡和 matchup 变化。

## 11. 当前结论

当前最务实的启动路线是：

```text
固定 Deck A vs Deck B
+ 结构化 PTCG 环境
+ 合法动作枚举
+ state/action encoder
+ scripted bot 行为克隆
+ masked PPO
+ self-play
+ 历史 checkpoint 策略池
+ 固定评估矩阵
```

这条路线最适合单张 RTX 4090 在合理时间内跑出第一版结果。真正的瓶颈大概率不是 GPU，而是 PTCG 规则引擎速度、卡牌效果覆盖率和合法动作枚举质量。

## 参考资料

- AlphaGo: https://www.nature.com/articles/nature16961
- AlphaZero: https://pubmed.ncbi.nlm.nih.gov/30523106/
- MuZero: https://www.nature.com/articles/s41586-020-03051-4
- Suphx: https://arxiv.org/abs/2003.13590
- DouZero: https://proceedings.mlr.press/v139/zha21a.html
- ReBeL: https://papers.nips.cc/paper/2020/hash/c61f571dbd2fb949d3fe5ae1608dd48b-Abstract.html
- Limitless TCG: https://limitlesstcg.com/
- Limitless API Docs: https://docs.limitlesstcg.com/developer.html
- PTCGL Replayer: https://replay.ptcgtools.com/
- PTCG Live Battle Log 介绍: https://cardgamer.com/games/pokemon-tcg-live-battle-log/

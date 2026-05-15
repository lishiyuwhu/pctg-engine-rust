# PTCG RL Training

## 快速开始

```bash
# 1. 安装依赖
pip install sb3-contrib stable-baselines3 gymnasium numpy wandb tensorboard

# 2. 构建 Rust 扩展
cd ..
maturin develop --release -m crates/ptcg-py/Cargo.toml
cd RL_TRAIN

# 3. 启动训练
PYTHONPATH=../crates/ptcg-py/python python train.py
```

## 训练目标

| 项目 | 设定 |
|------|------|
| 阶段 | Phase 1: 固定套牌对战 |
| Agent (P0) | Miraidon ex |
| Opponent (P1) | Charizard ex / Pidgeot ex |
| 算法 | MaskablePPO (SB3 Contrib) |
| 目标 | Agent 稳定击败 scripted bot (>50% 胜率) |

## 命令

```bash
# 完整训练 (10M steps, wandb 在线)
python train.py

# 不用 wandb
python train.py --no-wandb

# wandb 离线模式 (日志存本地，稍后同步)
python train.py --wandb-offline

# 自定义 wandb run 名称和 tags
python train.py --wandb-name "lr-test-3e4" --wandb-tags lr-test v2

# 指定训练步数
python train.py --timesteps 1_000_000

# 评估已保存的模型
python train.py --eval-only checkpoints/ppo_best.zip

# 从 checkpoint 继续训练 (断点续训)
python train.py --resume checkpoints/ppo_final.zip

# 断点续训 + 继续记录到同一个 wandb run
python train.py --resume checkpoints/ppo_best.zip --resume-wandb-id <WANDB_RUN_ID>

# 续训到指定步数
python train.py --resume checkpoints/ppo_best.zip --timesteps 20_000_000
```

## 断点续训

每次 `model.learn()` 结束后，模型会自动保存到 `checkpoints/ppo_final.zip`。
可以随时关闭 WSL，下次用 `--resume` 继续训练:

```bash
python train.py --resume checkpoints/ppo_final.zip
```

需要继续记录到同一个 wandb run（保持曲线连续）:

```bash
python train.py --resume checkpoints/ppo_final.zip --resume-wandb-id <WANDB_RUN_ID>
```

## Wandb 配置

首次使用需要登录:

```bash
wandb login
```

或在 `config.py` 中修改:

```python
WANDB_CONFIG = {
    "project": "ptcg-rl",        # 项目名
    "entity": "your-username",   # wandb 用户名或 team
    "tags": ["phase1", ...],
}
```

## 记录到 Wandb 的指标

### 评估指标 (每 100K 步)

| 指标 | 说明 |
|------|------|
| `eval/win_rate` | Agent 胜率 |
| `eval/loss_rate` | Agent 败率 |
| `eval/draw_rate` | 平局率 |
| `eval/avg_turns` | 平均回合数 |
| `eval/total_steps` | 累计训练步数 |

### PPO 训练指标 (每 rollout)

| 指标 | 说明 |
|------|------|
| `rollout/ep_rew_mean` | 平均 episode 奖励 |
| `rollout/ep_len_mean` | 平均 episode 长度 |
| `train/entropy_loss` | 策略熵 (越高越随机) |
| `train/policy_gradient_loss` | 策略梯度损失 |
| `train/value_loss` | 价值函数损失 |
| `train/approx_kl` | KL 散度 (越大更新越激进) |
| `train/clip_fraction` | 被裁剪的动作比例 |
| `train/explained_variance` | 价值函数解释方差 |
| `time/fps` | 训练速度 (frames/sec) |

### 系统指标

| 指标 | 说明 |
|------|------|
| `system/elapsed_seconds` | 已用时间 |
| `system/steps_per_second` | 平均训练速度 |

## 超参数

在 `config.py` 中调整:

```python
PPO_CONFIG = {
    "learning_rate": 3e-4,    # 学习率
    "n_steps": 2048,          # 每次 rollout 步数
    "batch_size": 64,         # 批次大小
    "n_epochs": 10,           # 每轮更新次数
    "gamma": 0.99,            # 折扣因子
    "ent_coef": 0.01,         # 熵系数 (探索)
    "clip_range": 0.2,        # PPO clip 范围
}
```

## 文件结构

```
RL_TRAIN/
├── README.md          # 本文档
├── train.py           # 主训练脚本
├── evaluate.py        # 独立评估脚本
├── config.py          # 超参数配置
├── checkpoints/       # 模型保存 (gitignore)
└── logs/              # TensorBoard 日志 (gitignore)
```

## 评估基线 (参考)

| 对手 | 随机策略 | 目标 |
|------|----------|------|
| Random bot | ~50% | >80% |
| Charizard heuristic | ~35% | >50% |
| Charizard strategy bot | <20% | >50% |

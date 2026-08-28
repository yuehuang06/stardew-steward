# Stardew Steward — 星露谷农场管家

星露谷物语的专属 AI Agent，持续读取本地存档，结合游戏知识库给出"够用就好"的当日日程建议。

## 快速开始

```bash
# 1. 复制配置
cp config.toml.example config.toml
cp .env.example .env

# 2. 填入 API Key 和存档路径
# 编辑 config.toml 和 .env

# 3. 运行
cargo run
```

## 配置说明

| 字段 | 说明 |
|------|------|
| `model.endpoint` | LLM API 地址 |
| `model.api_key` | API 密钥（也可用 .env 的 API_KEY） |
| `save.path` | 存档文件路径（WSL 用 /mnt/c/...，Windows 用 C:\...） |
| `agent.token_budget` | Token 预算上限（超限自动中断） |

## 项目结构

```
stardew-steward/
├── Cargo.toml
├── config.toml.example
├── .env.example
├── src/
│   ├── main.rs              # 入口 + CLI
│   ├── config.rs            # 模型配置 (R3)
│   ├── agent/
│   │   ├── mod.rs
│   │   ├── agent_loop.rs    # Agent Loop: LLM ↔ 工具调用循环
│   │   └── message.rs       # 消息类型
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── save.rs          # XML 存档解析器
│   │   └── state.rs         # GameState 结构体
│   ├── knowledge/
│   │   └── mod.rs           # 知识库 (rusqlite 关键词检索)
│   ├── solver/
│   │   ├── mod.rs
│   │   ├── task.rs          # 任务/日程定义
│   │   └── greedy.rs        # 贪心求解器
│   ├── tools/
│   │   ├── mod.rs           # 工具注册
│   │   ├── read_save.rs     # 工具: 读存档
│   │   ├── query_knowledge.rs  # 工具: 查知识库
│   │   └── solve_schedule.rs  # 工具: 跑求解器
│   ├── validator/
│   │   └── mod.rs           # 校验闭环
│   ├── ui/
│   │   └── mod.rs           # CLI 界面
│   └── usage.rs             # Token 统计 (R6)
├── data/
│   ├── crops.json           # 作物知识
│   ├── npcs.json            # NPC 喜好
│   └── fish.json            # 鱼类约束
├── logs/                    # 每日战报输出目录
└── README.md
```

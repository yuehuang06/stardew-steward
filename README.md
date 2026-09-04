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

## 配置说明（config.toml）

```toml
[model]
endpoint = "https://api.deepseek.com/v1/chat/completions"
api_key = ""              # 也可用 .env 中的 API_KEY
model = "deepseek-chat"
context_length = 8192
thinking_mode = false
price_input = 0.0014      # CNY per 1K input tokens
price_output = 0.0028     # CNY per 1K output tokens

[save]
path = "/mnt/c/Users/.../Saves/存档名/存档名"   # WSL 用 /mnt/c/...，Windows 用 C:\...

[agent]
token_budget = 50000      # Token 预算上限（超限自动中断）
max_steps = 10           # Agent Loop 最大步数

[knowledge]
db_path = "data/knowledge.db"
```

| 字段 | 说明 |
|------|------|
| `model.endpoint` | LLM API 地址 |
| `model.api_key` | API 密钥（也可用 .env 的 API_KEY） |
| `model.model` | 模型名 |
| `model.context_length` | 上下文窗口长度 |
| `model.thinking_mode` | 是否启用思考模式 |
| `model.price_input/output` | 输入/输出 token 单价（用于成本统计） |
| `save.path` | 存档文件路径 |
| `agent.token_budget` | Token 预算上限（超限自动中断） |
| `agent.max_steps` | Agent Loop 最大步数 |
| `knowledge.db_path` | SQLite 知识库路径 |

## CLI 用法

```bash
# 交互模式（默认）
cargo run

# 直接提问（不进入交互模式）
cargo run -- --ask "今天该干嘛"

# 指定存档路径（覆盖 config.toml）
cargo run -- --save "/path/to/save"

# 只解析存档并打印结构化 JSON（不调 LLM）
cargo run -- --parse

# 打印 Agent 视角的压缩存档摘要（不调 LLM）
cargo run -- --status
```

### 交互模式命令

| 命令 | 说明 |
|------|------|
| `/help` | 查看可用命令 |
| `/save <名称>` | 保存当前会话 |
| `/load <名称>` | 加载已保存的会话 |
| `/sessions` | 列出所有已保存会话 |
| `/log` | 查看 Agent 完整工作轨迹 |
| `/usage` | 查看 Token 用量与成本 |
| `/quit` | 退出 |

## 项目结构

```
stardew-steward/
├── Cargo.toml
├── config.toml.example
├── .env.example
├── src/
│   ├── main.rs              # 入口: CLI 参数解析 + 工具注册 + 交互循环
│   ├── config.rs            # 配置加载 (config.toml + .env)
│   │
│   ├── agent.rs             # Agent 模块入口
│   ├── agent/
│   │   ├── agent_loop.rs    # Agent Loop: LLM ↔ 工具调用循环 + 打断
│   │   ├── message.rs       # 消息类型 (system/user/assistant/tool)
│   │   ├── progress.rs      # ProgressReporter trait (进度回调)
│   │   └── session.rs       # 会话持久化 (save/load/list)
│   │
│   ├── parser.rs            # 存档解析器模块入口
│   ├── parser/
│   │   ├── save.rs          # roxmltree XML → GameState
│   │   └── state.rs         # GameState 结构体定义
│   │
│   ├── knowledge.rs         # 知识库 (rusqlite 关键词检索 + 种子数据导入)
│   │
│   ├── tools.rs             # ToolRegistry (异步 handler 注册/执行)
│   ├── tools/
│   │   ├── read_save.rs     # 工具: 读存档 → 压缩 JSON
│   │   ├── query_knowledge.rs # 工具: 查知识库
│   │   ├── solve_schedule.rs  # 工具: 手动求解日程
│   │   └── fetch_wiki.rs      # 工具: MediaWiki API 在线搜索
│   │
│   ├── solver.rs            # 求解器模块入口
│   ├── solver/
│   │   ├── task.rs          # Task/Schedule/Priority 定义
│   │   ├── greedy.rs        # 贪心算法
│   │   ├── auto_tasks.rs    # 自动任务生成 (吃 GameState + 知识库)
│   │   └── schedule.rs      # DailySchedule JSON + Markdown 渲染
│   │
│   ├── validator.rs         # 校验闭环 (资金/时长/优先级规则)
│   ├── usage.rs             # Token 用量与成本统计
│   └── ui.rs                 # CLI 界面 + CliReporter
│
├── data/
│   ├── crops.json           # 25 种作物知识
│   ├── npcs.json            # 29 位 NPC 喜好
│   ├── fish.json            # 26 种鱼类约束
│   ├── knowledge.db         # SQLite 知识库 (首次启动自动生成)
│   └── saves/               # 真实存档副本 (开发用, .gitignore)
│
├── docs/
│   ├── 设计文档.md
│   └── AI开发开销明细.md
├── sessions/                # 保存的会话 (.gitignore)
├── logs/                    # 日志输出目录
└── README.md
```

## Agent 工具

| 工具 | 说明 |
|------|------|
| `read_save` | 读取本地存档，返回压缩后的游戏状态 JSON (资金/日期/天气/作物/好感度/背包/木箱) |
| `query_knowledge` | 搜索本地知识库 (作物收益/NPC喜好/鱼类约束) |
| `fetch_wiki` | 从星露谷中文 wiki 在线搜索补充本地知识库没有的数据 |
| `solve_schedule` | 接收任务列表，调用贪心求解器输出最优日程 |
| `auto_schedule` | 自动分析存档 + 知识库，一步到位生成最优日程 JSON |

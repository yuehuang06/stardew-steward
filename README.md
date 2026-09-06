# Stardew Steward — 星露谷农场管家

星露谷物语的专属 AI Agent，持续读取本地存档，结合游戏知识库给出"够用就好"的当日日程建议。

## 快速开始

### 方式一：下载源码编译运行

需要安装 [Rust](https://rustup.rs/) 和 [Node.js](https://nodejs.org/)（≥18）。

```bash
# 1. 克隆仓库
git clone <仓库地址>
cd stardew-steward

# 2. 安装前端依赖
cd frontend && npm install && cd ..

# 3a. 运行 GUI（开发模式）—— 二选一
cd src-tauri && tauri dev
# 或安装了 cargo-tauri 的话也可以：
# cd src-tauri && cargo tauri dev

# 3b. 运行 CLI —— 二选一
cargo run
```

首次运行时自动在应用数据目录生成 `config.toml`（API key 为空）。
通过 GUI 设置面板填入 endpoint 和 API key 即可，也可手动编辑 config.toml。

> **关于 `tauri dev` vs `cargo tauri dev`：** 两者完全等价。`tauri` 是通过 `npm install -g @tauri-apps/cli` 安装的，`cargo tauri` 是通过 `cargo install tauri-cli` 安装的。装了哪个就用哪个。

### 方式二：下载安装包

安装 MSI/NSIS 包后直接打开应用，无需任何开发环境。首次运行自动生成配置文件，在设置面板填入 API key 即可。

### 打包

```bash
# 需要安装 tauri CLI（npm 或 cargo 均可）
cd src-tauri && tauri build
```

## 配置文件位置

首次运行时在应用数据目录自动生成：

| 系统 | 路径 |
|------|------|
| Windows | `%APPDATA%\stardew-steward\config.toml` |
| macOS | `~/Library/Application Support/stardew-steward/config.toml` |
| Linux | `~/.config/stardew-steward/config.toml` |

同目录下还会生成 `config.toml.example`（带注释的参考模板）。

**两种配置方式：**
1. **GUI 设置面板**（推荐普通用户）— 标题栏 ⚙ → 填 endpoint/key/model/context_length/thinking_mode/价格/Token预算 → 保存
2. **手动编辑 config.toml**（高级用户）— 在 OpenAI / 本地模型 / 清华 AI 平台之间切换 endpoint 和 model

## 配置说明（config.toml）

```toml
[model]
endpoint = "https://lab.cs.tsinghua.edu.cn/ai-platform/api/v1"
api_key = ""              # 也可用 .env 中的 API_KEY
model = "glm-5.2"
context_length = 8192
thinking_mode = false
price_input = 0.0014      # CNY per 1K input tokens
price_output = 0.0028     # CNY per 1K output tokens

[save]
path = ""                 # 留空则自动检测星露谷存档

[agent]
token_budget = 200000    # 每会话 Token 预算（超限自动中断）
max_steps = 10           # Agent Loop 最大步数

[knowledge]
db_path = "knowledge.db"
```

| 字段 | 说明 |
|------|------|
| `model.endpoint` | LLM API 地址（OpenAI 兼容） |
| `model.api_key` | API 密钥（也可用 .env 的 API_KEY） |
| `model.model` | 模型名 |
| `model.context_length` | 上下文窗口长度 |
| `model.thinking_mode` | 是否启用思考模式 |
| `model.price_input/output` | 输入/输出 token 单价（用于成本统计） |
| `save.path` | 存档文件路径（留空自动检测） |
| `agent.token_budget` | 每会话 Token 预算上限 |
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

## GUI 用法

启动 GUI 后，窗口以像素风悬浮在游戏窗口旁边（默认 280px 窄模式，可伸缩到 480px）。

| 功能 | 说明 |
|------|------|
| 聊天 | 输入框发送消息，Agent 回复支持 Markdown 渲染 + 打字机效果 |
| 日程时间线 | Agent 输出日程 JSON 时自动渲染为竖向时间线（图标+标签+进度条） |
| 像素进度条 | 底部像素方格进度条，每步填充一格 |
| 进度步骤 | 实时显示 Agent 当前在做什么（读存档/查知识库/生成日程） |
| 停止按钮 | 打断当前 Agent 任务 |
| 会话面板 | 新建/加载/删除会话，自动保存 |
| 设置面板 | 在线修改 endpoint/key/model/context_length/thinking_mode/价格/Token预算 |
| 用量统计 | 右下角实时显示 ¥成本 + per-session token 用量 |
| 窗口置顶 | 标题栏按钮切换，游戏时悬浮在上方 |
| 窗口伸缩 | 标题栏按钮在 280px 和 480px 间切换 |
| 刷新 | 标题栏 ↻ 按钮重新读取存档状态 |

## Agent 工具（8 个）

| 工具 | 类型 | 说明 |
|------|------|------|
| `read_save` | 读存档 | 解析 XML 存档，返回压缩的游戏状态 JSON |
| `query_knowledge` | 查知识库 | 搜索 SQLite 知识库（作物/NPC/鱼类） |
| `fetch_wiki` | 在线查 | 从星露谷中文 wiki 搜索并提取结构化信息 |
| `auto_schedule` | 生成日程 | 自动分析存档+知识库+求解器，一步到位生成日程 |
| `solve_schedule` | 手动求解 | 接收任务列表，调用贪心求解器输出最优日程 |
| `crop_advisor` | 收益分析 | 对比当前作物与当季全部作物的 g/天，推荐最优种植方案 |
| `gift_finder` | 送礼推荐 | 扫描背包+木箱匹配 NPC 喜好，推荐送礼方案 |
| `farm_hand` | 写存档 | 托管农场操作：浇水/收获/清理枯死/回档前一天（自动备份+校验） |

## 项目结构

```
stardew-steward/
├── Cargo.toml              # workspace [lib]+[[bin]]+workspace(含 src-tauri)
├── config.toml.example
├── .env.example
├── icon.png                # 应用图标源文件
├── src/                     # 核心库 (CLI + GUI 共用)
│   ├── lib.rs / main.rs / app.rs / config.rs
│   ├── agent/               # Agent Loop + 消息 + 进度 + 会话
│   ├── parser/              # roxmltree XML → GameState
│   ├── knowledge.rs         # SQLite 知识库
│   ├── tools.rs + tools/    # 8 个工具 (read_save, farm_hand, crop_advisor, ...)
│   ├── solver/              # 贪心 + auto_tasks + schedule
│   ├── validator.rs         # 校验闭环
│   ├── usage.rs             # per-session Token 统计
│   └── ui.rs                 # CLI 界面
├── src-tauri/                # Tauri 2 GUI 后端 (16 个命令)
│   └── src/main.rs + commands.rs + reporter.rs
├── frontend/                # React 18 + Vite 5 前端
│   └── src/components/ + hooks/ + styles/
├── data/                     # 知识库种子 + 内置 Ferris 存档
│   ├── crops.json / npcs.json / fish.json
│   └── saves/Rust + SaveGameInfo
├── sessions/                # 保存的会话 (.gitignore)
└── docs/                    # 设计文档 + 摘要 + AI 开销明细
```

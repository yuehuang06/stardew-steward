# Stardew Steward — 星露谷农场管家

星露谷物语的专属 AI Agent，持续读取本地存档，结合游戏知识库给出"够用就好"的当日日程建议。

## 平台支持

| 模式 | Windows | macOS | Linux |
|------|---------|-------|-------|
| CLI（终端交互） | ✅ | ✅ | ✅ |
| GUI（Tauri 桌面窗口） | ✅ | ⚠️ 需自行编译 | ⚠️ 需自行编译 |

- **CLI** 纯 Rust，跨平台，`cargo run` 即可运行
- **GUI** 基于 Tauri 2，已提供 Windows 预编译 exe；macOS/Linux 需从源码自行 `tauri build`

## 快速开始

### 方式一：下载源码编译运行

需要安装 [Rust](https://rustup.rs/)。

**CLI（所有平台）：**
```bash
git clone <仓库地址>
cd stardew-steward
cargo run
```

**GUI（需额外安装 Node.js ≥18 + Tauri CLI）：**
```bash
# 安装 Tauri CLI（二选一）
npm install -g @tauri-apps/cli   # 或
cargo install tauri-cli

# 安装前端依赖 + 运行
cd frontend && npm install && cd ..
cd src-tauri && tauri dev
```

首次运行时自动在应用数据目录生成 `config.toml`（API key 为空）。
通过 GUI 设置面板填入 endpoint 和 API key 即可，也可手动编辑 config.toml。

> **关于 `tauri dev` vs `cargo tauri dev`：** 两者完全等价。`tauri` 是通过 `npm install -g @tauri-apps/cli` 安装的，`cargo tauri` 是通过 `cargo install tauri-cli` 安装的。装了哪个就用哪个。

### 方式二：下载预编译包（仅 Windows，免编译）

仓库 `dist/` 目录下提供预编译产物：

| 文件 | 大小 | 说明 |
|------|------|------|
| `dist/stardew-steward-gui.exe` | 18MB | 独立 exe，双击即用 |
| `dist/Stardew Steward_0.1.0_x64-setup.exe` | 4MB | NSIS 安装包，安装后开始菜单有快捷方式 |

下载后直接运行即可，无需 Rust/Node.js 环境。首次打开自动生成配置文件，在设置面板填入 API key 即可。

### 打包

```bash
# Windows/macOS/Linux 均可，需 Tauri CLI
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

> **WSL 用户**：CLI 在 Linux 侧没有配置文件时，会自动复用 Windows 侧（GUI 设置面板写入的）同一份配置，`C:\...` 存档路径也会自动翻译成 `/mnt/c/...`——GUI 里改配置，CLI 直接生效。

**两种配置方式：**
1. **GUI 设置面板**（推荐普通用户）— 标题栏 ⚙ → 填 endpoint/key/model/context_length/thinking_mode/价格/Token预算 → 保存
2. **手动编辑 config.toml**（高级用户）— 在 OpenAI / 本地模型 / 其他平台之间切换 endpoint 和 model

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

## 演示用例

打开应用后，依次输入以下问句即可完整体验全部核心能力（每条例子标注了触发的工具和预期效果）：

### ① 安排今日日程（核心场景）

```
你：今天该干嘛？
```

- 触发：`read_save` → `auto_schedule`
- 预期：顶部进度条依次显示"正在读取存档 → 正在自动生成日程"，随后渲染**竖向时间线日程卡**——按必做/建议/可选列出收获、浇水、送礼、下矿等任务，底部汇总预计收入与耗时
- 验证点：日程内容与你存档实际状态一致（如"浇水 N 株"中的 N 未计入洒水器覆盖的作物）

### ② 托管农场（通用 Agent 做不到）

```
你：帮我浇水
（关闭游戏后）
你：帮我浇水
```

- 触发：`farm_hand(action="water_all")`
- 预期：游戏运行中会拒绝并提示先关游戏；关闭后显示"农场助手正在浇水"，报告"N 块耕地已浇 + 备份路径"；打开游戏加载存档，作物全部湿润
- 验证点：`.bak` 备份存在；进游戏后土地确实已浇

### ③ 作物收益分析

```
你：第一年秋天种什么最赚？
```

- 触发：`read_save` → `crop_advisor`
- 预期：列出当季作物 g/天 排行（南瓜/蔓越莓/玉米…），结合你的剩余天数和资金给出补种建议——确定性计算，不是模型口算

### ④ 送礼推荐

```
你：该给谁送礼物？
```

- 触发：`read_save` → `gift_finder`
- 预期：扫描背包+木箱，与 29 位 NPC 的最爱/喜欢清单交叉匹配，标出"送谁、送什么、在你背包还是木箱"

### ⑤ 回档反悔

```
你：我在矿井晕倒了，想回到昨天
```

- 触发：`farm_hand(action="rollback_day")`
- 预期：用游戏自带 `_old` 存档回退一天，显示回档前后的日期/资金对比；当天进度另存 `.bak` 可再恢复

### ⑥ 知识问答

```
你：阿比盖尔喜欢什么礼物？
你：鲶鱼什么季节能钓到？
```

- 触发：`query_knowledge`（查不到时 `fetch_wiki`）
- 预期：SQLite 知识库秒回结构化数据；NPC 一律官方中文名

> 提示：演示前先点标题栏 ↻ 刷新存档状态；右下角实时显示每一步的 token 消耗。

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

# Stardew Steward 开发进度

> 最后更新: 2026-09-05
> 项目路径:
> - WSL: `/home/gnauh/coding/2026aug/大作业/stardew-steward`
> - Windows: `C:\Users\gnauh\stardew-steward-win` (克隆副本, 用于 GUI 调试)
> - 开发方式: WSL 编辑代码 → git commit → Windows `git pull && tauri dev` 运行

---

## 一、项目现状

### 已完成

**阶段 1: Rust 核心 (CLI Agent)**
- 存档解析器 (`src/parser/`) — roxmltree 解析 XML → GameState
- 知识库 (`src/knowledge.rs`) — rusqlite, 25 种作物 / 29 位 NPC / 26 种鱼
- Agent Loop (`src/agent/`) — LLM ↔ 工具调用循环, Ctrl-C 打断, 会话持久化
- 求解器 (`src/solver/`) — 贪心算法, 自动任务生成, DailySchedule JSON
- 校验器 (`src/validator.rs`) — 资金/时长/优先级规则校验
- 工具注册 (`src/tools.rs` + `src/app.rs`) — 5 个工具: read_save, query_knowledge, fetch_wiki, solve_schedule, auto_schedule
- Token 统计 (`src/usage.rs`) — 成本追踪, 预算自动中断
- CLI 交互 (`src/main.rs` + `src/ui.rs`) — rustyline 行编辑, 斜杠命令

**阶段 2: 库抽象 (为 GUI 准备)**
- `src/lib.rs` — 导出所有 pub mod, 核心代码变成可复用库
- `src/app.rs` — `build_tools()` / `build_system_prompt()` 公共函数
- `Cargo.toml` — `[lib]` + `[[bin]]` + workspace (含 `src-tauri`)
- ProgressReporter trait 加 `on_thinking()` / `on_error()`

**阶段 3: Tauri GUI 客户端**
- `src-tauri/` — Tauri 2 后端, 16 个命令
- TauriReporter — emit 事件 (agent-step/thinking/error/done)
- React 18 + Vite 5 前端
- 窗口: 透明/无边框/始终置顶/可伸缩 (280px→480px)

**阶段 4: GUI 功能完善 (9/4~9/5)**
- 日程卡片 (ScheduleCard) — JSON → 像素风卡片, 优先级中文标签 (必做/建议/可选)
- 打字机效果 — 逐字显示, 18ms/字, 闪烁光标, interrupt 即停
- 进度步骤 — 最近 2 条, 最后一条闪烁, 回复出现时清空
- Markdown 渲染 — react-markdown + remark-gfm (表格/加粗/列表)
- 会话管理 — 自动保存, 加载历史, 删除历史, 新建对话
- 设置面板 — API 配置 (endpoint/key/model/context_length/thinking_mode/价格), 用量统计, 持久化 config.toml
- 双层用量 — per-session + global token/cost 统计
- NPC 中文名 — 系统提示词含 28 人中英对照表
- 像素木边框 — 9-slice PNG border-image, 三色 3px, 斜角切边
- 聊天气泡/输入框 — 9-slice 边框, 左上右下透明
- 窗口拖拽 — `getCurrentWindow().startDragging()` (data-tauri-drag-region 在 Windows 不可靠)
- DPI 感知伸缩 — LogicalSize 替代 PhysicalSize
- 中文字体 — Noto Sans SC (Google Fonts), WSL 无 CJK 字体
- 中文输入 — uncontrolled input + `e.isComposing`

### R1-R6 完成度对照

| 要求 | 状态 | 说明 |
|------|------|------|
| R1 核心 Rust | ✅ | 全部核心逻辑在 Rust, CLI + GUI 共用库 |
| R2 用户界面 | ✅ | CLI (rustyline) + GUI (Tauri 2 + React) |
| R3 可配置模型 | ✅ | config.toml + 设置面板 (endpoint/api_key/model/context_length/thinking_mode/价格) |
| R4 实时进度+打断 | ✅ | 进度闪烁 + 停止按钮 + Ctrl-C (CLI) |
| R5 历史管理 | ✅ | 自动保存/加载/删除, 轨迹导出, 会话级用量 |
| R6 Token 统计 | ✅ | per-session + global, 预算自动中断 |

### 待办 (优先级排序)

1. **[高] 接入 context_length 和 thinking_mode** — 配置项已在设置面板, 但 `call_llm()` 未实际使用这两个字段
2. **[高] 图标替换** — 当前 `src-tauri/icons/` 是 tauri 自动生成的占位图, 需要像素风图标
3. **[中] 知识库数据补全** — crops/npcs/fish 数据仍需扩充
4. **[中] 设计文档更新** — 需与最终实现对齐, 补充 GUI 架构、设置面板等内容
5. **[低] 存档监听** — notify crate 监听存档变化, 自动刷新
6. **[低] 存档 diff** — 睡觉后存档变化, 自动生成战报

---

## 二、编译状态

### 当前: 无编译错误

```bash
cargo build                          # ✅ CLI (2 warnings)
cargo build -p stardew-steward-gui   # ✅ Tauri 后端 (3 lib warnings)
cd frontend && npm run build         # ✅ (~335KB JS, 4.6KB CSS)
```

### Warnings (均为无害)
- `unused import: crate::parser::GameState` (read_save.rs)
- `field system_prompt is never read` (agent_loop.rs) — 实际被 new_session 使用
- `field season is never read` (validator.rs)
- `field money is never read` (validator.rs)

---

## 三、技术栈和目录结构

### 技术栈

| 层 | 技术 | 说明 |
|----|------|------|
| 核心库 | Rust 2024 edition | roxmltree / reqwest / rusqlite / serde / tokio |
| CLI | clap + rustyline | `cargo run` 交互式终端 |
| GUI 后端 | Tauri 2 | Rust, 复用核心库, 16 个命令 |
| GUI 前端 | React 18 + Vite 5 | invoke() + listen() 通信 |
| 字体 | Noto Sans SC + Pixelify Sans | Google Fonts CDN |
| 边框 | 9-slice PNG (base64 data URI) | Python 生成, border-image |
| 知识库 | SQLite (rusqlite bundled) | 首次从 data/*.json 导入 |

### 目录结构

```
stardew-steward/
├── Cargo.toml              # workspace 根 [lib]+[[bin]]+workspace
├── config.toml             # 运行时配置 (gitignored)
├── PROGRESS.md             # 本文件
│
├── src/                     # 核心库 (CLI + GUI 共用)
│   ├── lib.rs               # 库入口 pub mod
│   ├── main.rs              # CLI 入口
│   ├── app.rs               # build_tools() + build_system_prompt()
│   ├── config.rs            # Config (Serialize+Deserialize) + load() + save()
│   ├── agent/
│   │   ├── agent_loop.rs    # Agent::run() + new_session() + update_model_config()
│   │   ├── message.rs / progress.rs / session.rs  # SessionUsage 结构体
│   ├── parser/              # save.rs + state.rs
│   ├── knowledge.rs         # SQLite 知识库
│   ├── tools/               # read_save / query_knowledge / solve_schedule / fetch_wiki / auto_schedule
│   ├── solver/              # task / greedy / auto_tasks / schedule
│   ├── validator.rs / usage.rs / ui.rs
│
├── src-tauri/               # Tauri 2 GUI 后端
│   ├── Cargo.toml           # 依赖 stardew-steward = { path = ".." }
│   ├── tauri.conf.json      # 窗口配置
│   ├── capabilities/default.json  # 权限
│   ├── icons/               # PNG + ICO
│   └── src/
│       ├── main.rs          # setup() 初始化 Agent + manage(AppState)
│       ├── reporter.rs      # TauriReporter (emit 事件)
│       └── commands.rs      # 16 个命令 (chat/get_config/update_config/get_usage_detail/new_session/delete_session/...)
│
├── frontend/               # React 前端
│   ├── package.json         # react, react-markdown, remark-gfm, @tauri-apps/api
│   └── src/
│       ├── main.jsx / App.jsx
│       ├── styles/theme.css  # CSS 变量 + 9-slice border-image + 像素木框
│       ├── components/       # Titlebar / ChatMessage / ProgressIndicator / InputBar / ScheduleCard / SessionPanel / SettingsPanel / Markdown
│       ├── hooks/           # useChat / useWindowState / useSessions / useAlwaysOnTop / useSettings
│       └── utils/          # parseSchedule
│
├── data/                   # 知识库种子数据
├── sessions/               # 自动保存的会话 JSON
└── docs/                   # 设计文档 + AI开发开销明细
```

### 关键设计

- **库/二进制分离**: CLI 和 Tauri 共用 `stardew_steward` 库
- **ProgressReporter trait**: CLI 用 println, Tauri 用 emit, agent_loop 不直接 IO
- **9-slice border-image**: PNG 用 Python 脚本生成, base64 内嵌 CSS
- **会话用量**: SavedSession 含 SessionUsage, 保存/加载时携带
- **窗口拖拽**: `onMouseDown` → `getCurrentWindow().startDragging()`

---

## 四、下一步具体 TODO

### TODO 1: 接入 context_length 和 thinking_mode

**问题**: 设置面板已有这两个配置项, 但 `Agent::call_llm()` 未读取它们。

**位置**: `src/agent/agent_loop.rs` 的 `call_llm()` 方法

**做法**:
- `context_length`: 在构建 LLM 请求时, 取 `history` 最后 N 条消息 (N * 平均长度 ≤ context_length), 截断旧消息
- `thinking_mode`: 如果为 true, 在请求参数中启用模型的思考模式 (取决于具体 API, 如 GLM 的 `thinking` 参数)

### TODO 2: 图标替换

**问题**: `src-tauri/icons/` 当前是 Tauri 自动生成的纯色占位图。

**做法**:
- 画或找一张 32×32 像素风图标 (星露谷风格, 如锄头/作物/谷仓)
- 用 `tauri icon <path>` 生成全套 (32/128/128@2x/ico/icns)
- 替换 `src-tauri/icons/` 下的文件

### TODO 3: 知识库数据补全

**问题**: data/crops.json 只有 25 种作物, data/npcs.json 29 人, data/fish.json 26 种。

**做法**:
- 从星露谷中文 wiki 补充缺失的作物/NPC/鱼/物品数据
- 或者用 `fetch_wiki` 工具批量抓取并缓存到 knowledge.db

### TODO 4: 设计文档更新

**问题**: `docs/设计文档.md` 未与最终实现对齐, 缺少 GUI 架构、设置面板、会话管理等内容。

**做法**:
- 更新系统架构图 (加入 Tauri 层和 React 层)
- 补充场景定制方案 (像素 UI、日程卡片、知识库+wiki 双层检索)
- 更新功能清单 (标注已完成/待办)
- 更新技术选型 (Tauri 2, React 18, 9-slice border-image)

---

## 五、常用命令

```bash
# CLI 运行
cargo run
cargo run -- --status
cargo run -- --ask "今天该干嘛"

# GUI 开发 (Windows PowerShell)
cd C:\Users\gnauh\stardew-steward-win
tauri dev

# 同步: WSL → Windows
# (WSL 侧) git push
# (Windows 侧) git pull

# 编译检查
cargo build                          # CLI + 库
cargo build -p stardew-steward-gui   # Tauri 后端
cd frontend && npm run build         # 前端

# 生成 border-image PNG (修改边框颜色时用)
python3 -c "..."  # 见 git history, 9-slice 27x27 PNG
```

## 六、AI 开发开销

| 阶段 | 人时 | 花费 |
|------|------|------|
| 需求分析 | ~3h | ~$2 |
| 架构设计 | ~2h | ~$1 |
| Rust 核心逻辑 | ~8h | ~$12 |
| 完善工具 | ~3h | ~$18 |
| 搭 GUI 框架 | ~4h | ~$6 |
| GUI 功能开发与 UI 打磨 | ~7h | ~$24 |
| **总计** | **~26h** | **~$69** |

模型: glm-5.2 (清华 AI 平台, 套餐制)
工具: opencode

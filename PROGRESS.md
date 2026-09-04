# Stardew Steward 开发进度

> 最后更新: 2026-09-05

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
- CLI 交互 (`src/main.rs` + `src/ui.rs`) — rustyline 行编辑, 斜杠命令 (/save /load /sessions /log /usage)

**阶段 2: 库抽象 (为 GUI 准备)**
- `src/lib.rs` — 导出所有 pub mod, 核心代码变成可复用库
- `src/app.rs` — 工具注册和系统提示词构建抽成公共函数 `build_tools()` / `build_system_prompt()`
- `Cargo.toml` — 加 `[lib]` + `[[bin]]` + workspace (含 `src-tauri` 成员)
- `src/agent/progress.rs` — ProgressReporter trait 加 `on_thinking()` / `on_error()` 默认方法
- `src/agent/agent_loop.rs` — 3 处 `println!` 替换成 reporter 调用
- CLI (`cargo run`) 和库编译互不影响, 行为完全不变

**阶段 3: Tauri GUI 客户端**
- `src-tauri/` — Tauri 2 后端, 依赖 stardew_steward 库
- `src-tauri/src/reporter.rs` — TauriReporter, 把进度推送为前端事件
- `src-tauri/src/commands.rs` — 13 个 Tauri 命令 (chat/interrupt/get_save_status/save_session/load_session/list_sessions/get_messages/toggle_always_on_top 等)
- `src-tauri/src/main.rs` — 初始化 Agent + 注册命令 + 窗口配置
- `src-tauri/tauri.conf.json` — 窗口: 透明/无边框/始终置顶/可伸缩 (280px→480px)
- `frontend/` — React 18 + Vite 5
- `frontend/src/styles/theme.css` — 星露谷配色 (木框/羊皮纸) + Pixelify Sans 像素字体
- `frontend/src/App.jsx` — 侧边栏布局: 标题栏 + 聊天区 + 输入框 + 会话面板
- `frontend/src/components/` — Titlebar / ChatMessage / ProgressIndicator / InputBar / ScheduleCard / SessionPanel
- `frontend/src/hooks/` — useChat (发消息/收事件/加载会话) / useWindowState (窗口伸缩/存档状态/用量) / useSessions / useAlwaysOnTop
- `frontend/src/utils/` — parseSchedule (从消息文本提取日程 JSON)
- 后端编译通过, 前端构建通过

**阶段 4: GUI 功能完善**
- 日程卡片渲染 — ScheduleCard 组件: Agent 返回的 DailySchedule JSON 渲染成星露谷风格卡片 (任务行/动作图标/耗时花费收入徽章/优先级颜色/总计栏/提醒)
- 会话管理 UI — SessionPanel 组件: 保存当前会话/加载历史会话, 显示时间戳和消息数
- 始终置顶开关 — Titlebar 图钉按钮, toggle_always_on_top 命令
- 窗口伸缩动画 — toggle_window_width 改为 15 步 ease-out cubic 插值动画 (~180ms)

### 进行中

- 无

### 待办

- [ ] `cargo tauri dev` 实际运行测试 — 需要图形环境
- [ ] 图标替换 — 当前是纯色占位 PNG, 需要像素风图标
- [ ] 知识库数据补全 — crops/npcs/fish 数据仍需扩充

---

## 二、编译状态

### 当前: 无编译错误

```
cargo build              → ✅ (CLI, 4 warnings)
cargo build -p stardew-steward-gui → ✅ (Tauri 后端, 复用库 warnings)
npm run build (frontend) → ✅ (40 modules, 148KB JS)
```

### Warnings (均为已有, 非本次引入)
- `unused import: crate::parser::GameState` (read_save.rs)
- `unused variable: era` (session.rs)
- `field system_prompt is never read` (agent_loop.rs)
- `field season is never read` (validator.rs)

### 历史编译问题及修复 (已解决)
- `Config` 未实现 `Clone` → 全部 config 结构体加 `#[derive(Clone)]`
- `app.manage()` 找不到方法 → `use tauri::Manager`
- `useChat.js` 中 `useState` 导入在文件末尾 → 移到顶部

---

## 三、技术栈和目录结构

### 技术栈

| 层 | 技术 | 说明 |
|----|------|------|
| 核心库 | Rust 2024 edition | roxmltree / reqwest / rusqlite / serde / tokio |
| CLI | clap + rustyline | `cargo run` 交互式终端 |
| GUI 后端 | Tauri 2 | Rust 后端, 复用核心库, 11 个命令 |
| GUI 前端 | React 18 + Vite 5 | 单页应用, 调 invoke() + listen() |
| 字体 | Pixelify Sans (Google Fonts) | 像素字体, 接近星露谷本体风格 |
| 知识库 | SQLite (rusqlite bundled) | 首次启动从 data/*.json 导入 |

### 目录结构

```
stardew-steward/
├── Cargo.toml              # workspace 根, 含 [lib] + [[bin]] + workspace
│
├── src/                     # 核心库 (CLI + GUI 共用)
│   ├── lib.rs               # 库入口, 导出 pub mod (含 app)
│   ├── main.rs              # CLI 二进制入口, use stardew_steward::*
│   ├── app.rs               # build_tools() + build_system_prompt() 公共函数
│   ├── config.rs            # 配置加载 (全部 #[derive(Clone)])
│   ├── agent/
│   │   ├── agent_loop.rs    # Agent::run() 主循环 (println→reporter)
│   │   ├── message.rs       # 消息类型
│   │   ├── progress.rs      # ProgressReporter trait (+on_thinking/on_error)
│   │   └── session.rs       # 会话持久化
│   ├── parser/
│   │   ├── save.rs          # XML → GameState
│   │   └── state.rs         # GameState 结构体
│   ├── knowledge.rs         # SQLite 知识库
│   ├── tools/
│   │   ├── read_save.rs     # 读存档 → 压缩 JSON
│   │   ├── query_knowledge.rs
│   │   ├── solve_schedule.rs
│   │   └── fetch_wiki.rs    # MediaWiki API
│   ├── solver/
│   │   ├── task.rs / greedy.rs / auto_tasks.rs / schedule.rs
│   ├── validator.rs / usage.rs / ui.rs
│
├── src-tauri/               # Tauri GUI 后端
│   ├── Cargo.toml           # 依赖 stardew-steward = { path = ".." }
│   ├── tauri.conf.json      # 窗口: 透明/无边框/置顶/280→480px
│   ├── capabilities/default.json
│   ├── icons/               # 占位 PNG (需替换)
│   └── src/
│       ├── main.rs          # setup() 初始化 Agent + manage(AppState)
│       ├── reporter.rs      # TauriReporter (emit 事件)
│       └── commands.rs     # 11 个 #[tauri::command]
│
├── frontend/               # React 前端
│   ├── package.json         # @tauri-apps/api, react, vite
│   ├── vite.config.js       # port 1420
│   └── src/
│       ├── main.jsx / App.jsx
│       ├── styles/theme.css  # 星露谷配色 + 像素字体
│       ├── components/       # Titlebar / ChatMessage / ProgressIndicator / InputBar
│       └── hooks/           # useChat / useWindowState
│
├── data/                   # 知识库种子数据 (crops/npcs/fish.json + knowledge.db)
├── sessions/               # 保存的会话
├── docs/                   # 设计文档
└── config.toml             # 运行时配置
```

### 关键设计

- **库/二进制分离**: `src/lib.rs` 导出 pub mod, `src/main.rs` 用 `use stardew_steward::*` 引用. CLI 和 Tauri 共用同一份代码
- **ProgressReporter trait**: CLI 用 println, Tauri 用 emit. agent_loop 不直接调 println
- **app.rs 公共函数**: 工具注册和系统提示词只写一遍, CLI 和 Tauri 都调用
- **窗口配置**: decorations=false + transparent=true + alwaysOnTop=true, 窄条侧边栏

---

## 四、下一步

### 立即可做 (无依赖)

1. **`cargo tauri dev` 运行测试** — 需要图形环境 (WSL 需要 WSLg 或 X server)
   ```bash
   cargo tauri dev
   ```
   预期问题: WSL 下透明窗口可能黑底, 需要 `WSLg` 支持

2. **图标** — 用像素画工具画 32x32 / 128x128 星露谷风图标

### 长期

3. **知识库补全** — 扩充 data/*.json, 或用 fetch_wiki 缓存更多数据
4. **存档监听** — notify crate 监听存档文件变化, 自动刷新状态
5. **存档 diff** — 睡觉后存档变化, 自动生成战报

---

## 五、常用命令

```bash
# CLI 运行
cargo run
cargo run -- --status        # 打印压缩存档摘要
cargo run -- --parse         # 打印结构化存档 JSON
cargo run -- --ask "今天该干嘛"

# GUI 开发 (热重载)
cargo tauri dev

# GUI 生产构建
cargo tauri build

# 前端单独构建
cd frontend && npm run build

# 编译检查 (不运行)
cargo build                  # CLI + 库
cargo build -p stardew-steward-gui  # Tauri 后端
```

# AI 开发开销明细表

> 数据来源：opencode session 数据库 + 平台账单
> 模型：glm-5.2（清华 AI 平台，套餐制）
> AI 开发工具：opencode

## 总览

| 项目 | 数值 |
|------|------|
| Session 跨度 | 8/27 21:52 ~ 9/4（持续中） |
| 实际活跃开发 | ~26 小时 |
| 总 Token 数 | 134,272,319+ |
| 总花费 | ~$69 USD（原先 ~$39，9/4 GUI 开发新增 ~$30） |

## 分阶段统计

| 阶段 | 人时 | API调用次数 | 总Token数 | 总花费(USD) |
|------|------|-----------|-----------|------------|
| 需求分析 | ~3h | ~60次 | ~8M | ~$2 |
| 架构设计 | ~2h | ~40次 | ~5M | ~$1 |
| Rust核心逻辑编写 | ~8h | ~200次 | ~52M | ~$12 |
| 完善工具（9/3） | ~3h | ~150次 | ~65M | ~$18 |
| 搭GUI框架（9/4上） | ~4h | ~120次 | — | ~$6 |
| GUI功能开发与UI打磨（9/4下） | ~7h | ~250次 | — | ~$24 |
| 测试调试 | — | — | — | — |

## Token 构成（opencode 数据库精确值）

| 类型 | Token 数 |
|------|---------|
| Input (prompt) | 8,757,647 |
| Output (completion) | 153,244 |
| Reasoning | 65,556 |
| Cache read | 125,295,872 |
| **总计** | **134,272,319** |

## 单价推算

- 总花费 $18 ÷ 总 Token 134.3M ≈ $0.134 / 1M tokens（加权平均，含 cache 折扣）
- 清华平台套餐制，opencode 记录 cost=$0，实际费用通过平台充值消耗
- 注：cache read 占总量 93%，实际"新计算"的 token 仅 ~9M（input+output+reasoning）

## 备注

- 8/29~9/3 "完善工具"阶段消耗激增（~$7），主要因为：
  - fetch_wiki 工具调试（中文 wiki API、花括号解析 bug、无限循环修复）
  - R4/R5 实现（Ctrl-C 信号、rustyline、会话持久化）
  - 求解器重构（auto_tasks 吃 GameState）
  - 运气值分层校准（查 wiki 确认官方边界）
  - 多次测试运行（每次 cargo run --ask 消耗 API token）
- "实现UI"、"测试调试" 阶段待后续开发完成后补充
- 9/4 "搭GUI框架"阶段（~$6）主要工作：
  - README 更新（对照实际代码修正模块结构/工具表/配置表/CLI用法）
  - 库抽象：src/lib.rs + src/app.rs，核心代码变可复用库
  - ProgressReporter trait 扩展（on_thinking/on_error）
  - Tauri 2 后端搭建：TauriReporter + 11 个命令 + 窗口配置
  - React + Vite 前端：星露谷配色/像素字体/侧边栏布局/聊天组件
  - 编译错误修复（Config Clone, tauri::Manager 导入）
- 9/4 "GUI功能开发与UI打磨"阶段（~$24，~7h）主要工作：
  - Windows 环境搭建（winget 装 Node.js、git clone 到 Windows 路径、tauri-cli 安装）
  - 字体修复（WSL 无 CJK 字体→加载 Noto Sans SC 网页字体；移除 image-rendering: pixelated）
  - 中文输入修复（uncontrolled input + onCompositionStart/End → e.isComposing）
  - Markdown 渲染（react-markdown + remark-gfm，表格/加粗/列表/emoji）
  - 日程卡片组件（ScheduleCard：任务行/徽章/优先级/总计栏）
  - 打字机效果（逐字显示 agent 回复，18ms/字，闪烁光标）
  - 进度步骤保留（最近 2 条，闪烁，回复出现清空）
  - 像素木边框（9-slice PNG border-image，三色 3px，斜角切边）
  - 聊天气泡/输入框 9-slice 边框（左上右下透明，对角填色）
  - 设置面板（API 配置/用量统计/会话信息，get_config + update_config + 持久化 config.toml）
  - 新建对话功能（new_session 命令，清空 Agent 历史）
  - 会话级用量统计（SavedSession 含 SessionUsage，加载时恢复）
  - 删除历史记录（delete_session 命令 + UI 按钮）
  - NPC 中文名映射（系统提示词增加 28 人对照表）
  - 窗口拖拽修复（data-tauri-drag-region 覆盖整个标题栏）
  - DPI 感知窗口伸缩（LogicalSize 替代 PhysicalSize）
  - 自动保存会话（chat 命令后调 auto_save_session）
  - 大量 UI 微调（字号统一、按钮布局、分隔线、颜色调整等）
- 人时为估算值（包含与 AI 讨论方案、审查代码、调试错误的时间）
- API 调用次数为估算值（opencode 每次工具调用、每轮对话各计一次）

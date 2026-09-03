mod agent;
mod config;
mod knowledge;
mod parser;
mod solver;
mod tools;
mod ui;
mod usage;
mod validator;

use std::sync::{Arc, Mutex};
use clap::Parser;
use ui::CliReporter;

#[derive(Parser)]
#[command(name = "stardew-steward")]
#[command(about = "星露谷农场管家 — 你的专属农场日程顾问")]
struct Cli {
    /// 存档文件路径（覆盖 config.toml 中的设置）
    #[arg(long)]
    save: Option<String>,

    /// 直接提问（不进入交互模式）
    #[arg(long)]
    ask: Option<String>,

    /// 只解析存档并打印结构化 JSON（不调 LLM）
    #[arg(long)]
    parse: bool,

    /// 打印 Agent 视角的压缩存档摘要（不调 LLM）
    #[arg(long)]
    status: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = config::load()?;

    ui::print_welcome();
    println!("模型: {} | 预算: {} tokens", config.model.model, config.agent.token_budget);
    println!();

    let save_path = cli.save
        .unwrap_or_else(|| config.save.path.clone());

    if cli.parse {
        println!("解析存档: {}", save_path);
        let state = parser::parse(std::path::Path::new(&save_path))?;
        let json = serde_json::to_string_pretty(&state)?;
        println!("{}", json);
        return Ok(());
    }

    if cli.status {
        println!("{}", tools::read_save::execute(&save_path)?);
        return Ok(());
    }

    let mut tools = tools::ToolRegistry::new();

    let kb = Arc::new(Mutex::new(
        knowledge::KnowledgeBase::open(&config.knowledge.db_path)?
    ));

    let default_save = save_path.clone();
    tools.register(
        "read_save",
        "读取星露谷物语本地存档，返回结构化游戏状态（资金、日期、天气、作物、好感度、背包）。",
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "存档文件路径"
                }
            },
            "required": ["path"]
        }),
        move |args| {
            let path = args["path"].as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| default_save.clone());
            async move { tools::read_save::execute(&path) }
        },
    );

    let kb_clone = Arc::clone(&kb);
    tools.register(
        "query_knowledge",
        "搜索星露谷知识库（作物收益、NPC喜好、鱼类约束等）。传入关键词。",
        serde_json::json!({
            "type": "object",
            "properties": {
                "keyword": {
                    "type": "string",
                    "description": "搜索关键词，多个关键词用空格分隔，如'蓝莓 辣椒 Abigail'"
                }
            },
            "required": ["keyword"]
        }),
        move |args| {
            let keyword = args["keyword"].as_str().unwrap_or("").to_string();
            let kb = kb_clone.clone();
            async move { tools::query_knowledge::execute(&kb, &keyword) }
        },
    );

    tools.register(
        "solve_schedule",
        "根据任务列表和时间/体力预算，用求解器排一份最优日程。",
        serde_json::json!({
            "type": "object",
            "properties": {
                "tasks": {
                    "type": "array",
                    "description": "可选任务列表",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "time_cost": {"type": "number"},
                            "energy_cost": {"type": "integer"},
                            "money_gain": {"type": "integer"},
                            "priority": {"type": "string", "enum": ["Must", "Should", "Could"]}
                        }
                    }
                },
                "time_budget": {"type": "number", "description": "时间预算（游戏小时）"},
                "energy_budget": {"type": "integer", "description": "体力预算"}
            },
            "required": ["tasks", "time_budget", "energy_budget"]
        }),
        |args| async move {
            let tasks: Vec<solver::task::Task> = serde_json::from_value(args["tasks"].clone())
                .unwrap_or_default();
            let time_budget = args["time_budget"].as_f64().unwrap_or(12.0) as f32;
            let energy_budget = args["energy_budget"].as_i64().unwrap_or(270) as i32;
            tools::solve_schedule::execute(tasks, time_budget, energy_budget)
        },
    );

    // auto_schedule: 求解器直接吃 GameState + 知识库，自动生成任务
    let auto_save = save_path.clone();
    let auto_kb = Arc::clone(&kb);
    tools.register(
        "auto_schedule",
        "自动分析存档并生成最优日程。返回的 JSON 已经是最终日程格式，直接输出给用户即可，不需要再调其他工具或转换格式。",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        move |_| {
            let save = auto_save.clone();
            let kb = auto_kb.clone();
            async move {
            let state = parser::parse(std::path::Path::new(&save))?;
            let tasks = solver::auto_tasks::generate_tasks(&state, &kb);
            let time_budget = 14.0;
            let energy_budget = 270;
            let solved = solver::greedy::solve(tasks, time_budget, energy_budget);

            // 直接转成 DailySchedule 格式，LLM 不需要再转换
            let schedule_tasks: Vec<solver::schedule::ScheduleTask> = solved.tasks.iter().map(|t| {
                let action = match t.priority {
                    solver::task::Priority::Must => {
                        if t.name.contains("收获") { "harvest" }
                        else if t.name.contains("浇水") { "water" }
                        else { "other" }
                    }
                    solver::task::Priority::Should => {
                        if t.name.contains("送礼") { "gift" }
                        else if t.name.contains("出货") { "shop" }
                        else { "other" }
                    }
                    solver::task::Priority::Could => {
                        if t.name.contains("下矿") { "mine" }
                        else if t.name.contains("钓鱼") { "fish" }
                        else { "other" }
                    }
                }.to_string();
                solver::schedule::ScheduleTask {
                    action,
                    description: t.name.clone(),
                    time_cost: t.time_cost,
                    cost: if t.money_gain < 0 { -t.money_gain } else { 0 },
                    income: if t.money_gain > 0 { t.money_gain } else { 0 },
                    priority: format!("{:?}", t.priority).to_lowercase(),
                }
            }).collect();

            let daily = solver::schedule::DailySchedule {
                summary: format!(
                    "{} 夏{}日 | 资金{}g | 运气{:+.3} | 选了{}项任务，预计收入{}g",
                    state.date.season, state.date.day, state.money,
                    state.daily_luck, schedule_tasks.len(), solved.estimated_income,
                ),
                tasks: schedule_tasks,
                total_time: solved.total_time,
                total_cost: 0,
                total_income: solved.estimated_income,
                notes: {
                    let mut n = Vec::new();
                    if state.weather.is_raining {
                        n.push(format!("今天下雨，不用浇水"));
                    }
                    if state.daily_luck > 0.07 {
                        n.push(format!("运气极佳，强烈推荐下矿/钓鱼"));
                    } else if state.daily_luck < -0.07 {
                        n.push(format!("运气很差，不建议下矿"));
                    }
                    n
                },
            };
            Ok(serde_json::to_string_pretty(&daily)?)
            }
        },
    );

    // fetch_wiki: LLM 按需爬星露谷 wiki 补充本地知识库没有的数据
    tools.register(
        "fetch_wiki",
        "从星露谷 wiki 搜索并提取结构化信息（作物、NPC、鱼类、物品等）。当本地知识库 query_knowledge 查不到时使用此工具。",
        serde_json::json!({
            "type": "object",
            "properties": {
                "keyword": {
                    "type": "string",
                    "description": "搜索关键词（英文效果更好，如 Hot Pepper, Abigail, Catfish）"
                }
            },
            "required": ["keyword"]
        }),
        |args| {
            let keyword = args["keyword"].as_str().unwrap_or("").to_string();
            async move { tools::fetch_wiki::execute(&keyword).await }
        },
    );

    let system_prompt = format!(
        "你是「星露谷农场管家」，一个专为星露谷物语休闲玩家设计的 AI 助手。\n\
        你的职责是：\n\
        1. 先用 read_save 工具读取玩家存档，了解当前游戏状态\n\
        2. 根据存档状态给出「够用就好」的建议——不要 min-max，不要让玩家焦虑\n\
        3. 关注：哪些作物该收了、今天是谁的生日该送礼、明天有什么要熟\n\
        \n\
        规则：\n\
        - 每次被提问时，先调 read_save 读取最新存档状态\n\
        - 需要查询作物/NPC/鱼类的具体数据时，调 query_knowledge，可在 keyword 中传入多个关键词用空格分隔（如 蓝莓 辣椒 啤酒花）\n\
        - 如果 query_knowledge 查不到，调 fetch_wiki 从星露谷 wiki 在线搜索（英文关键词效果更好）\n\
        - 用户要求安排日程时，调 auto_schedule 工具，返回的 JSON 已是最终日程格式，直接原样输出即可（不要修改字段名、不要再调其他工具补充信息）\n\
        - 用中文回答\n\
        - 存档路径: {}\n\
        \n\
        回答方式：\n\
        - 如果用户问的是一般问题（如「蓝莓什么季节种」「Abigail喜欢什么」），直接用自然语言回答\n\
        - 如果用户要求安排日程或问「今天该干嘛」，输出以下 JSON 格式（不要输出其他文字）：\n\
        ```\n\
        {{\n\
          \"summary\": \"一句话总结今天该干嘛\",\n\
          \"tasks\": [\n\
            {{\n\
              \"action\": \"harvest|shop|gift|water|mine|fish|other\",\n\
              \"description\": \"具体做什么\",\n\
              \"time_cost\": 0.5,\n\
              \"cost\": 2000,\n\
              \"income\": 800,\n\
              \"priority\": \"must|should|could\"\n\
            }}\n\
          ],\n\
          \"total_time\": 8.5,\n\
          \"total_cost\": 2000,\n\
          \"total_income\": 800,\n\
          \"notes\": [\"提醒1\", \"提醒2\"]\n\
        }}\n\
        ```\n\
        注意：\n\
        - cost 是花费（正数），income 是收入（正数）\n\
        - total_cost 不能超过玩家当前资金\n\
        - total_time 不能超过一天约 12 游戏小时\n\
        - 至少要有一个 must 优先级的任务（如浇水或收获）",
        save_path
    );

    let reporter = Arc::new(CliReporter::new());

    // 从存档预读资金和季节，用于校验器
    let validator = {
        match parser::parse(std::path::Path::new(&save_path)) {
            Ok(state) => {
                let v = validator::Validator::new(
                    state.money,
                    &state.date.season,
                    12.0,
                );
                Some(v)
            }
            Err(e) => {
                eprintln!("警告: 无法预读存档用于校验: {}", e);
                None
            }
        }
    };

    let mut agent = agent::Agent::new(config, tools, reporter, system_prompt);
    if let Some(v) = validator {
        agent.set_validator(v);
    }

    if let Some(question) = &cli.ask {
        let result = agent.run(question).await?;
        ui::print_response(&result);
        ui::print_usage(&agent.usage_summary());
    } else {
        use std::io::{self, Write};
        use std::sync::atomic::Ordering;

        // R4: Ctrl-C 打断 — 单击打断当前任务，双击强制退出
        let interrupt_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
        {
            let flag = Arc::clone(&interrupt_flag);
            tokio::spawn(async move {
                loop {
                    if tokio::signal::ctrl_c().await.is_err() {
                        continue;
                    }
                    if flag.swap(true, Ordering::SeqCst) {
                        println!("\n强制退出");
                        std::process::exit(130);
                    }
                    println!("\n⚠️ 正在打断（再按一次 Ctrl-C 强制退出）...");
                }
            });
        }
        agent.set_interrupt_flag(Arc::clone(&interrupt_flag));

        println!("输入 /help 查看可用命令");

        // stdin 读取放独立线程，通过 channel 传给主循环
        // —— Agent 工作期间也能立即响应新输入（如中途 /quit）
        let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::channel::<String>(16);
        tokio::task::spawn_blocking(move || {
            let stdin = std::io::stdin();
            loop {
                let mut line = String::new();
                match stdin.read_line(&mut line) {
                    Ok(0) => break,          // EOF
                    Ok(_) => {
                        if stdin_tx.blocking_send(line).is_err() {
                            break;
                        }
                    }
                    Err(_) => {
                        // Ctrl-C 等信号打断了读取，重试
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }
        });

        let mut pending: Vec<String> = Vec::new();
        let mut quit_requested = false;

        loop {
            // 取下一条输入（优先消费排队消息）
            let raw = if !pending.is_empty() {
                let first = pending.remove(0);
                println!("你(排队): {}", first);
                first
            } else {
                print!("你: ");
                io::stdout().flush()?;
                match stdin_rx.recv().await {
                    Some(line) => line,
                    None => break, // stdin 关闭
                }
            };

            let input = raw.trim().to_string();
            if input.is_empty() {
                continue;
            }
            if input == "/quit" || input == "/exit" {
                println!("再见，祝你农场丰收！");
                break;
            }

            // 斜杠命令（R5: 历史管理）
            if let Some(rest) = input.strip_prefix('/') {
                let mut parts = rest.splitn(2, ' ');
                let cmd = parts.next().unwrap_or("");
                let arg = parts.next().map(str::trim).unwrap_or("");
                match cmd {
                    "help" => {
                        println!("可用命令:");
                        println!("  /save <名称>  保存当前会话");
                        println!("  /load <名称>  加载已保存的会话");
                        println!("  /sessions     列出所有已保存会话");
                        println!("  /log          查看 Agent 完整工作轨迹");
                        println!("  /usage        查看 Token 用量与成本");
                        println!("  /quit         退出（Agent 工作中也可立即退出）");
                    }
                    "usage" => {
                        println!("📊 {}", agent.usage_summary());
                    }
                    "save" => {
                        if arg.is_empty() {
                            println!("用法: /save <名称>");
                        } else {
                            match agent.save_session(arg) {
                                Ok(path) => println!("💾 已保存到 {}", path),
                                Err(e) => eprintln!("保存失败: {}", e),
                            }
                        }
                    }
                    "load" => {
                        if arg.is_empty() {
                            println!("用法: /load <名称>");
                        } else {
                            match agent.load_session(arg) {
                                Ok(n) => println!("📂 已加载会话「{}」（{} 条消息，上下文已恢复）", arg, n),
                                Err(e) => eprintln!("{}", e),
                            }
                        }
                    }
                    "sessions" => {
                        let list = agent::session::list_sessions();
                        if list.is_empty() {
                            println!("还没有已保存的会话，用 /save <名称> 保存");
                        } else {
                            println!("已保存的会话:");
                            for (name, ts, count) in list {
                                println!("  {} | {} | {} 条消息", name, agent::session::format_time(ts), count);
                            }
                        }
                    }
                    "log" => {
                        let traj = agent.trajectory();
                        println!("─── Agent 工作轨迹 ───");
                        for line in traj {
                            println!("{}", line);
                        }
                        println!("──────────────────────");
                    }
                    _ => println!("未知命令 /{}，输入 /help 查看", cmd),
                }
                continue;
            }

            // 每轮任务开始前清除打断标记
            interrupt_flag.store(false, Ordering::SeqCst);

            // 运行 Agent，同时监听新输入（中途 /quit 立即打断退出）
            let result = {
                let run_fut = agent.run(&input);
                tokio::pin!(run_fut);
                loop {
                    tokio::select! {
                        r = &mut run_fut => break r,
                        maybe_line = stdin_rx.recv() => {
                            match maybe_line {
                                Some(line) => {
                                    let t = line.trim().to_string();
                                    if t == "/quit" || t == "/exit" {
                                        quit_requested = true;
                                        interrupt_flag.store(true, Ordering::SeqCst);
                                        println!("\n⚠️ 收到退出请求，正在打断当前任务...");
                                    } else if !t.is_empty() {
                                        println!("\n（Agent 忙碌中，输入已排队）");
                                        pending.push(t);
                                    }
                                }
                                None => {
                                    // stdin 关闭
                                    quit_requested = true;
                                    interrupt_flag.store(true, Ordering::SeqCst);
                                }
                            }
                        }
                    }
                }
            };

            match result {
                Ok(text) => {
                    ui::print_response(&text);
                    ui::print_usage(&agent.usage_summary());
                }
                Err(e) => {
                    eprintln!("Agent 出错: {}", e);
                }
            }
            // 任务结束后清除打断标记，避免影响下一轮
            interrupt_flag.store(false, Ordering::SeqCst);

            if quit_requested {
                println!("再见，祝你农场丰收！");
                break;
            }
        }
    }

    Ok(())
}

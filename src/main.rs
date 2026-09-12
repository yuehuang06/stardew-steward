mod ui;

use std::sync::{Arc, Mutex};
use clap::Parser;
use stardew_steward::agent;
use stardew_steward::app;
use stardew_steward::config;
use stardew_steward::knowledge;
use stardew_steward::parser;
use stardew_steward::tools;
use stardew_steward::validator;
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

    let kb = Arc::new(Mutex::new(
        knowledge::KnowledgeBase::open(&config.knowledge.db_path)?
    ));

    let tools = app::build_tools(&save_path, Arc::clone(&kb));

    let system_prompt = app::build_system_prompt(&save_path);

    let reporter = Arc::new(CliReporter::new());

    // 从存档预读资金和季节，用于校验器
    let validator = {
        match parser::parse(std::path::Path::new(&save_path)) {
            Ok(state) => {
                let v = validator::Validator::new(state.money, 12.0);
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

        // rustyline: 正确处理 UTF-8 退格、行编辑、历史记录
        let mut rl = rustyline::DefaultEditor::new().expect("rustyline init");

        let mut pending: Vec<String> = Vec::new();
        let quit_requested = false;

        loop {
            // 取下一条输入（优先消费排队消息）
            let raw = if !pending.is_empty() {
                let first = pending.remove(0);
                println!("你(排队): {}", first);
                first
            } else {
                let prompt = format!("你 [{}]: ", agent.usage_brief());
                match rl.readline(&prompt) {
                    Ok(line) => {
                        let _ = rl.add_history_entry(&line);
                        line
                    }
                    Err(rustyline::error::ReadlineError::Interrupted) => {
                        // Ctrl-C 在输入时：忽略，继续等待
                        continue;
                    }
                    Err(rustyline::error::ReadlineError::Eof) => break,
                    Err(e) => {
                        eprintln!("输入错误: {}", e);
                        break;
                    }
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
                                Ok((n, rounds, usage)) => println!("📂 已加载会话「{}」（{} 条消息，{} 轮交互，上下文已恢复，本会话用量: {}in/{}out ¥{:.4}）", arg, n, rounds, usage.input_tokens, usage.output_tokens, usage.cost),
                                Err(e) => eprintln!("{}", e),
                            }
                        }
                    }
                    "sessions" => {
                        let list = agent::session::list_sessions();
                        if list.is_empty() {
                            println!("还没有已保存的会话");
                        } else {
                            println!("已保存的会话:");
                            for (name, ts, count, rounds) in list {
                                println!("  {} | {} | {} 条消息 | {} 轮交互", name, agent::session::format_time(ts), count, rounds);
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

            // 运行 Agent（Ctrl-C 可打断，通过 interrupt_flag 实现）
            let result = agent.run(&input).await;
            match result {
                Ok(text) => {
                    ui::print_response(&text);
                    ui::print_usage(&agent.usage_summary());
                    if let Some(path) = agent.auto_save_session() {
                        println!("💾 会话已自动保存: {}", path);
                    }
                }
                Err(e) => {
                    eprintln!("Agent 出错: {}", e);
                }
            }
            interrupt_flag.store(false, Ordering::SeqCst);

            if quit_requested {
                println!("再见，祝你农场丰收！");
                break;
            }
        }
    }

    Ok(())
}

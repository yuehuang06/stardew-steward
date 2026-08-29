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
                .unwrap_or(&default_save);
            tools::read_save::execute(path)
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
                    "description": "搜索关键词，如作物名、NPC名、鱼名"
                }
            },
            "required": ["keyword"]
        }),
        move |args| {
            let keyword = args["keyword"].as_str().unwrap_or("");
            tools::query_knowledge::execute(&kb_clone, keyword)
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
        |args| {
            let tasks: Vec<solver::task::Task> = serde_json::from_value(args["tasks"].clone())
                .unwrap_or_default();
            let time_budget = args["time_budget"].as_f64().unwrap_or(12.0) as f32;
            let energy_budget = args["energy_budget"].as_i64().unwrap_or(270) as i32;
            tools::solve_schedule::execute(tasks, time_budget, energy_budget)
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
        - 建议要具体、可执行，不要泛泛而谈\n\
        - 用中文回答\n\
        - 存档路径: {}",
        save_path
    );

    let reporter = Arc::new(CliReporter::new());
    let mut agent = agent::Agent::new(config, tools, reporter, system_prompt);

    if let Some(question) = &cli.ask {
        let result = agent.run(question).await?;
        ui::print_response(&result);
        ui::print_usage(&agent.usage_summary());
    } else {
        use std::io::{self, BufRead, Write};

        loop {
            print!("你: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let input = input.trim();
            if input.is_empty() {
                continue;
            }
            if input == "/quit" || input == "/exit" {
                println!("再见，祝你农场丰收！");
                break;
            }

            match agent.run(input).await {
                Ok(result) => {
                    ui::print_response(&result);
                    ui::print_usage(&agent.usage_summary());
                }
                Err(e) => {
                    eprintln!("Agent 出错: {}", e);
                }
            }
        }
    }

    Ok(())
}

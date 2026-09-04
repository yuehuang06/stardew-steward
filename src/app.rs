use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::knowledge::KnowledgeBase;
use crate::parser;
use crate::solver;
use crate::tools::ToolRegistry;

pub fn build_tools(save_path: &str, kb: Arc<Mutex<KnowledgeBase>>) -> ToolRegistry {
    let mut tools = ToolRegistry::new();

    let default_save = save_path.to_string();
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
            async move { crate::tools::read_save::execute(&path) }
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
            async move { crate::tools::query_knowledge::execute(&kb, &keyword) }
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
            crate::tools::solve_schedule::execute(tasks, time_budget, energy_budget)
        },
    );

    let auto_save = save_path.to_string();
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
            let state = parser::parse(Path::new(&save))?;
            let tasks = solver::auto_tasks::generate_tasks(&state, &kb);
            let time_budget = 14.0;
            let energy_budget = 270;
            let solved = solver::greedy::solve(tasks, time_budget, energy_budget);

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

    tools.register(
        "fetch_wiki",
        "从星露谷中文 wiki 搜索并提取结构化信息（作物、NPC、鱼类、物品等）。当本地知识库 query_knowledge 查不到时使用此工具。支持中文关键词搜索。",
        serde_json::json!({
            "type": "object",
            "properties": {
                "keyword": {
                    "type": "string",
                    "description": "搜索关键词（中英文均可，如 夏季亮片, Abigail, 鲶鱼）"
                }
            },
            "required": ["keyword"]
        }),
        |args| {
            let keyword = args["keyword"].as_str().unwrap_or("").to_string();
            async move { crate::tools::fetch_wiki::execute(&keyword).await }
        },
    );

    tools
}

pub fn build_system_prompt(save_path: &str) -> String {
    format!(
        "你是「星露谷农场管家」，一个专为星露谷物语休闲玩家设计的 AI 助手。\n\
        你的职责是：\n\
        1. 先用 read_save 工具读取玩家存档，了解当前游戏状态\n\
        2. 根据存档状态给出「够用就好」的建议——不要 min-max，不要让玩家焦虑\n\
        3. 关注：哪些作物该收了、今天是谁的生日该送礼、明天有什么要熟\n\
        \n\
        规则：\n\
        - 每次被提问时，先调 read_save 读取最新存档状态\n\
        - 需要查询作物/NPC/鱼类的具体数据时，调 query_knowledge，可在 keyword 中传入多个关键词用空格分隔（如 蓝莓 辣椒 啤酒花）\n\
        - 如果 query_knowledge 查不到，调 fetch_wiki 从星露谷中文 wiki 在线搜索（中英文关键词均可，如 夏季亮片, Abigail, 鲶鱼）。每次查询使用不同的关键词时，最多调用 2 次 fetch_wiki，之后用已有结果回答\n\
        - 如果用户的问题模糊到无法确定查询目标（如「那个花」），先反问用户确认，不要盲目猜测后查询\n\
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
    )
}

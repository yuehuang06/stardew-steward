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
        "自动分析存档并生成确定性基线日程（基于真实收获/浇水/任务/运气规则求解）。把它当作草稿：你应该基于存档细节优化它——改写摘要、丰富每项任务的具体描述（去哪、找谁、为什么）、调整任务取舍、补充提醒，但保持 JSON 字段结构不变。",
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

    let advisor_save = save_path.to_string();
    let advisor_kb = Arc::clone(&kb);
    tools.register(
        "crop_advisor",
        "分析当前作物的收益（g/天），与当季其他作物对比，推荐更赚钱的种植方案。适合用户问「种什么最赚」「该不该换作物」时调用。",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        move |_| {
            let save = advisor_save.clone();
            let kb = advisor_kb.clone();
            async move {
                let state = parser::parse(Path::new(&save))?;
                crate::tools::crop_advisor::execute(&state, &kb)
            }
        },
    );

    let gift_save = save_path.to_string();
    let gift_kb = Arc::clone(&kb);
    tools.register(
        "gift_finder",
        "扫描背包和木箱中的物品，与知识库中 NPC 的喜好交叉匹配，推荐最优送礼方案。适合用户问「该送谁礼物」「谁快生日了」时调用。",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        move |_| {
            let save = gift_save.clone();
            let kb = gift_kb.clone();
            async move {
                let state = parser::parse(Path::new(&save))?;
                crate::tools::gift_finder::execute(&state, &kb)
            }
        },
    );

    let farm_save = save_path.to_string();
    let farm_kb = Arc::clone(&kb);
    tools.register(
        "farm_hand",
        "托管农场操作：直接修改存档文件替玩家完成重复劳动。action 可选: water_all(浇水所有作物)、harvest_all(收获所有成熟作物并自动出售)、clear_dead(清理枯死作物)、rollback_day(回档到前一天，用游戏自带的 _old 存档覆盖当前存档，适合矿井晕倒等失误后反悔)。操作前自动备份存档，修改后校验 XML 合法性，失败自动回滚。注意：必须在游戏关闭时使用，否则修改会被覆盖。",
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["water_all", "harvest_all", "clear_dead", "rollback_day"],
                    "description": "要执行的农场操作"
                }
            },
            "required": ["action"]
        }),
        move |args| {
            let save = farm_save.clone();
            let kb = farm_kb.clone();
            async move {
                let action = args["action"].as_str().unwrap_or("").to_string();
                crate::tools::farm_hand::execute(&save, &action, &kb)
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
        3. 关注：哪些作物该收了、今天是谁的生日该送礼、明天有什么要熟、玩家有什么任务在身\n\
        \n\
        规则：\n\
        - 每次被提问时，先调 read_save 读取最新存档状态\n\
        - 需要查询作物/NPC/鱼类的具体数据时，调 query_knowledge，可在 keyword 中传入多个关键词用空格分隔（如 蓝莓 辣椒 啤酒花）\n\
        - 如果 query_knowledge 查不到，调 fetch_wiki 从星露谷中文 wiki 在线搜索（中英文关键词均可，如 夏季亮片, Abigail, 鲶鱼）。每次查询使用不同的关键词时，最多调用 2 次 fetch_wiki，之后用已有结果回答\n\
        - 如果用户的问题模糊到无法确定查询目标（如「那个花」），先反问用户确认，不要盲目猜测后查询\n\
        - 用户要求安排日程或问「今天该干嘛」时，调 auto_schedule 工具拿到确定性基线日程，然后**结合存档细节优化它**：根据季节/天气/运气/任务/好感度改写 summary、把每项任务的 description 写具体（做什么、去哪里、找谁、为什么值得做）、可增删调整任务、在 notes 里加入贴心提醒（明天什么熟了、谁的生日快到了等）。保持 JSON 字段结构不变，total 数字与最终任务列表一致，最后只输出优化后的完整 JSON（不要 JSON 以外的文字）\n\
        - 用户问「种什么最赚」「该不该换作物」「哪个作物收益高」时，调 crop_advisor 工具，它会用确定性计算对比所有作物的 g/天\n\
        - 用户问「该送谁礼物」「谁快生日了」「送什么好」时，调 gift_finder 工具，它会扫描背包和木箱匹配 NPC 喜好\n\
        - 用户说「帮我浇水」「帮我收获」「清理枯死的」等托管操作时，调 farm_hand 工具，直接修改存档完成操作。提醒用户：必须先关闭游戏再使用，否则修改会被覆盖\n\
        - 用户说「回档」「回到昨天」「我想反悔」时，调 farm_hand(action=\"rollback_day\")，用游戏自带的 _old 存档回退一天\n\
        - 用中文回答\n\
        - 所有 NPC 名字一律使用官方中文名，不要用英文名。常见对照：Abigail=阿比盖尔, Sebastian=塞巴斯缇安, Sam=山姆, Penny=潘妮, Leah=莉娅, Maru=玛鲁, Alex=亚历克斯, Haley=海莉, Emily=艾米丽, Shane=谢恩, Caroline=卡罗琳, Demetrius=德米崔斯, Dwarf=矮人, Elliott=艾里欧特, George=乔治, Gus=古斯, Jas=贾斯, Jodi=乔迪, Kent=肯特, Lewis=路易斯, Linus=莱纳斯, Marnie=玛妮, Pam=帕姆, Pierre=皮埃尔, Robin=罗宾, Sandy=桑迪, Vincent=文森特, Willy=威利, Wizard=法师, Krobus=科罗巴斯, Leo=雷欧\n\
        - 存档路径: {}\n\
        \n\
        日程安排指引：\n\
        - 日程不要只包含浇水和收菜，要根据当日运气和天气设计丰富的一天\n\
        - 运气好（>0.07）时重点推荐下矿和钓鱼，收益远高于日常\n\
        - 运气差（<-0.05）时避免下矿，推荐钓鱼、采集、社交\n\
        - 雨天不用浇水，省下的时间用来钓鱼或下矿\n\
        - 存档中如果有未完成的任务（交付/寻找/收集/剧情），在日程中安排时间去做\n\
        - 好感度低的 NPC 可以日常拜访聊天\n\
        - 有资金时建议去商店买当季种子\n\
        - 日程里每一项任务都要具体：做什么、去哪里、预计花多少时间\n\
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
        - 至少要有一个 must 优先级的任务（如浇水或收获）\n\
        - 日程应该包含 4-8 项任务，不要只有浇水和收菜",
        save_path
    )
}

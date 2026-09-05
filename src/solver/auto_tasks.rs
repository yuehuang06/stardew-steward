// 自动任务生成: 直接从 GameState + 知识库推导候选任务
// 不依赖 LLM 提供参数 — 这是"把决策从 LLM 拿回 Rust"的核心

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::knowledge::KnowledgeBase;
use crate::parser::GameState;
use super::task::{Task, Priority};

/// 从游戏状态自动生成候选任务列表
pub fn generate_tasks(state: &GameState, kb: &Arc<Mutex<KnowledgeBase>>) -> Vec<Task> {
    let kb = kb.lock().unwrap();
    let mut tasks = Vec::new();

    // 1. 收获 — 扫描所有可收作物
    let mut harvest_by_name: HashMap<String, u32> = HashMap::new();
    let mut total_living_crops = 0u32;
    for crop in &state.crops {
        if crop.is_dead {
            continue;
        }
        total_living_crops += 1;
        if crop.days_to_harvest == 0 {
            *harvest_by_name.entry(crop.name.clone()).or_insert(0) += 1;
        }
    }

    let mut harvest_income = 0i32;
    let mut harvest_count = 0u32;
    for (name, count) in &harvest_by_name {
        let price = kb.get_crop_price(name).unwrap_or(0);
        let income = *count as i32 * price;
        harvest_income += income;
        harvest_count += count;
    }
    if harvest_count > 0 {
        tasks.push(Task {
            name: format!("收获{}株作物（{}种）", harvest_count, harvest_by_name.len()),
            time_cost: harvest_count as f32 * 0.01,
            energy_cost: harvest_count as i32,
            money_gain: harvest_income,
            priority: Priority::Must,
        });
    }

    // 2. 浇水 — 非雨天必做
    if !state.weather.is_raining && total_living_crops > 0 {
        tasks.push(Task {
            name: format!("浇水（{}株作物）", total_living_crops),
            time_cost: total_living_crops as f32 * 0.02,
            energy_cost: (total_living_crops as i32) * 2,
            money_gain: 0,
            priority: Priority::Must,
        });
    }

    // 3. 存档中的任务 — 根据任务类型生成具体行动
    for quest in &state.quests {
        if quest.completed {
            continue;
        }
        match quest.quest_type.as_str() {
            "delivery" => {
                let npc = quest.target.as_deref().unwrap_or("NPC");
                let item = quest.item.as_deref().unwrap_or("物品");
                let n = quest.number.unwrap_or(1);
                tasks.push(Task {
                    name: format!("交付任务「{}」: 送给{} {}个{}", quest.title, npc, n, item),
                    time_cost: 0.5,
                    energy_cost: 0,
                    money_gain: quest.money_reward,
                    priority: Priority::Should,
                });
            }
            "lost_item" => {
                let loc = if quest.description.contains("动物") { "动物商店" } else { "镇上" };
                tasks.push(Task {
                    name: format!("寻找任务「{}」: {} ({})", quest.title, quest.objective, loc),
                    time_cost: 1.5,
                    energy_cost: 20,
                    money_gain: quest.money_reward,
                    priority: Priority::Should,
                });
            }
            "story" => {
                tasks.push(Task {
                    name: format!("剧情任务「{}」: {}", quest.title, quest.objective),
                    time_cost: 2.0,
                    energy_cost: 50,
                    money_gain: quest.money_reward,
                    priority: if quest.title.contains("矿") { Priority::Should } else { Priority::Could },
                });
            }
            "daily" => {
                tasks.push(Task {
                    name: format!("每日任务: {}", quest.objective),
                    time_cost: 1.0,
                    energy_cost: 30,
                    money_gain: quest.money_reward,
                    priority: Priority::Should,
                });
            }
            "collect" => {
                tasks.push(Task {
                    name: format!("收集任务「{}」: {}", quest.title, quest.objective),
                    time_cost: 1.5,
                    energy_cost: 40,
                    money_gain: quest.money_reward,
                    priority: Priority::Should,
                });
            }
            _ => {}
        }
    }

    // 4. 送礼 — 近期生日的 NPC
    for f in &state.friendships {
        if let Some(birthday) = kb.get_npc_birthday(&f.npc) {
            if let Some(days) = days_to_birthday(&birthday, &state.date.season, state.date.day) {
                if days <= 7 && days >= 0 {
                    tasks.push(Task {
                        name: format!("给 {} 送生日礼物（{}天后生日）", f.npc, days),
                        time_cost: 0.5,
                        energy_cost: 0,
                        money_gain: 0,
                        priority: Priority::Should,
                    });
                }
            }
        }
    }

    // 5. 下矿 — 根据运气调整优先级和收益
    if state.daily_luck > -0.05 {
        let (est_income, reason) = if state.daily_luck > 0.07 {
            (800, "运气极佳，矿石和楼梯都很丰富")
        } else if state.daily_luck > 0.02 {
            (500, "运气不错，适合探索深层")
        } else if state.daily_luck > -0.02 {
            (300, "运气平平，可以稳步推进")
        } else {
            (150, "运气稍差但还能下矿探索")
        };
        let prio = if state.daily_luck > 0.07 { Priority::Should } else { Priority::Could };
        tasks.push(Task {
            name: format!("下矿探险（{}，预计矿石收益~{}g）", reason, est_income),
            time_cost: 4.0,
            energy_cost: 150,
            money_gain: est_income,
            priority: prio,
        });
    }

    // 6. 钓鱼 — 雨天或运气好时推荐
    if state.weather.is_raining || state.daily_luck > 0.0 {
        let (est_income, reason) = if state.weather.is_raining {
            (500, "雨天鱼活跃且无需浇水")
        } else if state.daily_luck > 0.07 {
            (400, "运气极佳，可能钓到宝箱")
        } else {
            (250, "运气不错，适合钓鱼")
        };
        tasks.push(Task {
            name: format!("钓鱼（{}，预计收益~{}g）", reason, est_income),
            time_cost: 3.0,
            energy_cost: 60,
            money_gain: est_income,
            priority: Priority::Could,
        });
    }

    // 7. 采集 — 野外觅食，四季都有
    let foraging_items = match state.date.season.as_str() {
        "spring" => "野山葵、水仙、韭葱",
        "summer" => "红蘑菇、葡萄、香料浆果",
        "fall" => "蘑菇、野梅、黑莓、榛子",
        "winter" => "冬根、雪山药、番红花",
        _ => "采集品",
    };
    tasks.push(Task {
        name: format!("野外采集（{}，锻炼采集技能）", foraging_items),
        time_cost: 1.5,
        energy_cost: 40,
        money_gain: 100,
        priority: Priority::Could,
    });

    // 8. 买种子 — 根据季节和资金推荐
    let seed_suggestion = match state.date.season.as_str() {
        "spring" => "草莓种子（节日后买）或土豆种子",
        "summer" => "蓝莓种子或辣椒种子",
        "fall" => "南瓜种子或蔓越莓种子",
        "winter" => "冬季种子（测完用）或准备春种",
        _ => "当季种子",
    };
    if state.money > 2000 {
        tasks.push(Task {
            name: format!("去皮埃尔商店买种子（推荐{}，当前资金{}g）", seed_suggestion, state.money),
            time_cost: 0.5,
            energy_cost: 0,
            money_gain: 0,
            priority: Priority::Could,
        });
    }

    // 9. 出售收获 — 有可收作物时
    if harvest_income > 0 {
        tasks.push(Task {
            name: format!("把收获放入出货箱（预计{}g明天到账）", harvest_income),
            time_cost: 0.3,
            energy_cost: 0,
            money_gain: 0,
            priority: Priority::Should,
        });
    }

    // 10. 社交 — 好感度较低的 NPC 可以日常拜访
    let low_friendship: Vec<_> = state.friendships
        .iter()
        .filter(|f| f.points < 250)
        .take(3)
        .collect();
    if !low_friendship.is_empty() {
        let names: Vec<String> = low_friendship.iter()
            .map(|f| f.npc.clone())
            .collect();
        tasks.push(Task {
            name: format!("社交拜访: 找 {} 聊天（好感度较低）", names.join("、")),
            time_cost: 1.0,
            energy_cost: 0,
            money_gain: 0,
            priority: Priority::Could,
        });
    }

    tasks
}

/// 粗略计算距生日还有几天
fn days_to_birthday(birthday: &str, current_season: &str, current_day: u32) -> Option<i32> {
    // birthday 格式: "Fall 13" / "Winter 10"
    let parts: Vec<&str> = birthday.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }
    let bday_season = parts[0].to_lowercase();
    let bday_day: u32 = parts[1].parse().ok()?;

    let season_order = |s: &str| -> i32 {
        match s {
            "spring" => 0,
            "summer" => 1,
            "fall" => 2,
            "winter" => 3,
            _ => 0,
        }
    };

    let curr_s = season_order(current_season);
    let bday_s = season_order(&bday_season);
    let days_per_season = 28i32;

    let curr_total = curr_s * days_per_season + current_day as i32;
    let bday_total = bday_s * days_per_season + bday_day as i32;

    let diff = bday_total - curr_total;
    if diff < 0 {
        Some(diff + days_per_season * 4) // 明年的生日
    } else {
        Some(diff)
    }
}

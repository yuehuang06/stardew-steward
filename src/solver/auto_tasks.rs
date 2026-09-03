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

    // 3. 送礼 — 近期生日的 NPC
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

    // 4. 下矿 — 运气好时推荐
    if state.daily_luck > 0.02 {
        let est_income = if state.daily_luck > 0.07 { 500 } else { 300 };
        tasks.push(Task {
            name: format!("下矿探险（运气{:+.3}，预计矿石收益~{}g）", state.daily_luck, est_income),
            time_cost: 4.0,
            energy_cost: 150,
            money_gain: est_income,
            priority: Priority::Could,
        });
    }

    // 5. 钓鱼 — 雨天或运气好时推荐
    if state.weather.is_raining || state.daily_luck > 0.02 {
        let est_income = if state.weather.is_raining { 400 } else { 200 };
        tasks.push(Task {
            name: format!("钓鱼（{}，预计收益~{}g）",
                if state.weather.is_raining { "雨天适合" } else { "运气不错" },
                est_income),
            time_cost: 3.0,
            energy_cost: 60,
            money_gain: est_income,
            priority: Priority::Could,
        });
    }

    // 6. 出售收获 — 有可收作物时
    if harvest_income > 0 {
        tasks.push(Task {
            name: format!("把收获放入出货箱（预计{}g明天到账）", harvest_income),
            time_cost: 0.3,
            energy_cost: 0,
            money_gain: 0,
            priority: Priority::Should,
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

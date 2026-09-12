use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::knowledge::KnowledgeBase;
use crate::parser::GameState;
use serde::Serialize;

#[derive(Serialize)]
struct CropAnalysis {
    current_crops: Vec<CropProfit>,
    season_ranking: Vec<CropProfit>,
    total_potential_income: i32,
    recommendation: Vec<String>,
}

#[derive(Serialize)]
struct CropProfit {
    name: String,
    count: u32,
    sell_price: i32,
    growth_days: i32,
    regrows: bool,
    gold_per_day: f32,
    total_income: i32,
}

/// crop_advisor: 分析当前作物收益并与当季其他作物对比
pub fn execute(state: &GameState, kb: &Arc<Mutex<KnowledgeBase>>) -> anyhow::Result<String> {
    let kb = kb.lock().map_err(|e| anyhow::anyhow!("知识库锁异常: {}", e))?;
    let season_cn = match state.date.season.as_str() {
        "spring" => "春", "summer" => "夏", "fall" => "秋", "winter" => "冬", _ => &state.date.season,
    };

    let mut crop_groups: HashMap<String, u32> = HashMap::new();
    let mut total_potential_income = 0i32;

    for crop in &state.crops {
        if crop.is_dead { continue; }
        *crop_groups.entry(crop.name.clone()).or_insert(0) += 1;
    }

    let mut current: Vec<CropProfit> = Vec::new();
    for (name, count) in &crop_groups {
        if let Some(info) = kb.get_crop_info(&name) {
            let cycle_days = if info.regrows && info.growth_days > 0 {
                info.growth_days.max(1) as f32
            } else {
                info.growth_days.max(1) as f32
            };
            let gold_per_day = info.sell_price as f32 / cycle_days;
            let total_income = *count as i32 * info.sell_price;
            total_potential_income += total_income;
            current.push(CropProfit {
                name: name.clone(),
                count: *count,
                sell_price: info.sell_price,
                growth_days: info.growth_days,
                regrows: info.regrows,
                gold_per_day,
                total_income,
            });
        }
    }
    current.sort_by(|a, b| b.gold_per_day.partial_cmp(&a.gold_per_day).unwrap_or(std::cmp::Ordering::Equal));

    let mut ranking: Vec<CropProfit> = kb.get_crops_for_season(&state.date.season)
        .into_iter()
        .map(|(name, info)| {
            let cycle_days = if info.regrows && info.growth_days > 0 {
                info.growth_days.max(1) as f32
            } else {
                info.growth_days.max(1) as f32
            };
            let gold_per_day = info.sell_price as f32 / cycle_days;
            CropProfit {
                name,
                count: 0,
                sell_price: info.sell_price,
                growth_days: info.growth_days,
                regrows: info.regrows,
                gold_per_day,
                total_income: 0,
            }
        })
        .collect();
    ranking.sort_by(|a, b| b.gold_per_day.partial_cmp(&a.gold_per_day).unwrap_or(std::cmp::Ordering::Equal));

    let mut recommendations = Vec::new();

    if !current.is_empty() {
        let best = &current[0];
        let worst = &current[current.len() - 1];
        if best.name != worst.name {
            recommendations.push(format!(
                "你种了{}，{:.1}g/天（最佳）；收益最低的是{}，仅{:.1}g/天",
                best.name, best.gold_per_day, worst.name, worst.gold_per_day
            ));
        }
    }

    if !ranking.is_empty() {
        let top = &ranking[0];
        let has_top = current.iter().any(|c| c.name == top.name);
        if !has_top {
            let max_buy = state.money / top.sell_price.max(1);
            recommendations.push(format!(
                "{}当季最赚的是{}（{:.1}g/天，{}复收），你还没种。资金{}g可买{}株",
                season_cn, top.name, top.gold_per_day,
                if top.regrows { "可" } else { "不" },
                state.money, max_buy,
            ));
        }
    }

    let remaining_days = 28 - state.date.day as i32;
    if remaining_days > 0 && !ranking.is_empty() {
        for crop in &ranking {
            if crop.growth_days > 0 && crop.growth_days <= remaining_days {
                if !current.iter().any(|c| c.name == crop.name) {
                    recommendations.push(format!(
                        "还剩{}天本季结束，现在种{}来得及（生长{}天，售价{}g）",
                        remaining_days, crop.name, crop.growth_days, crop.sell_price,
                    ));
                }
                break;
            }
        }
    }

    if recommendations.is_empty() {
        recommendations.push("当前作物选择合理，继续照顾即可。".to_string());
    }

    let analysis = CropAnalysis {
        current_crops: current,
        season_ranking: ranking,
        total_potential_income,
        recommendation: recommendations,
    };

    Ok(serde_json::to_string_pretty(&analysis)?)
}

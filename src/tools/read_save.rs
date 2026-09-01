use crate::parser::GameState;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
struct CompactState {
    money: i32,
    date: String,
    weather: String,
    skills: String,
    crop_summary: Vec<CropSummary>,
    top_friendships: Vec<String>,
    inventory: Vec<String>,
    chests: Vec<String>,
}

#[derive(Serialize)]
struct CropSummary {
    name: String,
    count: u32,
    harvestable: u32,
    days_to_harvest: Vec<i32>,
}

/// 工具: read_save
/// 读取本地存档，返回**压缩后**的游戏状态 JSON
/// 135 株作物压缩成几条汇总（按名字分组），而非逐条列出坐标
pub fn execute(path: &str) -> anyhow::Result<String> {
    let state = crate::parser::parse(std::path::Path::new(path))?;

    let mut groups: HashMap<String, CropSummary> = HashMap::new();
    for crop in &state.crops {
        let entry = groups.entry(crop.name.clone()).or_insert(CropSummary {
            name: crop.name.clone(),
            count: 0,
            harvestable: 0,
            days_to_harvest: Vec::new(),
        });
        entry.count += 1;
        if !crop.is_dead {
            entry.days_to_harvest.push(crop.days_to_harvest);
            if crop.days_to_harvest == 0 {
                entry.harvestable += 1;
            }
        }
    }

    let mut crop_summary: Vec<_> = groups.into_values().collect();
    crop_summary.sort_by(|a, b| b.harvestable.cmp(&a.harvestable));

    let top_friendships: Vec<String> = state.friendships.iter()
        .take(10)
        .map(|f| format!("{}({})", f.npc, f.points))
        .collect();

    let chest_summary: Vec<String> = state.chests.iter()
        .map(|c| {
            let items: Vec<String> = c.items.iter()
                .map(|i| format!("{}×{}", i.name, i.count))
                .collect();
            format!("{}({:.0},{:.0}): {}", c.location, c.x, c.y, items.join(", "))
        })
        .collect();

    let compact = CompactState {
        money: state.money,
        date: format!("第{}年 {} {}日", state.date.year, state.date.season, state.date.day),
        weather: format!(
            "今天{}{}，明天{}",
            if state.weather.is_raining { "下雨" } else { "晴天" },
            if state.weather.is_lightning { "打雷" } else { "" },
            state.weather.tomorrow,
        ),
        skills: format!(
            "种地{} 矿{} 战{} 采{} 钓{}",
            state.skills.farming, state.skills.mining, state.skills.combat,
            state.skills.foraging, state.skills.fishing,
        ),
        crop_summary,
        top_friendships,
        inventory: state.inventory.iter()
            .map(|i| format!("{}×{}", i.name, i.count))
            .collect(),
        chests: chest_summary,
    };

    Ok(serde_json::to_string_pretty(&compact)?)
}

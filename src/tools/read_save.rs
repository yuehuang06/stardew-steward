use crate::parser::GameState;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
struct CompactState {
    money: i32,
    date: String,
    weather: String,
    luck: String,
    skills: String,
    watered_summary: String,
    crop_summary: Vec<CropSummary>,
    top_friendships: Vec<String>,
    inventory: Vec<String>,
    chests: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    quests: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    buildings: Vec<String>,
}

#[derive(Serialize)]
struct CropSummary {
    name: String,
    count: u32,
    harvestable: u32,
    unwatered: u32,
    days_to_harvest: Vec<i32>,
}

/// 工具: read_save
/// 读取本地存档，返回**压缩后**的游戏状态 JSON
/// 135 株作物压缩成几条汇总（按名字分组），而非逐条列出坐标
pub fn execute(path: &str) -> anyhow::Result<String> {
    let state = crate::parser::parse(std::path::Path::new(path))?;

    let mut groups: HashMap<String, CropSummary> = HashMap::new();
    let mut total_living = 0u32;
    let mut total_unwatered = 0u32;
    for crop in &state.crops {
        if crop.is_dead {
            continue; // 枯死的作物不进汇总（换季后遗留的死株没有行动价值）
        }
        let entry = groups.entry(crop.name.clone()).or_insert(CropSummary {
            name: crop.name.clone(),
            count: 0,
            harvestable: 0,
            unwatered: 0,
            days_to_harvest: Vec::new(),
        });
        entry.count += 1;
        total_living += 1;
        if !crop.watered {
            total_unwatered += 1;
            entry.unwatered += 1;
        }
        entry.days_to_harvest.push(crop.days_to_harvest);
        if crop.harvestable {
            entry.harvestable += 1;
        }
    }

    let mut crop_summary: Vec<_> = groups.into_values().collect();
    crop_summary.sort_by(|a, b| b.harvestable.cmp(&a.harvestable));

    let watered_summary = if state.junimo_huts > 0 {
        format!(
            "已浇水{}/{}株（农场有{}个Junimo小屋，Junimo每天自动浇水，通常无需手动浇）",
            total_living.saturating_sub(total_unwatered), total_living, state.junimo_huts
        )
    } else if state.weather.is_raining {
        format!("今天下雨，{}株作物自动浇水", total_living)
    } else {
        format!(
            "已浇水{}/{}株，未浇{}株",
            total_living.saturating_sub(total_unwatered), total_living, total_unwatered
        )
    };

    let building_list: Vec<String> = state.buildings.iter()
        .map(|b| format!("{}×{}", b.name, b.count))
        .collect();

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

    let quest_list: Vec<String> = state.quests.iter()
        .map(|q| {
            let reward = if q.money_reward > 0 { format!("（奖励{}g）", q.money_reward) } else { String::new() };
            let target = q.target.as_ref().map(|t| format!(" → {}", t)).unwrap_or_default();
            format!("[{}] {}{}: {}{}", q.quest_type, q.title, target, q.objective, reward)
        })
        .collect();

    let compact = CompactState {
        money: state.money,
        date: format!("第{}年 {} {}日", state.date.year, season_cn(&state.date.season), state.date.day),
        weather: format!(
            "今天{}{}，明天{}",
            if state.weather.is_raining { "下雨" } else { "晴天" },
            if state.weather.is_lightning { "打雷" } else { "" },
            weather_cn(&state.weather.tomorrow),
        ),
        luck: {
            // 分层与官方电视占卜频道一致（wiki: TV::getFortuneForecast）
            let v = state.daily_luck;
            let (tier, tv) = if v >= 0.1 {
                ("运气极佳", "电视⭐最亮星星：精灵非常开心，会竭力降下好运")
            } else if v > 0.07 {
                ("运气非常好", "电视⭐星星：精灵非常开心，会竭力降下好运")
            } else if v > 0.02 {
                ("运气不错", "电视🔺金字塔：精灵心情不错，你会有一点额外好运")
            } else if v == 0.0 {
                ("绝对中立（罕见）", "电视〰️漩涡光：精灵绝对中立，非常少见")
            } else if v >= -0.02 {
                ("运气平平", "电视〰️漩涡光：精灵保持中立，这天由你自己掌控")
            } else if v >= -0.07 {
                ("运气不佳", "电视🦇蝙蝠：精灵有些恼火，运气不会站在你这边")
            } else {
                ("运气很差", "电视💀骷髅：精灵非常不满，会尽力给你使绊子")
            };
            let advice = if v > 0.07 {
                "强烈适合下矿/钓鱼（矿石掉落、宝石节点、宝箱概率都更高）"
            } else if v > 0.02 {
                "适合下矿/钓鱼（掉落和宝箱略好）"
            } else if v >= -0.02 {
                "运气影响不大，随意安排"
            } else {
                "不建议下矿/钓鱼（掉落差、死亡损失更大），适合干农活、送礼、钓鱼以外的日常"
            };
            format!("{}（{:+.3}，{}。{}）", tier, v, tv, advice)
        },
        skills: format!(
            "种地{} 矿{} 战{} 采{} 钓{}",
            state.skills.farming, state.skills.mining, state.skills.combat,
            state.skills.foraging, state.skills.fishing,
        ),
        crop_summary,
        watered_summary,
        top_friendships,
        inventory: state.inventory.iter()
            .map(|i| format!("{}×{}", i.name, i.count))
            .collect(),
        chests: chest_summary,
        quests: quest_list,
        buildings: building_list,
    };

    Ok(serde_json::to_string_pretty(&compact)?)
}

fn season_cn(s: &str) -> &str {
    match s.to_lowercase().as_str() {
        "spring" => "春",
        "summer" => "夏",
        "fall" => "秋",
        "winter" => "冬",
        _ => s,
    }
}

fn weather_cn(s: &str) -> &str {
    match s.to_lowercase().as_str() {
        "sun" => "晴天",
        "rain" => "雨天",
        "snow" => "雪天",
        "storm" => "暴雨",
        "wind" => "大风",
        "fallleaves" => "落叶",
        _ => s,
    }
}

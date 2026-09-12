use std::sync::{Arc, Mutex};

use crate::knowledge::KnowledgeBase;
use crate::parser::GameState;
use serde::Serialize;

#[derive(Serialize)]
struct GiftSuggestion {
    npc: String,
    birthday: String,
    friendship_points: u32,
    days_to_birthday: Option<i32>,
    best_gift: String,
    gift_source: String,
    match_type: String,
    available_count: u32,
}

/// gift_finder: 扫描背包+木箱，匹配 NPC 喜好，推荐送礼方案
pub fn execute(state: &GameState, kb: &Arc<Mutex<KnowledgeBase>>) -> anyhow::Result<String> {
    let kb = kb.lock().map_err(|e| anyhow::anyhow!("知识库锁异常: {}", e))?;
    let mut suggestions = Vec::new();

    let mut all_items: Vec<(String, u32)> = Vec::new();
    for item in &state.inventory {
        all_items.push((item.name.clone(), item.count));
    }
    for chest in &state.chests {
        for item in &chest.items {
            all_items.push((item.name.clone(), item.count));
        }
    }

    for friend in &state.friendships {
        let loves = kb.get_npc_loves(&friend.npc);
        let likes = kb.get_npc_likes(&friend.npc);
        let birthday = kb.get_npc_birthday(&friend.npc).unwrap_or_default();

        let mut found_love: Option<(&str, u32)> = None;
        for (item_name, count) in &all_items {
            for loved in &loves {
                if item_name.contains(loved.as_str()) || loved.contains(item_name.as_str()) {
                    found_love = Some((item_name, *count));
                    break;
                }
            }
            if found_love.is_some() { break; }
        }

        let mut found_like: Option<(&str, u32)> = None;
        if found_love.is_none() {
            for (item_name, count) in &all_items {
                for liked in &likes {
                    if item_name.contains(liked.as_str()) || liked.contains(item_name.as_str()) {
                        found_like = Some((item_name, *count));
                        break;
                    }
                }
                if found_like.is_some() { break; }
            }
        }

        let days_to = if !birthday.is_empty() {
            crate::solver::auto_tasks::days_to_birthday(&birthday, &state.date.season, state.date.day)
        } else {
            None
        };

        let is_birthday_soon = days_to.map(|d| d >= 0 && d <= 7).unwrap_or(false);
        let is_low_friend = friend.points < 250;

        if is_birthday_soon || is_low_friend {
            if let Some((item, count)) = found_love {
                suggestions.push(GiftSuggestion {
                    npc: friend.npc.clone(),
                    birthday,
                    friendship_points: friend.points,
                    days_to_birthday: days_to,
                    best_gift: item.to_string(),
                    gift_source: if state.inventory.iter().any(|i| i.name == item) { "背包" } else { "木箱" }.to_string(),
                    match_type: "最爱".to_string(),
                    available_count: count,
                });
            } else if let Some((item, count)) = found_like {
                suggestions.push(GiftSuggestion {
                    npc: friend.npc.clone(),
                    birthday,
                    friendship_points: friend.points,
                    days_to_birthday: days_to,
                    best_gift: item.to_string(),
                    gift_source: if state.inventory.iter().any(|i| i.name == item) { "背包" } else { "木箱" }.to_string(),
                    match_type: "喜欢".to_string(),
                    available_count: count,
                });
            } else if is_birthday_soon {
                let loves_str = loves.join(", ");
                let likes_str = likes.join(", ");
                suggestions.push(GiftSuggestion {
                    npc: friend.npc.clone(),
                    birthday,
                    friendship_points: friend.points,
                    days_to_birthday: days_to,
                    best_gift: format!("（背包/木箱里没有{}最爱或喜欢的物品。最爱: {}, 喜欢: {}）", friend.npc, loves_str, likes_str),
                    gift_source: "无".to_string(),
                    match_type: "未找到".to_string(),
                    available_count: 0,
                });
            }
        }
    }

    suggestions.sort_by_key(|s| {
        let bday = s.days_to_birthday.unwrap_or(99);
        let priority = match s.match_type.as_str() {
            "最爱" => 0,
            "喜欢" => 1,
            _ => 2,
        };
        (bday, priority)
    });

    if suggestions.is_empty() {
        Ok("当前没有需要送礼的 NPC（近期无生日且好感度均≥250）。".to_string())
    } else {
        Ok(serde_json::to_string_pretty(&suggestions)?)
    }
}

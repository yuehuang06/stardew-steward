use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 从存档解析出的完整游戏状态
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameState {
    pub money: i32,
    pub date: GameDate,
    pub weather: Weather,
    pub skills: Skills,
    pub crops: Vec<CropStatus>,
    pub friendships: Vec<Friendship>,
    pub inventory: Vec<InventoryItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameDate {
    pub year: u32,
    pub season: String,
    pub day: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Weather {
    pub is_raining: bool,
    pub is_lightning: bool,
    pub tomorrow: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Skills {
    pub farming: u32,
    pub mining: u32,
    pub combat: u32,
    pub foraging: u32,
    pub fishing: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CropStatus {
    pub item_id: i32,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub current_phase: i32,
    pub total_phases: i32,
    pub days_to_harvest: i32,
    pub is_dead: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Friendship {
    pub npc: String,
    pub points: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InventoryItem {
    pub name: String,
    pub count: u32,
}

/// 物品 ID → 中文名 映射表（仅常见作物，后续从知识库动态加载）
fn item_id_to_name() -> HashMap<i32, &'static str> {
    [
        (24u32 as i32, "草莓"), // 24? no - these are just examples
    ].iter().map(|&(id, name)| (id, name)).collect()
}

// 星露谷作物 itemID 对照（从 wiki 数据整理）
fn crop_id_to_name(id: i32) -> String {
    match id {
        24 => "防风草".into(),
        188 => "花椰菜".into(),
        190 => "马铃薯".into(),
        192 => "花椰菜".into(),
        24 => "防风草".into(),
        242 => "太阳鱼".into(), // 不是作物
        252 => "大黄".into(),
        254 => "甜瓜".into(),
        256 => "番茄".into(),
        257 => "辣椒".into(),
        258 => "蓝莓".into(),
        260 => "啤酒花".into(),
        262 => "小麦".into(),
        264 => "萝卜".into(),
        266 => "茄子".into(),
        270 => "南瓜".into(),
        272 => "玉米".into(),
        276 => "茄子".into(),
        280 => "山药".into(),
        281 => "甜瓜".into(),
        282 => "蔓越莓".into(),
        283 => "向日葵".into(),
        284 => "红叶卷心菜".into(),
        300 => "苋菜".into(),
        304 => "辣椒".into(),
        416 => "蓝莓".into(),
        417 => "玉米".into(),
        418 => "茄子".into(),
        421 => "啤酒花".into(),
        433 => "咖啡豆".into(),
        591 => "甜宝石浆果".into(),
        833 => "姜".into(),
        834 => "茶叶".into(),
        _ => format!("未知作物#{}", id),
    }
}

impl GameState {
    pub fn mock() -> anyhow::Result<Self> {
        Ok(Self {
            money: 1200,
            date: GameDate { year: 1, season: "summer".into(), day: 15 },
            weather: Weather { is_raining: false, is_lightning: false, tomorrow: "Sun".into() },
            skills: Skills { farming: 6, mining: 2, combat: 1, foraging: 5, fishing: 4 },
            crops: vec![
                CropStatus { item_id: 258, name: "蓝莓".into(), x: 0.0, y: 0.0, current_phase: 1, total_phases: 5, days_to_harvest: 3, is_dead: false },
                CropStatus { item_id: 272, name: "玉米".into(), x: 1.0, y: 0.0, current_phase: 4, total_phases: 5, days_to_harvest: 0, is_dead: false },
                CropStatus { item_id: 304, name: "辣椒".into(), x: 2.0, y: 0.0, current_phase: 3, total_phases: 5, days_to_harvest: 1, is_dead: false },
            ],
            friendships: vec![
                Friendship { npc: "Abigail".into(), points: 106 },
                Friendship { npc: "Pierre".into(), points: 618 },
                Friendship { npc: "Emily".into(), points: 506 },
            ],
            inventory: vec![
                InventoryItem { name: "夏南瓜种子".into(), count: 5 },
                InventoryItem { name: "铜矿石".into(), count: 14 },
                InventoryItem { name: "啤酒花".into(), count: 6 },
            ],
        })
    }
}

use serde::{Deserialize, Serialize};

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
    pub chests: Vec<Chest>,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Chest {
    pub location: String,
    pub x: f32,
    pub y: f32,
    pub items: Vec<InventoryItem>,
}

impl GameState {
    pub fn mock() -> anyhow::Result<Self> {
        Ok(Self {
            money: 8640,
            date: GameDate { year: 1, season: "summer".into(), day: 15 },
            weather: Weather { is_raining: true, is_lightning: true, tomorrow: "Sun".into() },
            skills: Skills { farming: 6, mining: 2, combat: 1, foraging: 5, fishing: 4 },
            crops: vec![
                CropStatus { item_id: 258, name: "蓝莓".into(), x: 60.0, y: 27.0, current_phase: 5, total_phases: 6, days_to_harvest: 0, is_dead: false },
                CropStatus { item_id: 304, name: "辣椒".into(), x: 61.0, y: 26.0, current_phase: 5, total_phases: 6, days_to_harvest: 0, is_dead: false },
            ],
            friendships: vec![
                Friendship { npc: "Pierre".into(), points: 618 },
                Friendship { npc: "Abigail".into(), points: 106 },
            ],
            inventory: vec![
                InventoryItem { name: "Copper Ore".into(), count: 14 },
                InventoryItem { name: "Summer Squash Seeds".into(), count: 5 },
            ],
            chests: vec![
                Chest {
                    location: "Farm".into(),
                    x: 57.0, y: 17.0,
                    items: vec![
                        InventoryItem { name: "Copper Bar".into(), count: 6 },
                        InventoryItem { name: "Wood".into(), count: 60 },
                    ],
                },
            ],
        })
    }
}

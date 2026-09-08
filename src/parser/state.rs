use serde::{Deserialize, Serialize};

/// 从存档解析出的完整游戏状态
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameState {
    pub money: i32,
    pub date: GameDate,
    pub weather: Weather,
    /// 每日运气（占卜频道），-0.1 ~ +0.1，影响下矿掉落/钓鱼宝箱
    pub daily_luck: f64,
    pub skills: Skills,
    pub crops: Vec<CropStatus>,
    pub friendships: Vec<Friendship>,
    pub inventory: Vec<InventoryItem>,
    pub chests: Vec<Chest>,
    /// 当前进行中的任务
    #[serde(default)]
    pub quests: Vec<Quest>,
    /// 农场建筑统计（buildingType 中文映射）
    #[serde(default)]
    pub buildings: Vec<BuildingInfo>,
    /// Junimo 小屋数量（自动浇水）
    #[serde(default)]
    pub junimo_huts: u32,
    /// 洒水器数量（含增压喷嘴的）
    #[serde(default)]
    pub sprinklers: u32,
    /// 果树状态
    #[serde(default)]
    pub fruit_trees: Vec<FruitTreeStatus>,
    /// 树木状态
    #[serde(default)]
    pub trees: Vec<TreeStatus>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BuildingInfo {
    /// 原始 buildingType（如 "Deluxe Coop"）
    pub kind: String,
    /// 中文名（如 "高级鸡舍"）
    pub name: String,
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FruitTreeStatus {
    pub name: String,
    pub count: u32,
    pub mature: u32,
    /// 当前置信可摇取（fruitsOnTree > 0）
    pub ready: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TreeStatus {
    pub name: String,
    pub count: u32,
    pub mature: u32,
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
    /// 是否已浇水（HoeDirt state bit 0）
    #[serde(default)]
    pub watered: bool,
    /// 1.6 收获规则: currentPhase 在 99999 槽位 && fullGrown && dayOfCurrentPhase < 0
    #[serde(default)]
    pub harvestable: bool,
    /// 是否被洒水器覆盖（6 点自动浇水，无需手浇）
    #[serde(default)]
    pub sprinkler_covered: bool,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Quest {
    pub title: String,
    pub description: String,
    pub objective: String,
    pub completed: bool,
    pub money_reward: i32,
    pub days_left: i32,
    pub quest_type: String,
    /// 物品交付任务的目标 NPC
    pub target: Option<String>,
    /// 需要的物品 ID (如 "(O)303")
    pub item: Option<String>,
    pub number: Option<i32>,
}

impl GameState {
    pub fn mock() -> anyhow::Result<Self> {
        Ok(Self {
            money: 8640,
            date: GameDate { year: 1, season: "summer".into(), day: 15 },
            weather: Weather { is_raining: true, is_lightning: true, tomorrow: "Sun".into() },
            daily_luck: 0.054,
            skills: Skills { farming: 6, mining: 2, combat: 1, foraging: 5, fishing: 4 },
            crops: vec![
                CropStatus { item_id: 258, name: "蓝莓".into(), x: 60.0, y: 27.0, current_phase: 5, total_phases: 6, days_to_harvest: 0, is_dead: false, watered: true, harvestable: true, sprinkler_covered: false },
                CropStatus { item_id: 304, name: "辣椒".into(), x: 61.0, y: 26.0, current_phase: 5, total_phases: 6, days_to_harvest: 0, is_dead: false, watered: false, harvestable: true, sprinkler_covered: false },
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
            quests: vec![
                Quest {
                    title: "矿场深处".into(),
                    description: "深入矿场探索".into(),
                    objective: "在矿场中达到 40 层".into(),
                    completed: false,
                    money_reward: 0,
                    days_left: 0,
                    quest_type: "story".into(),
                    target: None,
                    item: None,
                    number: None,
                },
            ],
            buildings: vec![
                BuildingInfo { kind: "Barn".into(), name: "畜棚".into(), count: 1 },
            ],
            junimo_huts: 0,
            sprinklers: 0,
            fruit_trees: vec![
                FruitTreeStatus { name: "苹果".into(), count: 1, mature: 1, ready: 0 },
            ],
            trees: vec![
                TreeStatus { name: "橡树".into(), count: 3, mature: 2 },
            ],
        })
    }
}

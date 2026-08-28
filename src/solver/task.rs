use serde::{Serialize, Deserialize};

/// 一个可选任务（浇蓝莓 / 送 Abigail 礼物 / 下矿...）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub name: String,
    pub time_cost: f32,        // 游戏小时数
    pub energy_cost: i32,      // 体力
    pub money_gain: i32,       // 预计收益（负数=花钱）
    pub priority: Priority,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Priority {
    Must,    // 必做（浇水）
    Should,  // 应该做（明天作物要熟了）
    Could,   // 可以做（下矿、钓鱼）
}

/// 排好的日程
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Schedule {
    pub tasks: Vec<Task>,
    pub total_time: f32,
    pub total_energy: i32,
    pub estimated_income: i32,
}

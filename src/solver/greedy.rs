use super::task::{Task, Schedule, Priority};

/// 贪心求解器: 按性价比排序，在时间/体力约束内贪心选择
///
/// 这是 MVP 版本 — 简单可靠，后续可升级为回溯搜索 / 并行多策略 (rayon)
pub fn solve(tasks: Vec<Task>, time_budget: f32, energy_budget: i32) -> Schedule {
    // 1. Must 优先级先全选
    // 2. 剩余预算按 (money_gain / time_cost) 排序贪心
    // 3. 累计时间、体力、预计收入

    let mut selected: Vec<Task> = tasks.iter()
        .filter(|t| t.priority == Priority::Must)
        .cloned()
        .collect();

    let mut used_time: f32 = selected.iter().map(|t| t.time_cost).sum();
    let mut used_energy: i32 = selected.iter().map(|t| t.energy_cost).sum();

    let mut optional: Vec<&Task> = tasks.iter()
        .filter(|t| t.priority != Priority::Must)
        .collect();
    optional.sort_by(|a, b| {
        let ra = a.money_gain as f32 / a.time_cost.max(0.1);
        let rb = b.money_gain as f32 / b.time_cost.max(0.1);
        rb.partial_cmp(&ra).unwrap_or(std::cmp::Ordering::Equal)
    });

    for t in optional {
        if used_time + t.time_cost <= time_budget
            && used_energy + t.energy_cost <= energy_budget
        {
            used_time += t.time_cost;
            used_energy += t.energy_cost;
            selected.push(t.clone());
        }
    }

    let estimated_income: i32 = selected.iter().map(|t| t.money_gain).sum();

    Schedule {
        tasks: selected,
        total_time: used_time,
        total_energy: used_energy,
        estimated_income,
    }
}

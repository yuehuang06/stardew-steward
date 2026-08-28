use crate::solver::{task::Task, greedy};

/// 工具: solve_schedule
/// 接收任务列表，调用求解器输出最优日程
pub fn execute(tasks: Vec<Task>, time_budget: f32, energy_budget: i32) -> anyhow::Result<String> {
    let schedule = greedy::solve(tasks, time_budget, energy_budget);
    let json = serde_json::to_string_pretty(&schedule)?;
    Ok(json)
}

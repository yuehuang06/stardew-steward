// 校验闭环: LLM 输出日程 JSON 后，用硬编码游戏规则检查合法性
// 校验失败 → 自动拼错误信息回 Prompt → LLM 重试
// 确定性保障，不依赖 LLM 运气

use crate::solver::DailySchedule;

pub struct Validator {
    money: i32,
    time_budget: f32,
}

impl Validator {
    pub fn new(money: i32, time_budget: f32) -> Self {
        Self { money, time_budget }
    }

    /// 校验日程，返回错误列表（空=通过）
    pub fn check(&self, schedule: &DailySchedule) -> Vec<String> {
        let mut errors = Vec::new();

        // 1. 资金检查: 推荐花费不能超过玩家现有资金
        if schedule.total_cost > self.money {
            errors.push(format!(
                "推荐的花费 {}g 超过玩家当前资金 {}g",
                schedule.total_cost, self.money
            ));
        }

        // 2. 时长检查: 总耗时不能超过一天的游戏时间
        if schedule.total_time > self.time_budget {
            errors.push(format!(
                "日程总耗时 {:.1}h 超过一天可用 {:.1}h",
                schedule.total_time, self.time_budget
            ));
        }

        // 3. 各任务耗时合理性: 单个任务不能超过总预算
        for task in &schedule.tasks {
            if task.time_cost > self.time_budget {
                errors.push(format!(
                    "任务「{}」耗时 {:.1}h 超过一天总预算",
                    task.description, task.time_cost
                ));
            }
        }

        // 4. 优先级合法性
        let valid_priorities = ["must", "should", "could"];
        for task in &schedule.tasks {
            if !valid_priorities.contains(&task.priority.as_str()) {
                errors.push(format!(
                    "任务「{}」的优先级「{}」无效，应为 must/should/could",
                    task.description, task.priority
                ));
            }
        }

        // 5. 必做任务不能为空
        let has_must = schedule.tasks.iter().any(|t| t.priority == "must");
        if !has_must && !schedule.tasks.is_empty() {
            errors.push("日程中没有任何 must 优先级的任务（至少应该有浇水或收获）".into());
        }

        errors
    }
}

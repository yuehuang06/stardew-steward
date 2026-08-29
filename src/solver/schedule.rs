use serde::{Deserialize, Serialize};

/// LLM 最终输出的日程 JSON
/// 校验器检查这个结构，通过后用模板渲染成 Markdown
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DailySchedule {
    /// 一句话总结
    pub summary: String,
    /// 任务列表
    pub tasks: Vec<ScheduleTask>,
    /// 总预计耗时（游戏小时）
    pub total_time: f32,
    /// 总预计花费（g）
    pub total_cost: i32,
    /// 总预计收入（g）
    pub total_income: i32,
    /// 提醒事项
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScheduleTask {
    /// 动作类型: harvest / shop / gift / water / mine / fish / other
    pub action: String,
    /// 人类可读描述
    pub description: String,
    /// 预计耗时（游戏小时）
    pub time_cost: f32,
    /// 预计花费（g，正数=花钱，0=不花钱）
    pub cost: i32,
    /// 预计收入（g）
    pub income: i32,
    /// 优先级: must / should / could
    pub priority: String,
}

/// 把日程 JSON 渲染成排版好的 Markdown
pub fn render(schedule: &DailySchedule) -> String {
    let mut out = String::new();

    out.push_str(&format!("## 📋 {}\n\n", schedule.summary));

    out.push_str("| # | 任务 | 类型 | 耗时 | 花费 | 收入 | 优先级 |\n");
    out.push_str("|---|------|------|------|------|------|--------|\n");
    for (i, task) in schedule.tasks.iter().enumerate() {
        out.push_str(&format!(
            "| {} | {} | {} | {:.1}h | {}g | {}g | {} |\n",
            i + 1,
            task.description,
            action_emoji(&task.action),
            task.time_cost,
            task.cost,
            task.income,
            task.priority,
        ));
    }

    out.push_str(&format!(
        "\n**总计**: {:.1}h | 支出 {}g | 收入 {}g | 净收益 {}g\n",
        schedule.total_time,
        schedule.total_cost,
        schedule.total_income,
        schedule.total_income - schedule.total_cost,
    ));

    if !schedule.notes.is_empty() {
        out.push_str("\n**提醒**:\n");
        for note in &schedule.notes {
            out.push_str(&format!("- {}\n", note));
        }
    }

    out
}

fn action_emoji(action: &str) -> &'static str {
    match action {
        "harvest" => "🌾 收获",
        "shop" => "🛒 购物",
        "gift" => "🎁 送礼",
        "water" => "💧 浇水",
        "mine" => "⛏️ 下矿",
        "fish" => "🎣 钓鱼",
        "other" => "📋 其他",
        _ => "📋 其他",
    }
}

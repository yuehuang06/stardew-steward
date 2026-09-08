use stardew_steward::agent::ProgressReporter;

pub struct CliReporter;

impl CliReporter {
    pub fn new() -> Self {
        Self
    }
}

impl ProgressReporter for CliReporter {
    fn on_step(&self, message: &str) {
        println!("  → {}", message);
    }

    fn on_done(&self) {
        println!("  ✓ 完成");
    }

    fn on_thinking(&self, message: &str) {
        println!("  💭 {}", message);
    }

    fn on_error(&self, message: &str) {
        println!("  ✗ {}", message);
    }
}

pub fn print_welcome() {
    println!();
    println!("🌾 星露谷农场管家 v0.1.0");
    println!("──────────────────────────────────");
    println!("输入你的问题，或输入 /quit 退出");
    println!("──────────────────────────────────");
}

pub fn print_usage(summary: &str) {
    println!("📊 {}", summary);
}

pub fn print_response(text: &str) {
    println!();
    // Try to parse as schedule JSON and render
    let json_str = text.trim();
    let json_str = if json_str.starts_with("```") {
        if let Some(start) = json_str.find('{') {
            if let Some(end) = json_str.rfind('}') {
                &json_str[start..=end]
            } else {
                text
            }
        } else {
            text
        }
    } else if json_str.starts_with('{') {
        json_str
    } else {
        text
    };

    match serde_json::from_str::<stardew_steward::solver::schedule::DailySchedule>(json_str) {
        Ok(schedule) => {
            let md = stardew_steward::solver::schedule::render(&schedule);
            termimad::print_text(&md);
        }
        Err(_) => {
            // 普通回答也走 termimad 渲染（表格/加粗/列表更美观）
            termimad::print_text(text);
        }
    }
    println!();
}

use crate::agent::ProgressReporter;

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
    println!("{}", text);
    println!();
}

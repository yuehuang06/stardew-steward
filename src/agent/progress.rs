pub trait ProgressReporter: Send + Sync {
    fn on_step(&self, message: &str);
    fn on_done(&self);
    fn on_thinking(&self, message: &str) {
        self.on_step(message);
    }
    fn on_error(&self, message: &str) {
        self.on_step(message);
    }
    /// 每次 LLM API 调用后触发（实时 token 用量，如 "¥0.05 · 14K/200K tok"）
    fn on_usage(&self, _brief: &str) {}
    fn is_interrupted(&self) -> bool {
        false
    }
}

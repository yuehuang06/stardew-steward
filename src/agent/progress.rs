pub trait ProgressReporter: Send + Sync {
    fn on_step(&self, message: &str);
    fn on_done(&self);
    fn on_thinking(&self, message: &str) {
        self.on_step(message);
    }
    fn on_error(&self, message: &str) {
        self.on_step(message);
    }
    fn is_interrupted(&self) -> bool {
        false
    }
}

pub trait ProgressReporter: Send + Sync {
    fn on_step(&self, message: &str);
    fn on_done(&self);
    fn is_interrupted(&self) -> bool {
        false
    }
}

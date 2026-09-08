use stardew_steward::agent::ProgressReporter;
use tauri::{AppHandle, Emitter};

pub struct TauriReporter {
    app: AppHandle,
}

impl TauriReporter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl ProgressReporter for TauriReporter {
    fn on_step(&self, message: &str) {
        let _ = self.app.emit("agent-step", message);
    }

    fn on_done(&self) {
        let _ = self.app.emit("agent-done", ());
    }

    fn on_thinking(&self, message: &str) {
        let _ = self.app.emit("agent-thinking", message);
    }

    fn on_error(&self, message: &str) {
        let _ = self.app.emit("agent-error", message);
    }

    fn on_usage(&self, brief: &str) {
        let _ = self.app.emit("agent-usage", brief);
    }
}

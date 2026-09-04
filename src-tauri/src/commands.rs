use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use stardew_steward::agent::Agent;
use stardew_steward::agent::session;
use stardew_steward::parser;
use stardew_steward::tools::read_save;
use tauri::State;
use tokio::sync::Mutex;

pub struct AppState {
    pub agent: Mutex<Agent>,
    pub interrupt_flag: Arc<AtomicBool>,
    pub save_path: String,
}

#[tauri::command]
pub async fn chat(state: State<'_, AppState>, message: String) -> Result<String, String> {
    state
        .interrupt_flag
        .store(false, std::sync::atomic::Ordering::SeqCst);
    let mut agent = state.agent.lock().await;
    agent.run(&message).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_usage(state: State<'_, AppState>) -> Result<String, String> {
    let agent = state.agent.lock().await;
    Ok(agent.usage_summary())
}

#[tauri::command]
pub async fn get_usage_brief(state: State<'_, AppState>) -> Result<String, String> {
    let agent = state.agent.lock().await;
    Ok(agent.usage_brief())
}

#[tauri::command]
pub async fn get_save_status(state: State<'_, AppState>) -> Result<String, String> {
    read_save::execute(&state.save_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn parse_save(path: String) -> Result<String, String> {
    parser::parse(std::path::Path::new(&path))
        .map(|s| serde_json::to_string_pretty(&s).unwrap_or_default())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_session(state: State<'_, AppState>, name: String) -> Result<String, String> {
    let agent = state.agent.lock().await;
    agent.save_session(&name).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_session(
    state: State<'_, AppState>,
    name: String,
) -> Result<(usize, usize), String> {
    let mut agent = state.agent.lock().await;
    agent.load_session(&name).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_sessions() -> Result<Vec<SessionInfo>, String> {
    let list = session::list_sessions();
    Ok(list
        .into_iter()
        .map(|(name, ts, count, rounds)| SessionInfo {
            name,
            saved_at: ts,
            message_count: count,
            interaction_rounds: rounds,
        })
        .collect())
}

#[derive(serde::Serialize)]
pub struct SessionInfo {
    pub name: String,
    pub saved_at: u64,
    pub message_count: usize,
    pub interaction_rounds: usize,
}

#[tauri::command]
pub async fn get_trajectory(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let agent = state.agent.lock().await;
    Ok(agent.trajectory())
}

#[tauri::command]
pub async fn interrupt(state: State<'_, AppState>) -> Result<(), String> {
    state
        .interrupt_flag
        .store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn toggle_window_width(window: tauri::WebviewWindow) -> Result<bool, String> {
    let cur = window.inner_size().map_err(|e| e.to_string())?;
    let expanded = cur.width <= 300;
    let new_w = if expanded { 480 } else { 280 };
    window
        .set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: new_w,
            height: cur.height,
        }))
        .map_err(|e| e.to_string())?;
    Ok(expanded)
}

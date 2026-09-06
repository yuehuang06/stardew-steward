use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use stardew_steward::agent::Agent;
use stardew_steward::agent::session;
use stardew_steward::config;
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
    let result = agent.run(&message).await.map_err(|e| e.to_string());
    // Auto-save session after each interaction round
    let _ = agent.auto_save_session();
    result
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
) -> Result<LoadResult, String> {
    let mut agent = state.agent.lock().await;
    let (count, rounds, usage) = agent.load_session(&name).map_err(|e| e.to_string())?;
    Ok(LoadResult {
        message_count: count,
        interaction_rounds: rounds,
        session_input_tokens: usage.input_tokens,
        session_output_tokens: usage.output_tokens,
        session_cost: usage.cost,
    })
}

#[derive(serde::Serialize)]
pub struct LoadResult {
    pub message_count: usize,
    pub interaction_rounds: usize,
    pub session_input_tokens: u64,
    pub session_output_tokens: u64,
    pub session_cost: f64,
}

#[tauri::command]
pub async fn new_session(state: State<'_, AppState>) -> Result<(), String> {
    let mut agent = state.agent.lock().await;
    agent.new_session();
    Ok(())
}

#[tauri::command]
pub async fn delete_session(name: String) -> Result<(), String> {
    Agent::delete_session(&name).map_err(|e| e.to_string())
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
pub async fn get_messages(state: State<'_, AppState>) -> Result<Vec<ChatMsg>, String> {
    let agent = state.agent.lock().await;
    Ok(agent
        .chat_messages()
        .into_iter()
        .map(|(role, text)| ChatMsg { role, text })
        .collect())
}

#[derive(serde::Serialize)]
pub struct ChatMsg {
    pub role: String,
    pub text: String,
}

#[tauri::command]
pub async fn toggle_always_on_top(window: tauri::WebviewWindow) -> Result<bool, String> {
    let cur = window
        .is_always_on_top()
        .map_err(|e| e.to_string())?;
    let next = !cur;
    window
        .set_always_on_top(next)
        .map_err(|e| e.to_string())?;
    Ok(next)
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
    let scale = window.scale_factor().unwrap_or(1.0);
    let cur = window.inner_size().map_err(|e| e.to_string())?;
    let cur_logical_w = (cur.width as f64 / scale) as f64;
    let cur_logical_h = (cur.height as f64 / scale) as f64;
    let expanded = cur_logical_w <= 300.0;
    let start_w = cur_logical_w;
    let end_w = if expanded { 480.0 } else { 280.0 };
    let steps = 15;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let eased = 1.0 - (1.0 - t).powi(3);
        let w = start_w + (end_w - start_w) * eased;
        let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(w, cur_logical_h)));
        tokio::time::sleep(std::time::Duration::from_millis(12)).await;
    }
    Ok(expanded)
}

#[derive(serde::Serialize)]
pub struct UsageDetail {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub budget: u64,
    pub cost: f64,
    pub price_input: f64,
    pub price_output: f64,
}

#[tauri::command]
pub async fn get_usage_detail(state: State<'_, AppState>) -> Result<UsageDetail, String> {
    let agent = state.agent.lock().await;
    let (input, output, budget, cost) = agent.usage_detail();
    let cfg = agent.config();
    Ok(UsageDetail {
        input_tokens: input,
        output_tokens: output,
        budget,
        cost,
        price_input: cfg.model.price_input,
        price_output: cfg.model.price_output,
    })
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let agent = state.agent.lock().await;
    let cfg = agent.config();
    let mut json = serde_json::to_value(cfg).map_err(|e| e.to_string())?;
    // Mask api_key — only show whether it's set
    if let Some(key) = json["model"]["api_key"].as_str() {
        let masked = if key.is_empty() {
            String::new()
        } else {
            let chars: Vec<char> = key.chars().collect();
            let head: String = chars.iter().take(4).collect();
            let tail: String = chars.iter().rev().take(4).collect();
            format!("{}...{}", head, tail.chars().rev().collect::<String>())
        };
        json["model"]["api_key"] = serde_json::Value::String(masked);
    }
    Ok(json)
}

#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    endpoint: String,
    #[allow(non_snake_case)]
    apiKey: String,
    model: String,
    context_length: usize,
    thinking_mode: bool,
    price_input: f64,
    price_output: f64,
) -> Result<(), String> {
    let mut agent = state.agent.lock().await;
    let mut new_model = agent.config().model.clone();
    new_model.endpoint = endpoint;
    let key_changed = !apiKey.is_empty() && !apiKey.contains("...");
    if key_changed {
        new_model.api_key = apiKey;
    }
    new_model.model = model;
    new_model.context_length = context_length;
    new_model.thinking_mode = thinking_mode;
    new_model.price_input = price_input;
    new_model.price_output = price_output;

    agent.update_model_config(new_model.clone());

    let mut cfg = agent.config().clone();
    cfg.model = new_model;
    config::save(&cfg).map_err(|e| {
        format!("保存配置失败: {} (app_data_dir: {:?})", e, config::app_data_dir())
    })?;

    Ok(())
}

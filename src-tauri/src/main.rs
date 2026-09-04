mod commands;
mod reporter;

use std::sync::Arc;

use commands::AppState;
use reporter::TauriReporter;
use stardew_steward::{agent::Agent, app, config, knowledge, validator};
use tauri::Manager;
use tokio::sync::Mutex;

fn main() {
    let config = config::load().expect("加载配置失败");
    let save_path = config.save.path.clone();

    let kb = Arc::new(std::sync::Mutex::new(
        knowledge::KnowledgeBase::open(&config.knowledge.db_path).expect("知识库打开失败"),
    ));
    let tools = app::build_tools(&save_path, Arc::clone(&kb));
    let system_prompt = app::build_system_prompt(&save_path);

    let interrupt_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));

    tauri::Builder::default()
        .setup({
            let config = config.clone();
            let save_path = save_path.clone();
            let interrupt_flag = Arc::clone(&interrupt_flag);
            move |app| {
                let reporter = Arc::new(TauriReporter::new(app.handle().clone()));

                let mut agent = Agent::new(config, tools, reporter, system_prompt);
                agent.set_interrupt_flag(Arc::clone(&interrupt_flag));

                if let Ok(state) =
                    stardew_steward::parser::parse(std::path::Path::new(&save_path))
                {
                    agent.set_validator(validator::Validator::new(
                        state.money,
                        &state.date.season,
                        12.0,
                    ));
                }

                let app_state = AppState {
                    agent: Mutex::new(agent),
                    interrupt_flag,
                    save_path,
                };
                app.manage(app_state);
                Ok(())
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::chat,
            commands::get_usage,
            commands::get_usage_brief,
            commands::get_usage_detail,
            commands::get_config,
            commands::update_config,
            commands::get_save_status,
            commands::parse_save,
            commands::save_session,
            commands::load_session,
            commands::new_session,
            commands::delete_session,
            commands::list_sessions,
            commands::get_trajectory,
            commands::get_messages,
            commands::interrupt,
            commands::toggle_window_width,
            commands::toggle_always_on_top,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

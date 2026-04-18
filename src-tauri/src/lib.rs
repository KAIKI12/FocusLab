//! FocusLab 后端 library crate root.
//!
//! 模块划分对齐 docs/04 §5。

pub mod ai;
pub mod commands;
pub mod db;
pub mod models;
pub mod services;
pub mod system;
pub mod utils;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化 tracing 日志 — RUST_LOG 环境变量可覆盖,默认 info
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("focuslab=info,warn")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle();
            let db = db::init(handle)?;
            app.manage(db);
            tracing::info!("FocusLab started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::task_commands::list_tasks,
            commands::task_commands::create_task,
            commands::task_commands::complete_task,
            commands::timer_commands::get_timer_state,
            commands::timer_commands::update_timer_state,
            commands::timer_commands::reset_timer_state,
            commands::recovery_commands::check_crash_recovery,
            commands::assignment_commands::list_assignments,
            commands::assignment_commands::create_assignment,
            commands::assignment_commands::update_assignment_status,
            commands::assignment_commands::remove_assignment,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

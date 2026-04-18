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

            // 番茄钟服务:持有 AppHandle 以便 emit 事件 + 访问 Db State
            let timer_service = services::timer_service::TimerService::new(handle.clone());
            app.manage(timer_service);

            tracing::info!("FocusLab started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // task CRUD (Week 1a)
            commands::task_commands::list_tasks,
            commands::task_commands::create_task,
            commands::task_commands::complete_task,
            commands::task_commands::update_task,
            commands::task_commands::delete_task,
            commands::task_commands::get_task_name,
            // timer_state 低层 CRUD (Week 1b)
            commands::timer_commands::get_timer_state,
            commands::timer_commands::update_timer_state,
            commands::timer_commands::reset_timer_state,
            commands::recovery_commands::check_crash_recovery,
            // DTA CRUD (Week 1b)
            commands::assignment_commands::list_assignments,
            commands::assignment_commands::create_assignment,
            commands::assignment_commands::update_assignment_status,
            commands::assignment_commands::remove_assignment,
            // 计划锁定 + 完成率 (Week 2b)
            commands::assignment_commands::lock_plan,
            commands::assignment_commands::get_completion_stats,
            // 番茄钟控制 (Week 2a)
            commands::focus_commands::start_pomodoro,
            commands::focus_commands::pause_timer,
            commands::focus_commands::resume_timer,
            commands::focus_commands::abandon_timer,
            commands::focus_commands::skip_break,
            commands::focus_commands::resume_from_crash,
            commands::focus_commands::abandon_from_crash,
            // 休息三选一 + 自由模式 (Week 2b)
            commands::focus_commands::continue_after_break,
            commands::focus_commands::switch_task_after_break,
            commands::focus_commands::extend_break,
            commands::focus_commands::start_free,
            commands::focus_commands::complete_free,
            // 中断记录 (Week 2b)
            commands::interruption_commands::create_interruption,
            commands::interruption_commands::end_interruption,
            // 长线目标 + 里程碑 (Phase 2)
            commands::goal_commands::list_goals,
            commands::goal_commands::create_goal,
            commands::goal_commands::update_goal,
            commands::goal_commands::archive_goal,
            commands::goal_commands::list_milestones,
            commands::goal_commands::create_milestone,
            commands::goal_commands::update_milestone,
            commands::goal_commands::complete_milestone,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

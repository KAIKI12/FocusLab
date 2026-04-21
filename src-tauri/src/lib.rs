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

            // AI 服务:启动时尝试从 settings 读配置
            let ai_service = ai::AIService::new();
            {
                let db_state = app.state::<db::Db>();
                let conn = db_state.0.lock().unwrap();
                let get = |key: &str| -> String {
                    conn.query_row(
                        "SELECT value FROM settings WHERE key = ?1",
                        rusqlite::params![key],
                        |r| r.get(0),
                    )
                    .unwrap_or_default()
                };
                let provider = get("ai_provider");
                let base_url = get("ai_base_url");
                let api_key = get("ai_api_key");
                let model = get("ai_model");
                if !api_key.is_empty() || provider == "ollama" {
                    let ai_ref = &ai_service;
                    tauri::async_runtime::block_on(async {
                        ai_ref.configure(&provider, &base_url, &api_key, &model).await;
                    });
                    tracing::info!("AI auto-configured from settings: provider={provider}");
                }
            }
            app.manage(ai_service);

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
            commands::task_commands::generate_recurring_tasks,
            commands::task_commands::check_shelved_tasks,
            commands::task_commands::create_task_reflection,
            commands::task_commands::unshelve_task,
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
            commands::assignment_commands::pin_due_tasks,
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
            // 手动补录
            commands::focus_commands::create_manual_session,
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
            commands::goal_commands::set_milestone_target_date,
            commands::goal_commands::list_milestone_notes,
            commands::goal_commands::add_milestone_note,
            commands::goal_commands::delete_milestone_note,
            commands::goal_commands::get_goal_weekly_invest,
            commands::goal_commands::list_goal_weekly_invests,
            // 日结算 (Phase 2)
            commands::settlement_commands::settle_day,
            commands::settlement_commands::get_settlement,
            commands::settlement_commands::get_yesterday_summary,
            commands::settlement_commands::list_day_summaries,
            // AI (Phase 2)
            commands::ai_commands::configure_ai,
            commands::ai_commands::test_ai_connection,
            commands::ai_commands::ai_decompose_task,
            commands::ai_commands::ai_settlement_narrative,
            commands::ai_commands::ai_daily_suggestions,
            commands::ai_commands::ai_classify_quadrant,
            commands::ai_commands::ai_weekly_summary,
            // 数据导出
            commands::export_commands::export_tasks_json,
            commands::export_commands::export_sessions_csv,
            // 数据洞察
            commands::stats_commands::get_focus_heatmap,
            commands::stats_commands::get_completion_trend,
            commands::stats_commands::get_time_by_category,
            commands::stats_commands::get_stats_overview,
            // 窗口管理
            commands::window_commands::show_main_window,
            // 通用设置
            commands::settings_commands::get_setting,
            commands::settings_commands::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

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
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_global_shortcut::Builder::new().build())?;

            let handle = app.handle();
            let db = db::init(handle)?;
            app.manage(db);

            // 番茄钟服务:持有 AppHandle 以便 emit 事件 + 访问 Db State
            let timer_service = services::timer_service::TimerService::new(handle.clone());
            app.manage(timer_service);

            // AI 服务:启动时读 active chat / embedding profile,回退到旧 keys 兼容
            let ai_service = ai::AIService::new();
            {
                let db_state = app.state::<db::Db>();
                let conn = db_state.0.lock().unwrap();

                // chat: 优先 active profile,否则散 keys
                let active_chat = models::ai_profile::active_chat_id(&conn)
                    .ok()
                    .flatten()
                    .filter(|s| !s.is_empty())
                    .and_then(|id| models::ai_profile::get_chat(&conn, &id).ok().flatten());

                let (provider, api_format, base_url, api_key, model) = if let Some(p) = active_chat
                {
                    let primary = if !p.model_fast.is_empty() {
                        p.model_fast
                    } else {
                        p.model_strong
                    };
                    (p.provider, p.api_format, p.base_url, p.api_key, primary)
                } else {
                    let get = |key: &str| -> String {
                        conn.query_row(
                            "SELECT value FROM settings WHERE key = ?1",
                            rusqlite::params![key],
                            |r| r.get(0),
                        )
                        .unwrap_or_default()
                    };
                    (
                        get("ai_provider"),
                        get("ai_api_format"),
                        get("ai_base_url"),
                        get("ai_api_key"),
                        get("ai_model"),
                    )
                };

                if !api_key.is_empty() || provider == "ollama" {
                    let ai_ref = &ai_service;
                    tauri::async_runtime::block_on(async {
                        ai_ref
                            .configure(&provider, &api_format, &base_url, &api_key, &model)
                            .await;
                    });
                    tracing::info!(
                        "AI auto-configured: provider={provider} api_format={api_format}"
                    );
                }

                // embedding: 同样优先 active profile,旧 keys 由 user 触发首次激活时已镜像
                let active_emb = models::ai_profile::active_embedding_id(&conn)
                    .ok()
                    .flatten()
                    .filter(|s| !s.is_empty())
                    .and_then(|id| models::ai_profile::get_embedding(&conn, &id).ok().flatten());
                if let Some(e) = active_emb {
                    let ai_ref = &ai_service;
                    tauri::async_runtime::block_on(async {
                        ai_ref
                            .configure_embedding(&e.base_url, &e.api_key, &e.model)
                            .await;
                    });
                    tracing::info!("Embedding auto-configured from active profile");
                }
            }
            app.manage(ai_service);
            app.manage(commands::chat_commands::AbortRegistry::default());

            system::tray::build(handle)?;

            tracing::info!("FocusLab started");
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
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
            commands::task_commands::hard_delete_task,
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
            // 结算感想更新
            commands::settlement_commands::update_settlement_reflection,
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
            // 灵感工作台 / 图谱
            commands::inspiration_commands::list_inspirations,
            commands::inspiration_commands::create_inspiration,
            commands::inspiration_commands::update_inspiration_goal,
            commands::inspiration_commands::update_inspiration_verification,
            commands::inspiration_commands::delete_inspiration,
            commands::inspiration_commands::mark_inspiration_converted,
            commands::inspiration_commands::link_inspirations,
            commands::inspiration_commands::unlink_inspirations,
            commands::inspiration_commands::ignore_inspiration_recommendation,
            commands::inspiration_commands::list_inspiration_links,
            commands::inspiration_commands::suggest_related_inspirations,
            commands::inspiration_commands::migrate_inspirations_from_local,
            commands::inspiration_commands::batch_embed_pending,
            commands::inspiration_commands::retry_embed_inspiration,
            commands::inspiration_commands::extract_inspiration_keywords,
            commands::inspiration_commands::clear_inspirations_for_goal,
            // 日结算 (Phase 2)
            commands::settlement_commands::settle_day,
            commands::settlement_commands::get_settlement,
            commands::settlement_commands::list_day_summaries,
            commands::settlement_commands::get_yesterday_summary,
            commands::settlement_commands::check_unsettled_yesterday,
            commands::settlement_commands::get_persona_hatch_progress,
            // AI (Phase 2)
            commands::ai_commands::configure_ai,
            commands::ai_commands::configure_embedding,
            commands::ai_commands::test_ai_connection,
            commands::ai_commands::fetch_ai_models,
            commands::ai_commands::ai_decompose_task,
            commands::ai_commands::ai_settlement_narrative,
            commands::ai_commands::ai_daily_suggestions,
            commands::ai_commands::ai_classify_quadrant,
            commands::ai_commands::ai_optimize_quick_note,
            commands::ai_commands::ai_weekly_summary,
            commands::ai_commands::ai_unfinished_reminder,
            commands::ai_commands::ai_task_feedback,
            commands::ai_commands::ai_milestone_breakdown,
            commands::ai_commands::ai_estimate_task_duration,
            commands::ai_commands::ai_milestone_risk,
            commands::ai_commands::ai_suggest_goal_for_inspiration,
            commands::ai_commands::ai_draft_followup_experiment,
            commands::ai_commands::ai_analyze_correction,
            // AI Profile (chat / embedding 双独立池子)
            commands::ai_profile_commands::list_chat_profiles,
            commands::ai_profile_commands::list_embedding_profiles,
            commands::ai_profile_commands::get_active_chat_profile_id,
            commands::ai_profile_commands::get_active_embedding_profile_id,
            commands::ai_profile_commands::create_chat_profile,
            commands::ai_profile_commands::create_embedding_profile,
            commands::ai_profile_commands::update_chat_profile,
            commands::ai_profile_commands::update_embedding_profile,
            commands::ai_profile_commands::delete_chat_profile,
            commands::ai_profile_commands::delete_embedding_profile,
            commands::ai_profile_commands::activate_chat_profile,
            commands::ai_profile_commands::activate_embedding_profile,
            // AI 聊天 - 会话/消息 CRUD (批次 1)
            commands::chat_commands::list_conversations,
            commands::chat_commands::get_conversation,
            commands::chat_commands::list_chat_messages,
            commands::chat_commands::create_conversation,
            commands::chat_commands::rename_conversation,
            commands::chat_commands::set_conversation_model,
            commands::chat_commands::pin_conversation,
            commands::chat_commands::archive_conversation,
            commands::chat_commands::delete_conversation,
            commands::chat_commands::delete_all_conversations,
            commands::chat_commands::delete_chat_message,
            commands::chat_commands::ai_chat_is_configured,
            commands::chat_commands::send_chat_message,
            commands::chat_commands::abort_chat_message,
            // 数据导出
            commands::export_commands::export_tasks_json,
            commands::export_commands::export_sessions_csv,
            // 数据洞察
            commands::stats_commands::get_focus_heatmap,
            commands::stats_commands::get_completion_trend,
            commands::stats_commands::get_time_by_category,
            commands::stats_commands::get_stats_overview,
            commands::stats_commands::get_badge_extra_stats,
            // 窗口管理
            commands::window_commands::show_main_window,
            commands::window_commands::show_quick_add_window,
            commands::window_commands::show_quick_note_window,
            commands::window_commands::show_command_palette_window,
            // 通用设置
            commands::settings_commands::get_setting,
            commands::settings_commands::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

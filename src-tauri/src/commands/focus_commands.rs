//! 番茄钟控制命令 — 对接 TimerService + 手动补录。
//!
//! 命令均为 async,因为 TimerService 的内部状态锁是 tokio 异步锁。

use chrono::Utc;
use rusqlite::params;
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use crate::db::Db;
use crate::models::session::Session;
use crate::services::timer_service::{TimerService, TimerSnapshot};
use crate::utils::errors::{AppError, AppResult};

#[tauri::command]
pub async fn start_pomodoro(
    task_id: String,
    preset: String,
    timer: State<'_, TimerService>,
) -> AppResult<TimerSnapshot> {
    timer.start_pomodoro(task_id, preset).await
}

#[tauri::command]
pub async fn pause_timer(timer: State<'_, TimerService>) -> AppResult<TimerSnapshot> {
    timer.pause().await
}

#[tauri::command]
pub async fn resume_timer(timer: State<'_, TimerService>) -> AppResult<TimerSnapshot> {
    timer.resume().await
}

#[tauri::command]
pub async fn abandon_timer(
    reason: Option<String>,
    timer: State<'_, TimerService>,
) -> AppResult<()> {
    timer.abandon(reason).await
}

#[tauri::command]
pub async fn skip_break(timer: State<'_, TimerService>) -> AppResult<()> {
    timer.skip_break().await
}

#[tauri::command]
pub async fn resume_from_crash(timer: State<'_, TimerService>) -> AppResult<TimerSnapshot> {
    timer.resume_from_crash().await
}

#[tauri::command]
pub async fn abandon_from_crash(timer: State<'_, TimerService>) -> AppResult<()> {
    timer.abandon_from_crash().await
}

// ---------- Week 2b: 休息三选一 ----------

#[tauri::command]
pub async fn continue_after_break(timer: State<'_, TimerService>) -> AppResult<TimerSnapshot> {
    timer.continue_same_task().await
}

#[tauri::command]
pub async fn switch_task_after_break(
    task_id: String,
    timer: State<'_, TimerService>,
) -> AppResult<TimerSnapshot> {
    timer.switch_task(task_id).await
}

#[tauri::command]
pub async fn extend_break(
    extra_seconds: i64,
    timer: State<'_, TimerService>,
) -> AppResult<TimerSnapshot> {
    timer.extend_break(extra_seconds).await
}

// ---------- Week 2b: 自由模式 ----------

#[tauri::command]
pub async fn start_free(
    task_id: String,
    timer: State<'_, TimerService>,
) -> AppResult<TimerSnapshot> {
    timer.start_free(task_id).await
}

#[tauri::command]
pub async fn complete_free(timer: State<'_, TimerService>) -> AppResult<()> {
    timer.complete_free().await
}

// ---------- 手动补录 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualSessionInput {
    pub task_id: String,
    pub start_time: String,
    pub duration_minutes: i64,
    pub mode: Option<String>,
}

/// 手动补录一条已完成的专注记录。
#[tauri::command]
pub fn create_manual_session(input: ManualSessionInput, db: State<'_, Db>) -> AppResult<Session> {
    if input.duration_minutes <= 0 {
        return Err(AppError::Custom("时长必须大于 0".into()));
    }

    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;

    // 校验 task 存在
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1 AND shelved_at IS NULL",
            params![input.task_id],
            |r| r.get(0),
        )
        .unwrap_or(false);
    if !exists {
        return Err(AppError::Custom("任务不存在".into()));
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let mode = input.mode.as_deref().unwrap_or("free");

    // 计算 end_time
    let start = chrono::DateTime::parse_from_rfc3339(&input.start_time)
        .map_err(|e| AppError::Custom(format!("无效的起始时间: {e}")))?;
    let end = start + chrono::Duration::minutes(input.duration_minutes);
    let end_time = end.to_rfc3339();

    conn.execute(
        "INSERT INTO sessions
            (id, task_id, start_time, end_time, planned_duration_minutes,
             actual_duration_minutes, mode, status, is_manual_entry, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, 'completed', 1, ?7)",
        params![
            id,
            input.task_id,
            input.start_time,
            end_time,
            input.duration_minutes,
            mode,
            now,
        ],
    )?;

    Ok(Session {
        id: id.clone(),
        task_id: input.task_id,
        start_time: input.start_time,
        end_time: Some(end_time),
        planned_duration_minutes: Some(input.duration_minutes),
        actual_duration_minutes: Some(input.duration_minutes),
        mode: mode.to_string(),
        pomodoro_preset: None,
        status: "completed".into(),
        is_manual_entry: true,
        abandon_reason: None,
        created_at: now,
    })
}

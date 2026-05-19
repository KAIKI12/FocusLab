//! TimerState CRUD 命令。
//!
//! - `get_timer_state`  · 读 id='current' 单行
//! - `update_timer_state` · 按 patch 更新指定字段(None = 不动),`updated_at` 自动 now()
//! - `reset_timer_state` · 清空所有字段回到 idle

use chrono::Utc;
use rusqlite::{params, params_from_iter, types::Value};
use tauri::State;

use crate::db::Db;
use crate::models::timer_state::{TimerState, TimerStatePatch};
use crate::utils::errors::{AppError, AppResult};

fn row_to_state(row: &rusqlite::Row<'_>) -> rusqlite::Result<TimerState> {
    Ok(TimerState {
        task_id: row.get("task_id")?,
        session_id: row.get("session_id")?,
        start_time: row.get("start_time")?,
        elapsed_seconds: row.get("elapsed_seconds")?,
        planned_seconds: row.get("planned_seconds")?,
        mode: row.get("mode")?,
        pomodoro_preset: row.get("pomodoro_preset")?,
        status: row.get("status")?,
        pomodoro_count: row.get("pomodoro_count")?,
        is_break: row.get::<_, i64>("is_break")? != 0,
        break_remaining: row.get("break_remaining")?,
        updated_at: row.get("updated_at")?,
    })
}

#[tauri::command]
pub fn get_timer_state(db: State<'_, Db>) -> AppResult<TimerState> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let state = conn.query_row(
        "SELECT task_id, session_id, start_time, elapsed_seconds, planned_seconds,
                mode, pomodoro_preset, status, pomodoro_count, is_break,
                break_remaining, updated_at
           FROM timer_state
          WHERE id = 'current'",
        [],
        row_to_state,
    )?;
    Ok(state)
}

#[tauri::command]
pub fn update_timer_state(patch: TimerStatePatch, db: State<'_, Db>) -> AppResult<TimerState> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;

    let mut sets: Vec<String> = Vec::new();
    let mut vals: Vec<Value> = Vec::new();

    macro_rules! push {
        ($field:expr, $col:literal, $val:expr) => {
            if let Some(v) = $field {
                sets.push(format!("{} = ?", $col));
                vals.push(Value::from($val(v)));
            }
        };
    }

    push!(patch.task_id, "task_id", |v: String| v);
    push!(patch.session_id, "session_id", |v: String| v);
    push!(patch.start_time, "start_time", |v: String| v);
    push!(patch.elapsed_seconds, "elapsed_seconds", |v: i64| v);
    push!(patch.planned_seconds, "planned_seconds", |v: i64| v);
    push!(patch.mode, "mode", |v: String| v);
    push!(patch.pomodoro_preset, "pomodoro_preset", |v: String| v);
    push!(patch.status, "status", |v: String| v);
    push!(patch.pomodoro_count, "pomodoro_count", |v: i64| v);
    push!(patch.is_break, "is_break", |v: bool| if v {
        1i64
    } else {
        0
    });
    push!(patch.break_remaining, "break_remaining", |v: i64| v);

    // updated_at: 显式传入时保留(调试用);否则落 now()
    let now = Utc::now().to_rfc3339();
    let updated_at = patch.updated_at.unwrap_or_else(|| now.clone());
    sets.push("updated_at = ?".into());
    vals.push(Value::from(updated_at));

    if sets.is_empty() {
        // 按设计永远至少有 updated_at,这里不会触发
        return Err(AppError::Custom("patch 为空".into()));
    }

    let sql = format!(
        "UPDATE timer_state SET {} WHERE id = 'current'",
        sets.join(", ")
    );
    conn.execute(&sql, params_from_iter(vals.iter()))?;

    // 返回更新后的状态(再查一次,避免前端不一致)
    let state = conn.query_row(
        "SELECT task_id, session_id, start_time, elapsed_seconds, planned_seconds,
                mode, pomodoro_preset, status, pomodoro_count, is_break,
                break_remaining, updated_at
           FROM timer_state
          WHERE id = 'current'",
        [],
        row_to_state,
    )?;
    Ok(state)
}

#[tauri::command]
pub fn reset_timer_state(db: State<'_, Db>) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    conn.execute(
        "UPDATE timer_state
            SET task_id = NULL,
                session_id = NULL,
                start_time = NULL,
                elapsed_seconds = 0,
                planned_seconds = NULL,
                mode = NULL,
                pomodoro_preset = NULL,
                status = 'idle',
                pomodoro_count = 0,
                is_break = 0,
                break_remaining = NULL,
                updated_at = ?1
          WHERE id = 'current'",
        params![now],
    )?;
    Ok(())
}

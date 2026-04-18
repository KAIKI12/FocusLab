//! 中断记录命令 — 前端暂停时可选调用。

use serde::Deserialize;
use tauri::State;

use crate::db::Db;
use crate::models::interruption;
use crate::utils::errors::{AppError, AppResult};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInterruptionInput {
    pub session_id: String,
    pub reason: Option<String>,
    pub note: Option<String>,
}

#[tauri::command]
pub fn create_interruption(
    input: CreateInterruptionInput,
    db: State<'_, Db>,
) -> AppResult<String> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    interruption::create_interruption(
        &conn,
        &input.session_id,
        input.reason.as_deref(),
        input.note.as_deref(),
    )
}

#[tauri::command]
pub fn end_interruption(id: String, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    interruption::end_interruption(&conn, &id)
}

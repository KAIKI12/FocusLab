//! 通用设置 KV 读写命令 — 供前端读写 settings 表。

use tauri::State;

use crate::db::Db;
use crate::models::settings;
use crate::utils::errors::{AppError, AppResult};

#[tauri::command]
pub fn get_setting(key: String, db: State<'_, Db>) -> AppResult<Option<String>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    settings::get(&conn, &key)
}

#[tauri::command]
pub fn set_setting(key: String, value: String, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    settings::set(&conn, &key, &value)
}

//! 数据访问层。
//!
//! 用 `std::sync::Mutex<rusqlite::Connection>` 包单连接;MVP 阶段这足够用。
//! 有并发压力再换 `r2d2` 连接池或迁移到 `sqlx`。

use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::utils::errors::{AppError, AppResult};

pub mod migrator;

pub struct Db(pub Mutex<Connection>);

/// 打开 / 创建数据库,执行迁移,返回 `Db`(以 `.manage()` 注入 Tauri State)。
///
/// Windows 位置: `%APPDATA%\com.focuslab.app\focuslab.db`
pub fn init(app: &AppHandle) -> AppResult<Db> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Custom(format!("cannot resolve app_data_dir: {e}")))?;
    std::fs::create_dir_all(&app_data)?;

    let db_path = app_data.join("focuslab.db");
    tracing::info!("SQLite at {}", db_path.display());

    let mut conn = Connection::open(&db_path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;

    migrator::run(&mut conn)?;

    Ok(Db(Mutex::new(conn)))
}

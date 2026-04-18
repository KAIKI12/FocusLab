//! 设置 KV 表读写辅助。
//!
//! `settings` 表是全局单键值存储,所有 value 都是 TEXT;本模块提供类型安全的
//! 取值 / 落盘入口。解析失败时按"看作未设置"处理,由调用方决定默认值。

use rusqlite::{params, Connection};

use crate::utils::datetime::DEFAULT_BOUNDARY_HOUR;
use crate::utils::errors::AppResult;

pub fn get(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    let value: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })?;
    Ok(value)
}

pub fn set(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO settings (key, value, updated_at)
         VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(key) DO UPDATE SET
            value = excluded.value,
            updated_at = excluded.updated_at",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_u32(conn: &Connection, key: &str) -> AppResult<Option<u32>> {
    Ok(get(conn, key)?.and_then(|s| s.parse::<u32>().ok()))
}

/// 读取 `day_boundary_hour`;缺失或非法时返回 [`DEFAULT_BOUNDARY_HOUR`] (= 4)。
pub fn get_boundary_hour(conn: &Connection) -> AppResult<u32> {
    Ok(get_u32(conn, "day_boundary_hour")?.unwrap_or(DEFAULT_BOUNDARY_HOUR))
}

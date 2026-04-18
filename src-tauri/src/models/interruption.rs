//! Interruption 模型 + 内部 helpers。
//!
//! 中断由前端在暂停时可选记录,TimerService 不强制要求。
//! 本模块提供 create / end 辅助函数,供 commands 层调用。

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::errors::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Interruption {
    pub id: String,
    pub session_id: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_minutes: Option<i64>,
    pub reason: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
}

/// 记录一次中断开始。返回生成的 id。
pub fn create_interruption(
    conn: &Connection,
    session_id: &str,
    reason: Option<&str>,
    note: Option<&str>,
) -> AppResult<String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO interruptions (id, session_id, start_time, reason, note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?3)",
        params![id, session_id, now, reason, note],
    )?;
    Ok(id)
}

/// 结束一次中断。计算 duration_minutes。
pub fn end_interruption(conn: &Connection, id: &str) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE interruptions
            SET end_time = ?1,
                duration_minutes = CAST(
                    (julianday(?1) - julianday(start_time)) * 24 * 60 AS INTEGER
                )
          WHERE id = ?2 AND end_time IS NULL",
        params![now, id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn mem_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE tasks (id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at DATETIME NOT NULL, updated_at DATETIME NOT NULL);
             CREATE TABLE sessions (
                id TEXT PRIMARY KEY, task_id TEXT NOT NULL REFERENCES tasks(id),
                start_time DATETIME NOT NULL, end_time DATETIME,
                planned_duration_minutes INTEGER, actual_duration_minutes INTEGER,
                mode TEXT NOT NULL DEFAULT 'pomodoro', pomodoro_preset TEXT,
                status TEXT NOT NULL DEFAULT 'in_progress', is_manual_entry BOOLEAN NOT NULL DEFAULT 0,
                abandon_reason TEXT, created_at DATETIME NOT NULL
             );
             CREATE TABLE interruptions (
                id TEXT PRIMARY KEY, session_id TEXT NOT NULL REFERENCES sessions(id),
                start_time DATETIME NOT NULL, end_time DATETIME,
                duration_minutes INTEGER, reason TEXT, note TEXT, created_at DATETIME NOT NULL
             );
             INSERT INTO tasks (id, name, created_at, updated_at) VALUES ('t1', 'test', datetime('now'), datetime('now'));
             INSERT INTO sessions (id, task_id, start_time, status, created_at) VALUES ('s1', 't1', datetime('now'), 'in_progress', datetime('now'));",
        )
        .unwrap();
        conn
    }

    #[test]
    fn create_and_end_interruption() {
        let conn = mem_db();
        let id = create_interruption(&conn, "s1", Some("phone_message"), None).unwrap();
        // end it
        end_interruption(&conn, &id).unwrap();
        let (end_time, reason): (Option<String>, Option<String>) = conn
            .query_row(
                "SELECT end_time, reason FROM interruptions WHERE id = ?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!(end_time.is_some());
        assert_eq!(reason.as_deref(), Some("phone_message"));
    }

    #[test]
    fn create_without_reason() {
        let conn = mem_db();
        let id = create_interruption(&conn, "s1", None, Some("just a note")).unwrap();
        let (reason, note): (Option<String>, Option<String>) = conn
            .query_row(
                "SELECT reason, note FROM interruptions WHERE id = ?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!(reason.is_none());
        assert_eq!(note.as_deref(), Some("just a note"));
    }
}

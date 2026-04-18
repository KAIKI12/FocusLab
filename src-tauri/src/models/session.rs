//! Session 模型 + 内部 helpers。
//!
//! sessions 生命周期由 TimerService 管理,**不直接暴露 CRUD 命令**;
//! 本模块提供 create / complete / abandon 的辅助函数,供 TimerService 调用。

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::errors::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub task_id: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub planned_duration_minutes: Option<i64>,
    pub actual_duration_minutes: Option<i64>,
    pub mode: String,
    pub pomodoro_preset: Option<String>,
    pub status: String,
    pub is_manual_entry: bool,
    pub abandon_reason: Option<String>,
    pub created_at: String,
}

/// 开始一个新 session,状态 `in_progress`。返回生成的 id。
pub fn create_session(
    conn: &Connection,
    task_id: &str,
    mode: &str,
    pomodoro_preset: Option<&str>,
    planned_duration_minutes: Option<i64>,
) -> AppResult<String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO sessions
            (id, task_id, start_time, planned_duration_minutes,
             mode, pomodoro_preset, status, is_manual_entry, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'in_progress', 0, ?3)",
        params![
            id,
            task_id,
            now,
            planned_duration_minutes,
            mode,
            pomodoro_preset,
        ],
    )?;
    Ok(id)
}

/// 正常完成 session。
pub fn complete_session(
    conn: &Connection,
    id: &str,
    actual_duration_minutes: i64,
) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE sessions
            SET status = 'completed',
                end_time = ?1,
                actual_duration_minutes = ?2
          WHERE id = ?3",
        params![now, actual_duration_minutes, id],
    )?;
    Ok(())
}

/// 放弃 session。reason 可选(Week 2b 接中断原因弹窗时填)。
pub fn abandon_session(
    conn: &Connection,
    id: &str,
    actual_duration_minutes: i64,
    reason: Option<&str>,
) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE sessions
            SET status = 'abandoned',
                end_time = ?1,
                actual_duration_minutes = ?2,
                abandon_reason = ?3
          WHERE id = ?4",
        params![now, actual_duration_minutes, reason, id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// 内存 SQLite + 最小 DDL(只开 tasks + sessions + 必要外键约束关闭,便于隔离)
    fn mem_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        // tasks 表是 sessions 的外键父,先建
        conn.execute_batch(
            "CREATE TABLE tasks (
                id TEXT PRIMARY KEY, name TEXT NOT NULL,
                quadrant TEXT DEFAULT 'important_not_urgent',
                status TEXT DEFAULT 'pending',
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL
             );
             CREATE TABLE sessions (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL REFERENCES tasks(id),
                start_time DATETIME NOT NULL,
                end_time DATETIME,
                planned_duration_minutes INTEGER,
                actual_duration_minutes INTEGER,
                mode TEXT NOT NULL DEFAULT 'pomodoro',
                pomodoro_preset TEXT,
                status TEXT NOT NULL DEFAULT 'in_progress',
                is_manual_entry BOOLEAN NOT NULL DEFAULT 0,
                abandon_reason TEXT,
                created_at DATETIME NOT NULL
             );",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tasks (id, name, created_at, updated_at)
             VALUES ('t1', 'test', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn create_session_inserts_row() {
        let conn = mem_db();
        let id = create_session(&conn, "t1", "pomodoro", Some("classic_25"), Some(25)).unwrap();
        let (status, mode, preset, planned): (String, String, Option<String>, Option<i64>) = conn
            .query_row(
                "SELECT status, mode, pomodoro_preset, planned_duration_minutes
                   FROM sessions WHERE id = ?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(status, "in_progress");
        assert_eq!(mode, "pomodoro");
        assert_eq!(preset.as_deref(), Some("classic_25"));
        assert_eq!(planned, Some(25));
    }

    #[test]
    fn complete_session_sets_end_and_actual() {
        let conn = mem_db();
        let id = create_session(&conn, "t1", "pomodoro", Some("classic_25"), Some(25)).unwrap();
        complete_session(&conn, &id, 25).unwrap();
        let (status, actual, end): (String, i64, Option<String>) = conn
            .query_row(
                "SELECT status, actual_duration_minutes, end_time
                   FROM sessions WHERE id = ?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(status, "completed");
        assert_eq!(actual, 25);
        assert!(end.is_some());
    }

    #[test]
    fn abandon_session_records_reason() {
        let conn = mem_db();
        let id = create_session(&conn, "t1", "pomodoro", Some("classic_25"), Some(25)).unwrap();
        abandon_session(&conn, &id, 12, Some("phone")).unwrap();
        let (status, actual, reason): (String, i64, Option<String>) = conn
            .query_row(
                "SELECT status, actual_duration_minutes, abandon_reason
                   FROM sessions WHERE id = ?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(status, "abandoned");
        assert_eq!(actual, 12);
        assert_eq!(reason.as_deref(), Some("phone"));
    }

    #[test]
    fn abandon_without_reason() {
        let conn = mem_db();
        let id = create_session(&conn, "t1", "free", None, None).unwrap();
        abandon_session(&conn, &id, 3, None).unwrap();
        let reason: Option<String> = conn
            .query_row(
                "SELECT abandon_reason FROM sessions WHERE id = ?1",
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(reason.is_none());
    }
}

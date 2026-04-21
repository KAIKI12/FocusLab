//! Milestone 模型 + 内部 helpers。

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::errors::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub goal_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub target_date: Option<String>,
}

pub fn list_milestones(conn: &Connection, goal_id: &str) -> AppResult<Vec<Milestone>> {
    let mut stmt = conn.prepare(
        "SELECT id, goal_id, name, description, status, sort_order, created_at, updated_at, completed_at, target_date
         FROM milestones WHERE goal_id = ?1 ORDER BY sort_order, created_at",
    )?;
    let rows = stmt
        .query_map(params![goal_id], |r| {
            Ok(Milestone {
                id: r.get("id")?,
                goal_id: r.get("goal_id")?,
                name: r.get("name")?,
                description: r.get("description")?,
                status: r.get("status")?,
                sort_order: r.get("sort_order")?,
                created_at: r.get("created_at")?,
                updated_at: r.get("updated_at")?,
                completed_at: r.get("completed_at")?,
                target_date: r.get("target_date")?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn create_milestone(conn: &Connection, goal_id: &str, name: &str, description: Option<&str>) -> AppResult<Milestone> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    // sort_order = 当前最大 +1
    let max_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM milestones WHERE goal_id = ?1",
            params![goal_id],
            |r| r.get(0),
        )
        .unwrap_or(-1);
    let order = max_order + 1;
    conn.execute(
        "INSERT INTO milestones (id, goal_id, name, description, status, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'pending', ?5, ?6, ?6)",
        params![id, goal_id, name, description, order, now],
    )?;
    Ok(Milestone {
        id,
        goal_id: goal_id.to_string(),
        name: name.to_string(),
        description: description.map(String::from),
        status: "pending".into(),
        sort_order: order,
        created_at: now.clone(),
        updated_at: now,
        completed_at: None,
        target_date: None,
    })
}

pub fn update_milestone(conn: &Connection, id: &str, name: Option<&str>, description: Option<&str>, status: Option<&str>) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    if let Some(n) = name {
        conn.execute("UPDATE milestones SET name = ?1, updated_at = ?2 WHERE id = ?3", params![n, now, id])?;
    }
    if let Some(d) = description {
        conn.execute("UPDATE milestones SET description = ?1, updated_at = ?2 WHERE id = ?3", params![d, now, id])?;
    }
    if let Some(s) = status {
        if s == "completed" {
            conn.execute(
                "UPDATE milestones SET status = ?1, completed_at = ?2, updated_at = ?2 WHERE id = ?3",
                params![s, now, id],
            )?;
        } else {
            conn.execute(
                "UPDATE milestones SET status = ?1, completed_at = NULL, updated_at = ?2 WHERE id = ?3",
                params![s, now, id],
            )?;
        }
    }
    Ok(())
}

pub fn complete_milestone(conn: &Connection, id: &str) -> AppResult<()> {
    update_milestone(conn, id, None, None, Some("completed"))
}

/// 设置预计完成日期。`date` 传 None 则清空(DATE NULL)。
pub fn set_milestone_target_date(conn: &Connection, id: &str, date: Option<&str>) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE milestones SET target_date = ?1, updated_at = ?2 WHERE id = ?3",
        params![date, now, id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE goals (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT,
                status TEXT DEFAULT 'active', target_date DATETIME,
                sort_order INTEGER DEFAULT 0,
                created_at DATETIME NOT NULL, updated_at DATETIME NOT NULL,
                completed_at DATETIME
             );
             CREATE TABLE milestones (
                id TEXT PRIMARY KEY,
                goal_id TEXT NOT NULL REFERENCES goals(id) ON DELETE CASCADE,
                name TEXT NOT NULL, description TEXT,
                status TEXT DEFAULT 'pending', sort_order INTEGER DEFAULT 0,
                created_at DATETIME NOT NULL, updated_at DATETIME NOT NULL,
                completed_at DATETIME,
                target_date DATE
             );
             INSERT INTO goals (id, name, created_at, updated_at) VALUES ('g1', 'Test Goal', datetime('now'), datetime('now'));",
        ).unwrap();
        conn
    }

    #[test]
    fn create_and_list_milestones() {
        let conn = mem_db();
        create_milestone(&conn, "g1", "MS1", None).unwrap();
        create_milestone(&conn, "g1", "MS2", Some("desc")).unwrap();
        let ms = list_milestones(&conn, "g1").unwrap();
        assert_eq!(ms.len(), 2);
        assert_eq!(ms[0].sort_order, 0);
        assert_eq!(ms[1].sort_order, 1);
    }

    #[test]
    fn complete_sets_completed_at() {
        let conn = mem_db();
        let m = create_milestone(&conn, "g1", "MS1", None).unwrap();
        complete_milestone(&conn, &m.id).unwrap();
        let (status, completed): (String, Option<String>) = conn
            .query_row(
                "SELECT status, completed_at FROM milestones WHERE id = ?1",
                params![m.id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(status, "completed");
        assert!(completed.is_some());
    }

    #[test]
    fn update_name_and_status() {
        let conn = mem_db();
        let m = create_milestone(&conn, "g1", "Draft", None).unwrap();
        update_milestone(&conn, &m.id, Some("Final"), None, Some("in_progress")).unwrap();
        let (name, status): (String, String) = conn
            .query_row(
                "SELECT name, status FROM milestones WHERE id = ?1",
                params![m.id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(name, "Final");
        assert_eq!(status, "in_progress");
    }

    #[test]
    fn target_date_round_trip() {
        let conn = mem_db();
        let m = create_milestone(&conn, "g1", "MS1", None).unwrap();
        assert!(m.target_date.is_none());
        set_milestone_target_date(&conn, &m.id, Some("2026-04-30")).unwrap();
        let listed = list_milestones(&conn, "g1").unwrap();
        assert_eq!(listed[0].target_date.as_deref(), Some("2026-04-30"));
        set_milestone_target_date(&conn, &m.id, None).unwrap();
        let listed2 = list_milestones(&conn, "g1").unwrap();
        assert!(listed2[0].target_date.is_none());
    }
}

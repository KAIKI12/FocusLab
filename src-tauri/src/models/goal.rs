//! Goal 模型 + 内部 helpers。

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::errors::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub target_date: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

pub fn list_goals(conn: &Connection, include_archived: bool) -> AppResult<Vec<Goal>> {
    let sql = if include_archived {
        "SELECT id, name, description, status, target_date, sort_order, created_at, updated_at, completed_at
         FROM goals ORDER BY sort_order, created_at DESC"
    } else {
        "SELECT id, name, description, status, target_date, sort_order, created_at, updated_at, completed_at
         FROM goals WHERE status != 'archived' ORDER BY sort_order, created_at DESC"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Goal {
                id: r.get("id")?,
                name: r.get("name")?,
                description: r.get("description")?,
                status: r.get("status")?,
                target_date: r.get("target_date")?,
                sort_order: r.get("sort_order")?,
                created_at: r.get("created_at")?,
                updated_at: r.get("updated_at")?,
                completed_at: r.get("completed_at")?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn create_goal(conn: &Connection, name: &str, description: Option<&str>, target_date: Option<&str>) -> AppResult<Goal> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO goals (id, name, description, target_date, status, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'active', 0, ?5, ?5)",
        params![id, name, description, target_date, now],
    )?;
    Ok(Goal {
        id,
        name: name.to_string(),
        description: description.map(String::from),
        status: "active".into(),
        target_date: target_date.map(String::from),
        sort_order: 0,
        created_at: now.clone(),
        updated_at: now,
        completed_at: None,
    })
}

pub fn update_goal(conn: &Connection, id: &str, name: Option<&str>, description: Option<&str>, target_date: Option<&str>) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    // 简单全字段 UPDATE(非 None 才覆盖)
    if let Some(n) = name {
        conn.execute("UPDATE goals SET name = ?1, updated_at = ?2 WHERE id = ?3", params![n, now, id])?;
    }
    if let Some(d) = description {
        conn.execute("UPDATE goals SET description = ?1, updated_at = ?2 WHERE id = ?3", params![d, now, id])?;
    }
    if let Some(t) = target_date {
        conn.execute("UPDATE goals SET target_date = ?1, updated_at = ?2 WHERE id = ?3", params![t, now, id])?;
    }
    Ok(())
}

pub fn archive_goal(conn: &Connection, id: &str) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE goals SET status = 'archived', updated_at = ?1 WHERE id = ?2",
        params![now, id],
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
             );",
        ).unwrap();
        conn
    }

    #[test]
    fn create_and_list() {
        let conn = mem_db();
        let g = create_goal(&conn, "Publish Paper", Some("SCI"), None).unwrap();
        assert_eq!(g.status, "active");
        let all = list_goals(&conn, false).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "Publish Paper");
    }

    #[test]
    fn archive_excludes_from_list() {
        let conn = mem_db();
        let g = create_goal(&conn, "Old Goal", None, None).unwrap();
        archive_goal(&conn, &g.id).unwrap();
        let active = list_goals(&conn, false).unwrap();
        assert_eq!(active.len(), 0);
        let all = list_goals(&conn, true).unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn update_name() {
        let conn = mem_db();
        let g = create_goal(&conn, "Draft", None, None).unwrap();
        update_goal(&conn, &g.id, Some("Final"), None, None).unwrap();
        let name: String = conn
            .query_row("SELECT name FROM goals WHERE id = ?1", params![g.id], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "Final");
    }
}

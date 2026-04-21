//! MilestoneNote 模型 + CRUD helpers。
//!
//! 里程碑上带日期的多条备注 / 科研日志。对齐 prototype/goals/milestones.html:546。

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::errors::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneNote {
    pub id: String,
    pub milestone_id: String,
    pub text: String,
    pub created_at: String,
}

pub fn list_notes(conn: &Connection, milestone_id: &str) -> AppResult<Vec<MilestoneNote>> {
    let mut stmt = conn.prepare(
        "SELECT id, milestone_id, text, created_at
         FROM milestone_notes WHERE milestone_id = ?1
         ORDER BY created_at DESC",
    )?;
    let rows = stmt
        .query_map(params![milestone_id], |r| {
            Ok(MilestoneNote {
                id: r.get("id")?,
                milestone_id: r.get("milestone_id")?,
                text: r.get("text")?,
                created_at: r.get("created_at")?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn add_note(conn: &Connection, milestone_id: &str, text: &str) -> AppResult<MilestoneNote> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO milestone_notes (id, milestone_id, text, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![id, milestone_id, text, now],
    )?;
    Ok(MilestoneNote {
        id,
        milestone_id: milestone_id.to_string(),
        text: text.to_string(),
        created_at: now,
    })
}

pub fn delete_note(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM milestone_notes WHERE id = ?1", params![id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE milestones (
                id TEXT PRIMARY KEY, goal_id TEXT, name TEXT, description TEXT,
                status TEXT, sort_order INTEGER,
                created_at DATETIME, updated_at DATETIME, completed_at DATETIME,
                target_date DATE
             );
             CREATE TABLE milestone_notes (
                id TEXT PRIMARY KEY,
                milestone_id TEXT NOT NULL REFERENCES milestones(id) ON DELETE CASCADE,
                text TEXT NOT NULL,
                created_at DATETIME NOT NULL
             );
             INSERT INTO milestones (id, goal_id, name, status, sort_order, created_at, updated_at)
             VALUES ('m1', 'g1', 'MS1', 'pending', 0, datetime('now'), datetime('now'));",
        )
        .unwrap();
        conn
    }

    #[test]
    fn add_and_list_desc_order() {
        let conn = mem_db();
        add_note(&conn, "m1", "first note").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        add_note(&conn, "m1", "second note").unwrap();
        let notes = list_notes(&conn, "m1").unwrap();
        assert_eq!(notes.len(), 2);
        // DESC by created_at: second 在前
        assert_eq!(notes[0].text, "second note");
        assert_eq!(notes[1].text, "first note");
    }

    #[test]
    fn delete_note_removes_row() {
        let conn = mem_db();
        let n = add_note(&conn, "m1", "to be removed").unwrap();
        delete_note(&conn, &n.id).unwrap();
        let notes = list_notes(&conn, "m1").unwrap();
        assert!(notes.is_empty());
    }

    #[test]
    fn cascade_delete_on_milestone() {
        let conn = mem_db();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        add_note(&conn, "m1", "note").unwrap();
        conn.execute("DELETE FROM milestones WHERE id = 'm1'", []).unwrap();
        let notes = list_notes(&conn, "m1").unwrap();
        assert!(notes.is_empty());
    }
}

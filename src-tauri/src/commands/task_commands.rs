//! Task 相关 #[tauri::command] — CRUD 全量。
//!
//! list_tasks / create_task / complete_task / update_task / delete_task
//! 字段映射对齐 docs/04 §7.2 `tasks` 表。

use chrono::Utc;
use rusqlite::params;
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use crate::db::Db;
use crate::models::task::Task;
use crate::utils::errors::{AppError, AppResult};

fn row_to_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get("id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        quadrant: row.get("quadrant")?,
        status: row.get("status")?,
        estimated_minutes: row.get("estimated_minutes")?,
        due_date: row.get("due_date")?,
        is_background: row.get("is_background")?,
        shelved_at: row.get("shelved_at")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        completed_at: row.get("completed_at")?,
    })
}

const SELECT_COLS: &str =
    "id, name, description, quadrant, status, estimated_minutes, due_date, is_background, shelved_at, created_at, updated_at, completed_at";

/// 列出任务。status_filter=None 时返回所有非 completed、非 shelved 的任务。
#[tauri::command]
pub fn list_tasks(
    status_filter: Option<String>,
    db: State<'_, Db>,
) -> AppResult<Vec<Task>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let mut stmt = if status_filter.is_some() {
        conn.prepare(&format!(
            "SELECT {SELECT_COLS} FROM tasks WHERE status = ?1 ORDER BY sort_order, created_at DESC"
        ))?
    } else {
        conn.prepare(&format!(
            "SELECT {SELECT_COLS} FROM tasks
             WHERE status IN ('pending', 'in_progress') AND shelved_at IS NULL
             ORDER BY sort_order, created_at DESC"
        ))?
    };

    let rows = if let Some(status) = status_filter {
        stmt.query_map(params![status], row_to_task)?
            .collect::<rusqlite::Result<Vec<_>>>()?
    } else {
        stmt.query_map([], row_to_task)?
            .collect::<rusqlite::Result<Vec<_>>>()?
    };

    Ok(rows)
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskInput {
    pub name: String,
    #[serde(default = "default_quadrant")]
    pub quadrant: String,
    pub recurrence_rule: Option<String>,
}

fn default_quadrant() -> String {
    "important_not_urgent".to_string()
}

/// 创建任务 — 只接最小必填字段,其他走 schema 默认值。
#[tauri::command]
pub fn create_task(input: CreateTaskInput, db: State<'_, Db>) -> AppResult<Task> {
    if input.name.trim().is_empty() {
        return Err(AppError::Custom("任务名不能为空".into()));
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;

    let is_recurring = input.recurrence_rule.is_some();

    conn.execute(
        "INSERT INTO tasks (id, name, quadrant, status, is_recurring, recurrence_rule, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'pending', ?4, ?5, ?6, ?6)",
        params![id, input.name, input.quadrant, is_recurring, input.recurrence_rule, now],
    )?;

    Ok(Task {
        id,
        name: input.name,
        description: None,
        quadrant: input.quadrant,
        status: "pending".into(),
        estimated_minutes: None,
        due_date: None,
        is_background: false,
        shelved_at: None,
        created_at: now.clone(),
        updated_at: now,
        completed_at: None,
    })
}

/// 标记任务完成。
#[tauri::command]
pub fn complete_task(id: String, db: State<'_, Db>) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;

    let affected = conn.execute(
        "UPDATE tasks
            SET status = 'completed',
                completed_at = ?1,
                updated_at = ?1
          WHERE id = ?2",
        params![now, id],
    )?;

    if affected == 0 {
        return Err(AppError::Custom(format!("task {id} not found")));
    }
    Ok(())
}

// ---------- Week 2b ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskInput {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub quadrant: Option<String>,
    pub estimated_minutes: Option<i64>,
    pub due_date: Option<String>,
    pub is_background: Option<bool>,
    pub milestone_id: Option<String>,
}

/// 部分更新任务字段。
#[tauri::command]
pub fn update_task(input: UpdateTaskInput, db: State<'_, Db>) -> AppResult<Task> {
    // 验证 name 不为空白
    if let Some(ref n) = input.name {
        if n.trim().is_empty() {
            return Err(AppError::Custom("任务名不能为空".into()));
        }
    }

    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let now = Utc::now().to_rfc3339();

    // 动态 SQL 构建
    let mut sets = vec!["updated_at = ?1".to_string()];
    let mut idx: usize = 2;
    macro_rules! push_set {
        ($field:expr, $col:expr) => {
            if $field.is_some() {
                sets.push(format!("{} = ?{}", $col, idx));
                idx += 1;
            }
        };
    }
    push_set!(input.name, "name");
    push_set!(input.description, "description");
    push_set!(input.quadrant, "quadrant");
    push_set!(input.estimated_minutes, "estimated_minutes");
    push_set!(input.due_date, "due_date");
    push_set!(input.is_background, "is_background");
    push_set!(input.milestone_id, "milestone_id");

    let sql = format!(
        "UPDATE tasks SET {} WHERE id = ?{} AND shelved_at IS NULL",
        sets.join(", "),
        idx
    );

    // 绑定参数
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];
    if let Some(ref v) = input.name {
        param_values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = input.description {
        param_values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = input.quadrant {
        param_values.push(Box::new(v.clone()));
    }
    if let Some(v) = input.estimated_minutes {
        param_values.push(Box::new(v));
    }
    if let Some(ref v) = input.due_date {
        param_values.push(Box::new(v.clone()));
    }
    if let Some(v) = input.is_background {
        param_values.push(Box::new(v));
    }
    if let Some(ref v) = input.milestone_id {
        param_values.push(Box::new(v.clone()));
    }
    param_values.push(Box::new(input.id.clone()));

    let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|b| b.as_ref()).collect();
    let affected = conn.execute(&sql, params_ref.as_slice())?;
    if affected == 0 {
        return Err(AppError::Custom(format!("task {} not found or already deleted", input.id)));
    }

    // 回读返回
    conn.query_row(
        &format!("SELECT {SELECT_COLS} FROM tasks WHERE id = ?1"),
        params![input.id],
        row_to_task,
    )
    .map_err(|e| AppError::Custom(e.to_string()))
}

/// 软删除(shelve)任务。
#[tauri::command]
pub fn delete_task(id: String, db: State<'_, Db>) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;

    let affected = conn.execute(
        "UPDATE tasks SET shelved_at = ?1, updated_at = ?1 WHERE id = ?2 AND shelved_at IS NULL",
        params![now, id],
    )?;
    if affected == 0 {
        return Err(AppError::Custom(format!("task {id} not found or already deleted")));
    }
    Ok(())
}

/// 为当日生成重复任务的 DTA 记录。
/// 规则格式: "daily" | "weekdays" | "weekly" | "monthly"
#[tauri::command]
pub fn generate_recurring_tasks(db: State<'_, Db>) -> AppResult<i64> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let weekday = chrono::Utc::now().format("%u").to_string(); // 1=Mon..7=Sun

    let mut stmt = conn.prepare(
        "SELECT id, recurrence_rule FROM tasks
         WHERE is_recurring = 1 AND recurrence_rule IS NOT NULL
         AND shelved_at IS NULL AND status != 'completed'",
    )?;

    let tasks: Vec<(String, String)> = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut count = 0i64;
    let now = Utc::now().to_rfc3339();

    for (task_id, rule) in &tasks {
        let should_create = match rule.as_str() {
            "daily" => true,
            "weekdays" => weekday != "6" && weekday != "7", // 排除周六日
            "weekly" => weekday == "1", // 每周一
            "monthly" => today.ends_with("-01"), // 每月1号
            _ => false,
        };

        if !should_create { continue; }

        // 检查今天是否已有该任务的 DTA
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM daily_task_assignments WHERE plan_date = ?1 AND task_id = ?2",
                params![today, task_id],
                |r| r.get(0),
            )
            .unwrap_or(false);

        if exists { continue; }

        let dta_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO daily_task_assignments (id, plan_date, task_id, is_planned, source, day_status, added_at, sort_order)
             VALUES (?1, ?2, ?3, 1, 'recurring', 'pending', ?4, 0)",
            params![dta_id, today, task_id, now],
        )?;
        count += 1;
    }

    Ok(count)
}

/// 根据 ID 获取任务名(轻量查询,供悬浮球窗口使用)。
#[tauri::command]
pub fn get_task_name(id: String, db: State<'_, Db>) -> AppResult<Option<String>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let name = conn
        .query_row(
            "SELECT name FROM tasks WHERE id = ?1",
            params![id],
            |r| r.get::<_, String>(0),
        )
        .ok();
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn mem_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE tasks (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT,
                quadrant TEXT DEFAULT 'important_not_urgent',
                status TEXT DEFAULT 'pending',
                estimated_minutes INTEGER, due_date DATE,
                is_background BOOLEAN DEFAULT 0, shelved_at DATETIME,
                created_at DATETIME NOT NULL, updated_at DATETIME NOT NULL,
                completed_at DATETIME, sort_order INTEGER DEFAULT 0
             );",
        )
        .unwrap();
        conn
    }

    fn insert_task(conn: &Connection, id: &str, name: &str) {
        conn.execute(
            "INSERT INTO tasks (id, name, created_at, updated_at) VALUES (?1, ?2, datetime('now'), datetime('now'))",
            params![id, name],
        )
        .unwrap();
    }

    #[test]
    fn update_partial_fields() {
        let conn = mem_db();
        insert_task(&conn, "t1", "original");
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE tasks SET name = ?1, quadrant = 'important_urgent', updated_at = ?2 WHERE id = 't1'",
            params!["updated", now],
        )
        .unwrap();
        let name: String = conn
            .query_row("SELECT name FROM tasks WHERE id = 't1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "updated");
        let q: String = conn
            .query_row("SELECT quadrant FROM tasks WHERE id = 't1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(q, "important_urgent");
    }

    #[test]
    fn delete_sets_shelved_at() {
        let conn = mem_db();
        insert_task(&conn, "t1", "to delete");
        let now = Utc::now().to_rfc3339();
        let affected = conn
            .execute(
                "UPDATE tasks SET shelved_at = ?1, updated_at = ?1 WHERE id = 't1' AND shelved_at IS NULL",
                params![now],
            )
            .unwrap();
        assert_eq!(affected, 1);
        let shelved: Option<String> = conn
            .query_row("SELECT shelved_at FROM tasks WHERE id = 't1'", [], |r| r.get(0))
            .unwrap();
        assert!(shelved.is_some());
    }

    #[test]
    fn delete_already_shelved_returns_zero() {
        let conn = mem_db();
        insert_task(&conn, "t1", "already gone");
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE tasks SET shelved_at = ?1 WHERE id = 't1'",
            params![now],
        )
        .unwrap();
        let affected = conn
            .execute(
                "UPDATE tasks SET shelved_at = ?1, updated_at = ?1 WHERE id = 't1' AND shelved_at IS NULL",
                params![now],
            )
            .unwrap();
        assert_eq!(affected, 0);
    }

    #[test]
    fn list_excludes_shelved() {
        let conn = mem_db();
        insert_task(&conn, "t1", "active");
        insert_task(&conn, "t2", "shelved");
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE tasks SET shelved_at = ?1 WHERE id = 't2'",
            params![now],
        )
        .unwrap();
        let mut stmt = conn
            .prepare("SELECT id FROM tasks WHERE status IN ('pending','in_progress') AND shelved_at IS NULL")
            .unwrap();
        let ids: Vec<String> = stmt.query_map([], |r| r.get(0)).unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "t1");
    }
}

//! Task 相关 #[tauri::command] — Week 1a 最小版 CRUD。
//!
//! list_tasks / create_task / complete_task
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
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        completed_at: row.get("completed_at")?,
    })
}

/// 列出任务。status_filter=None 时返回所有非 completed 的任务(pending + in_progress),
/// 否则按 status 精确过滤。
#[tauri::command]
pub fn list_tasks(
    status_filter: Option<String>,
    db: State<'_, Db>,
) -> AppResult<Vec<Task>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let mut stmt = if status_filter.is_some() {
        conn.prepare(
            "SELECT id, name, description, quadrant, status, created_at, updated_at, completed_at
             FROM tasks
             WHERE status = ?1
             ORDER BY sort_order, created_at DESC",
        )?
    } else {
        conn.prepare(
            "SELECT id, name, description, quadrant, status, created_at, updated_at, completed_at
             FROM tasks
             WHERE status IN ('pending', 'in_progress')
             ORDER BY sort_order, created_at DESC",
        )?
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

    conn.execute(
        "INSERT INTO tasks (id, name, quadrant, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'pending', ?4, ?4)",
        params![id, input.name, input.quadrant, now],
    )?;

    Ok(Task {
        id,
        name: input.name,
        description: None,
        quadrant: input.quadrant,
        status: "pending".into(),
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

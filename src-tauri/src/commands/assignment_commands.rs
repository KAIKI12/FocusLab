//! DailyTaskAssignment (DTA) CRUD 命令。
//!
//! 本轮只实现基本 CRUD;Week 3 再加:
//!   - 自动 carry_over(把未完成从昨天带入今天)
//!   - 到期自动置顶(source='auto_due_pinned')
//!   - 计划锁定(plan_locked_at 之后加入的算 extra 任务)

use chrono::Utc;
use rusqlite::{params, ErrorCode};
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use crate::db::Db;
use crate::models::daily_task_assignment::{AssignmentWithTask, DailyTaskAssignment};
use crate::models::settings;
use crate::utils::datetime::current_logical_date;
use crate::utils::errors::{AppError, AppResult};

fn row_to_joined(row: &rusqlite::Row<'_>) -> rusqlite::Result<AssignmentWithTask> {
    Ok(AssignmentWithTask {
        id: row.get("id")?,
        plan_date: row.get("plan_date")?,
        task_id: row.get("task_id")?,
        task_name: row.get("task_name")?,
        task_quadrant: row.get("task_quadrant")?,
        is_planned: row.get::<_, i64>("is_planned")? != 0,
        source: row.get("source")?,
        day_status: row.get("day_status")?,
        added_at: row.get("added_at")?,
        completed_at: row.get("completed_at")?,
        sort_order: row.get("sort_order")?,
    })
}

/// 列出某一天的任务分配(连表 tasks)。
/// `plan_date=None` 时使用当前逻辑日。
#[tauri::command]
pub fn list_assignments(
    plan_date: Option<String>,
    db: State<'_, Db>,
) -> AppResult<Vec<AssignmentWithTask>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let target_date = match plan_date {
        Some(d) => d,
        None => {
            let boundary = settings::get_boundary_hour(&conn)?;
            current_logical_date(boundary).to_string()
        }
    };

    let mut stmt = conn.prepare(
        "SELECT dta.id, dta.plan_date, dta.task_id, dta.is_planned, dta.source,
                dta.day_status, dta.added_at, dta.completed_at, dta.sort_order,
                t.name AS task_name, t.quadrant AS task_quadrant
           FROM daily_task_assignments dta
           JOIN tasks t ON t.id = dta.task_id
          WHERE dta.plan_date = ?1
          ORDER BY dta.sort_order, dta.added_at",
    )?;
    let rows = stmt
        .query_map(params![target_date], row_to_joined)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAssignmentInput {
    pub task_id: String,
    /// 留空走当前逻辑日
    pub plan_date: Option<String>,
    /// 缺省 "manual"
    pub source: Option<String>,
    /// 缺省 true
    pub is_planned: Option<bool>,
}

#[tauri::command]
pub fn create_assignment(
    input: CreateAssignmentInput,
    db: State<'_, Db>,
) -> AppResult<DailyTaskAssignment> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;

    let plan_date = match input.plan_date {
        Some(d) => d,
        None => {
            let boundary = settings::get_boundary_hour(&conn)?;
            current_logical_date(boundary).to_string()
        }
    };
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let source = input.source.unwrap_or_else(|| "manual".into());

    // 检查计划是否已锁定 — 锁定后新增的任务 is_planned=false
    let is_planned = if is_plan_locked(&conn, &plan_date) {
        false
    } else {
        input.is_planned.unwrap_or(true)
    };

    let insert_result = conn.execute(
        "INSERT INTO daily_task_assignments
            (id, plan_date, task_id, is_planned, source, day_status,
             added_at, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6, 0)",
        params![
            id,
            plan_date,
            input.task_id,
            if is_planned { 1i64 } else { 0 },
            source,
            now,
        ],
    );

    if let Err(rusqlite::Error::SqliteFailure(err, msg)) = &insert_result {
        if err.code == ErrorCode::ConstraintViolation {
            // 最常见:UNIQUE(plan_date, task_id) 冲突
            let detail = msg.as_deref().unwrap_or("");
            if detail.contains("plan_date") || detail.contains("UNIQUE") {
                return Err(AppError::Custom("该任务今天已在计划里".into()));
            }
        }
    }
    insert_result?;

    Ok(DailyTaskAssignment {
        id,
        plan_date,
        task_id: input.task_id,
        is_planned,
        source,
        day_status: "pending".into(),
        added_at: now,
        completed_at: None,
        sort_order: 0,
    })
}

#[tauri::command]
pub fn update_assignment_status(
    id: String,
    day_status: String,
    db: State<'_, Db>,
) -> AppResult<()> {
    // 验证入参
    let allowed = ["pending", "completed", "carried_forward", "shelved", "cancelled"];
    if !allowed.contains(&day_status.as_str()) {
        return Err(AppError::Custom(format!(
            "非法 day_status: {day_status}"
        )));
    }

    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let now = Utc::now().to_rfc3339();
    let affected = if day_status == "completed" {
        conn.execute(
            "UPDATE daily_task_assignments
                SET day_status = ?1, completed_at = ?2
              WHERE id = ?3",
            params![day_status, now, id],
        )?
    } else {
        conn.execute(
            "UPDATE daily_task_assignments
                SET day_status = ?1, completed_at = NULL
              WHERE id = ?2",
            params![day_status, id],
        )?
    };

    if affected == 0 {
        return Err(AppError::Custom(format!("assignment {id} not found")));
    }
    Ok(())
}

#[tauri::command]
pub fn remove_assignment(id: String, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let affected = conn.execute(
        "DELETE FROM daily_task_assignments WHERE id = ?1",
        params![id],
    )?;
    if affected == 0 {
        return Err(AppError::Custom(format!("assignment {id} not found")));
    }
    Ok(())
}

// ---------- Week 2b: 计划锁定 + 完成率 ----------

fn is_plan_locked(conn: &rusqlite::Connection, plan_date: &str) -> bool {
    conn.query_row(
        "SELECT plan_locked_at FROM daily_plans WHERE plan_date = ?1",
        params![plan_date],
        |r| r.get::<_, Option<String>>(0),
    )
    .unwrap_or(None)
    .is_some()
}

#[tauri::command]
pub fn lock_plan(plan_date: Option<String>, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let target = match plan_date {
        Some(d) => d,
        None => {
            let boundary = settings::get_boundary_hour(&conn)?;
            current_logical_date(boundary).to_string()
        }
    };
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO daily_plans (id, plan_date, plan_locked_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?3, ?3)
         ON CONFLICT(plan_date) DO UPDATE SET
           plan_locked_at = COALESCE(daily_plans.plan_locked_at, excluded.plan_locked_at),
           updated_at = excluded.updated_at",
        params![id, target, now],
    )?;
    Ok(())
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionStats {
    pub plan_date: String,
    pub is_locked: bool,
    pub planned_count: i64,
    pub completed_count: i64,
    pub extra_count: i64,
    pub extra_completed: i64,
    pub completion_rate: f64,
}

#[tauri::command]
pub fn get_completion_stats(
    plan_date: Option<String>,
    db: State<'_, Db>,
) -> AppResult<CompletionStats> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let target = match plan_date {
        Some(d) => d,
        None => {
            let boundary = settings::get_boundary_hour(&conn)?;
            current_logical_date(boundary).to_string()
        }
    };

    let locked = is_plan_locked(&conn, &target);

    let (planned_count, completed_count, extra_count, extra_completed) = conn
        .query_row(
            "SELECT
              COALESCE(SUM(CASE WHEN is_planned = 1 THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN is_planned = 1 AND day_status = 'completed' THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN is_planned = 0 THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN is_planned = 0 AND day_status = 'completed' THEN 1 ELSE 0 END), 0)
             FROM daily_task_assignments WHERE plan_date = ?1",
            params![target],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?, r.get::<_, i64>(3)?)),
        )
        .unwrap_or((0, 0, 0, 0));

    let rate = if planned_count > 0 {
        completed_count as f64 / planned_count as f64
    } else {
        0.0
    };

    Ok(CompletionStats {
        plan_date: target,
        is_locked: locked,
        planned_count,
        completed_count,
        extra_count,
        extra_completed,
        completion_rate: rate,
    })
}

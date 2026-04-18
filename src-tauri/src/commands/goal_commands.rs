//! 长线目标 + 里程碑 CRUD 命令。

use serde::Deserialize;
use tauri::State;

use crate::db::Db;
use crate::models::goal::{self, Goal};
use crate::models::milestone::{self, Milestone};
use crate::utils::errors::{AppError, AppResult};

// ---------- Goals ----------

#[tauri::command]
pub fn list_goals(include_archived: Option<bool>, db: State<'_, Db>) -> AppResult<Vec<Goal>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    goal::list_goals(&conn, include_archived.unwrap_or(false))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGoalInput {
    pub name: String,
    pub description: Option<String>,
    pub target_date: Option<String>,
}

#[tauri::command]
pub fn create_goal(input: CreateGoalInput, db: State<'_, Db>) -> AppResult<Goal> {
    if input.name.trim().is_empty() {
        return Err(AppError::Custom("目标名不能为空".into()));
    }
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    goal::create_goal(&conn, &input.name, input.description.as_deref(), input.target_date.as_deref())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGoalInput {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub target_date: Option<String>,
}

#[tauri::command]
pub fn update_goal(input: UpdateGoalInput, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    goal::update_goal(&conn, &input.id, input.name.as_deref(), input.description.as_deref(), input.target_date.as_deref())
}

#[tauri::command]
pub fn archive_goal(id: String, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    goal::archive_goal(&conn, &id)
}

// ---------- Milestones ----------

#[tauri::command]
pub fn list_milestones(goal_id: String, db: State<'_, Db>) -> AppResult<Vec<Milestone>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    milestone::list_milestones(&conn, &goal_id)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMilestoneInput {
    pub goal_id: String,
    pub name: String,
    pub description: Option<String>,
}

#[tauri::command]
pub fn create_milestone(input: CreateMilestoneInput, db: State<'_, Db>) -> AppResult<Milestone> {
    if input.name.trim().is_empty() {
        return Err(AppError::Custom("里程碑名不能为空".into()));
    }
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    milestone::create_milestone(&conn, &input.goal_id, &input.name, input.description.as_deref())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMilestoneInput {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[tauri::command]
pub fn update_milestone(input: UpdateMilestoneInput, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    milestone::update_milestone(&conn, &input.id, input.name.as_deref(), input.description.as_deref(), input.status.as_deref())
}

#[tauri::command]
pub fn complete_milestone(id: String, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    milestone::complete_milestone(&conn, &id)
}

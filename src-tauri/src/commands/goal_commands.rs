//! 长线目标 + 里程碑 CRUD 命令。

use chrono::{Datelike, DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::Db;
use crate::models::goal::{self, Goal};
use crate::models::milestone::{self, Milestone};
use crate::models::milestone_note::{self, MilestoneNote};
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetMilestoneTargetDateInput {
    pub milestone_id: String,
    /// 形如 "2026-04-30" 的 ISO date,`None` 表示清空。
    pub target_date: Option<String>,
}

#[tauri::command]
pub fn set_milestone_target_date(input: SetMilestoneTargetDateInput, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    milestone::set_milestone_target_date(&conn, &input.milestone_id, input.target_date.as_deref())
}

// ---------- Milestone Notes ----------

#[tauri::command]
pub fn list_milestone_notes(milestone_id: String, db: State<'_, Db>) -> AppResult<Vec<MilestoneNote>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    milestone_note::list_notes(&conn, &milestone_id)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMilestoneNoteInput {
    pub milestone_id: String,
    pub text: String,
}

#[tauri::command]
pub fn add_milestone_note(input: AddMilestoneNoteInput, db: State<'_, Db>) -> AppResult<MilestoneNote> {
    let text = input.text.trim();
    if text.is_empty() {
        return Err(AppError::Custom("备注内容不能为空".into()));
    }
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    milestone_note::add_note(&conn, &input.milestone_id, text)
}

#[tauri::command]
pub fn delete_milestone_note(id: String, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    milestone_note::delete_note(&conn, &id)
}

// ---------- Weekly Investment ----------

/// 一周 7 桶的时间投入数据(周一=0 .. 周日=6)。
/// 数据源:`sessions` 表,按 task.goal_id 过滤后按 `start_time` 的本地 weekday 分桶。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyInvestBucket {
    pub weekday: u32,
    pub minutes: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyInvest {
    /// 7 个桶,顺序一定是 0..=6(周一..周日),未命中日为 0
    pub buckets: Vec<WeeklyInvestBucket>,
    /// 本周(周一 00:00 到此刻)总投入分钟
    pub total_minutes: i64,
    /// 今天已投入分钟(方便前端高亮当日)
    pub today_minutes: i64,
}

/// 查询本目标本周(周一至周日)时间投入。
///
/// 聚合口径:
/// - 加入本周范围:start_time >= 本周周一 00:00:00(本机时区)
/// - 仅统计已有 end_time 的 session(完成/中止都算,按 actual_duration_minutes)
/// - 跨日不分割:归到 start_time 当天
#[tauri::command]
pub fn get_goal_weekly_invest(goal_id: String, db: State<'_, Db>) -> AppResult<WeeklyInvest> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;

    // 本机时区下本周一的 00:00(ISO weekday: 周一=1)
    let now_local = chrono::Local::now();
    let days_from_monday = now_local.weekday().num_days_from_monday() as i64;
    let monday = now_local
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .checked_sub_signed(chrono::Duration::days(days_from_monday))
        .unwrap();
    let monday_local = chrono::Local
        .from_local_datetime(&monday)
        .single()
        .ok_or_else(|| AppError::Custom("本地时间计算失败".into()))?;
    let monday_utc: DateTime<Utc> = monday_local.with_timezone(&Utc);
    let monday_rfc = monday_utc.to_rfc3339();

    let mut stmt = conn.prepare(
        "SELECT s.start_time, COALESCE(s.actual_duration_minutes, 0) AS mins
         FROM sessions s
         INNER JOIN tasks t ON t.id = s.task_id
         WHERE t.milestone_id IN (SELECT id FROM milestones WHERE goal_id = ?1)
           AND s.start_time >= ?2
           AND s.end_time IS NOT NULL",
    )?;

    let mut buckets = vec![0_i64; 7];
    let today_weekday = now_local.weekday().num_days_from_monday();

    let rows = stmt.query_map(rusqlite::params![goal_id, monday_rfc], |r| {
        let start: String = r.get(0)?;
        let mins: i64 = r.get(1)?;
        Ok((start, mins))
    })?;

    for row in rows {
        let (start, mins) = row?;
        // 转本地时间再取 weekday
        if let Ok(dt) = DateTime::parse_from_rfc3339(&start) {
            let wd = dt.with_timezone(&chrono::Local).weekday().num_days_from_monday() as usize;
            if wd < 7 {
                buckets[wd] += mins;
            }
        }
    }

    let total_minutes: i64 = buckets.iter().sum();
    let today_minutes = buckets[today_weekday as usize];

    Ok(WeeklyInvest {
        buckets: (0..7)
            .map(|i| WeeklyInvestBucket { weekday: i, minutes: buckets[i as usize] })
            .collect(),
        total_minutes,
        today_minutes,
    })
}

/// 全部 active goal 的本周投入汇总(用于 StatsView 侧栏)。
/// 过滤条件: goal.status='active' · session.end_time 非空 · start_time >= 本周一。
/// 返回按 total_minutes 降序,0 投入的不过滤,由前端决定是否隐藏。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalWeeklyInvest {
    pub goal_id: String,
    pub goal_name: String,
    pub total_minutes: i64,
}

#[tauri::command]
pub fn list_goal_weekly_invests(db: State<'_, Db>) -> AppResult<Vec<GoalWeeklyInvest>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;

    let now_local = chrono::Local::now();
    let days_from_monday = now_local.weekday().num_days_from_monday() as i64;
    let monday = now_local
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .checked_sub_signed(chrono::Duration::days(days_from_monday))
        .unwrap();
    let monday_local = chrono::Local
        .from_local_datetime(&monday)
        .single()
        .ok_or_else(|| AppError::Custom("本地时间计算失败".into()))?;
    let monday_utc: DateTime<Utc> = monday_local.with_timezone(&Utc);
    let monday_rfc = monday_utc.to_rfc3339();

    let mut stmt = conn.prepare(
        "SELECT g.id, g.name,
                COALESCE(SUM(COALESCE(s.actual_duration_minutes, 0)), 0) AS mins
         FROM goals g
         LEFT JOIN milestones m ON m.goal_id = g.id
         LEFT JOIN tasks t      ON t.milestone_id = m.id
         LEFT JOIN sessions s   ON s.task_id = t.id
             AND s.start_time >= ?1
             AND s.end_time IS NOT NULL
         WHERE g.status = 'active'
         GROUP BY g.id, g.name
         ORDER BY mins DESC, g.sort_order ASC",
    )?;

    let rows = stmt
        .query_map(rusqlite::params![monday_rfc], |r| {
            Ok(GoalWeeklyInvest {
                goal_id: r.get(0)?,
                goal_name: r.get(1)?,
                total_minutes: r.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(rows)
}

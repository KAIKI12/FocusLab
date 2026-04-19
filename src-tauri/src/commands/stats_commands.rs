//! 数据洞察统计命令 — 热力图 / 完成率趋势 / 时间分类。

use rusqlite::params;
use serde::Serialize;
use tauri::State;

use crate::db::Db;
use crate::utils::errors::{AppError, AppResult};

/// 热力图数据点 — 一周中每个时段的专注分钟数
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatmapCell {
    pub day_of_week: i64,  // 0=Sun, 6=Sat
    pub hour: i64,         // 0-23
    pub minutes: i64,      // 该时段的总专注分钟
}

/// 热力图：按周几×小时聚合近 N 天的专注数据
#[tauri::command]
pub fn get_focus_heatmap(days: Option<i64>, db: State<'_, Db>) -> AppResult<Vec<HeatmapCell>> {
    let n = days.unwrap_or(30);
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let mut stmt = conn.prepare(
        "SELECT
            CAST(strftime('%w', start_time) AS INTEGER) AS dow,
            CAST(strftime('%H', start_time) AS INTEGER) AS hr,
            COALESCE(SUM(actual_duration_minutes), 0) AS mins
         FROM sessions
         WHERE status = 'completed'
           AND start_time >= datetime('now', ?1)
         GROUP BY dow, hr
         ORDER BY dow, hr",
    )?;
    let offset = format!("-{n} days");
    let rows = stmt
        .query_map(params![offset], |r| {
            Ok(HeatmapCell {
                day_of_week: r.get(0)?,
                hour: r.get(1)?,
                minutes: r.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// 完成率趋势数据点
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendPoint {
    pub date: String,
    pub completion_rate: f64,
    pub completed: i64,
    pub total: i64,
    pub focus_minutes: i64,
}

/// 近 N 天的日完成率趋势
#[tauri::command]
pub fn get_completion_trend(days: Option<i64>, db: State<'_, Db>) -> AppResult<Vec<TrendPoint>> {
    let n = days.unwrap_or(30);
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let mut stmt = conn.prepare(
        "SELECT settle_date, completion_rate, completed_tasks, total_tasks, total_focus_minutes
         FROM settlements
         WHERE settle_date >= date('now', ?1)
         ORDER BY settle_date",
    )?;
    let offset = format!("-{n} days");
    let rows = stmt
        .query_map(params![offset], |r| {
            Ok(TrendPoint {
                date: r.get(0)?,
                completion_rate: r.get(1)?,
                completed: r.get(2)?,
                total: r.get(3)?,
                focus_minutes: r.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// 时间分类数据点 — 按象限统计专注分钟
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryTime {
    pub quadrant: String,
    pub minutes: i64,
    pub session_count: i64,
}

/// 按任务象限分类的专注时间统计
#[tauri::command]
pub fn get_time_by_category(days: Option<i64>, db: State<'_, Db>) -> AppResult<Vec<CategoryTime>> {
    let n = days.unwrap_or(30);
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let mut stmt = conn.prepare(
        "SELECT
            COALESCE(t.quadrant, 'unknown') AS q,
            COALESCE(SUM(s.actual_duration_minutes), 0) AS mins,
            COUNT(*) AS cnt
         FROM sessions s
         LEFT JOIN tasks t ON t.id = s.task_id
         WHERE s.status = 'completed'
           AND s.start_time >= datetime('now', ?1)
         GROUP BY q
         ORDER BY mins DESC",
    )?;
    let offset = format!("-{n} days");
    let rows = stmt
        .query_map(params![offset], |r| {
            Ok(CategoryTime {
                quadrant: r.get(0)?,
                minutes: r.get(1)?,
                session_count: r.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// 总体统计概览
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsOverview {
    pub total_focus_minutes: i64,
    pub total_sessions: i64,
    pub total_pomodoros: i64,
    pub total_tasks_completed: i64,
    pub avg_daily_focus: f64,
    pub best_grade_count: i64, // S 级次数
    pub current_streak: i64,   // 连续结算天数
}

/// 总体统计概览
#[tauri::command]
pub fn get_stats_overview(days: Option<i64>, db: State<'_, Db>) -> AppResult<StatsOverview> {
    let n = days.unwrap_or(30);
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let offset = format!("-{n} days");

    let (focus_min, sessions, pomodoros): (i64, i64, i64) = conn
        .query_row(
            "SELECT
                COALESCE(SUM(actual_duration_minutes), 0),
                COUNT(*),
                COALESCE(SUM(CASE WHEN mode='pomodoro' AND status='completed' THEN 1 ELSE 0 END), 0)
             FROM sessions
             WHERE start_time >= datetime('now', ?1)",
            params![offset],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap_or((0, 0, 0));

    let tasks_completed: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks WHERE status='completed' AND updated_at >= datetime('now', ?1)",
            params![offset],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let settle_days: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM settlements WHERE settle_date >= date('now', ?1)",
            params![offset],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let avg_daily = if settle_days > 0 {
        focus_min as f64 / settle_days as f64
    } else {
        0.0
    };

    let s_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM settlements WHERE grade='S' AND settle_date >= date('now', ?1)",
            params![offset],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // 连续结算天数（从今天往回数）
    let streak: i64 = conn
        .query_row(
            "WITH RECURSIVE dates(d, n) AS (
                SELECT date('now'), 0
                UNION ALL
                SELECT date(d, '-1 day'), n+1 FROM dates
                WHERE EXISTS (SELECT 1 FROM settlements WHERE settle_date = date(d, '-1 day'))
                  AND n < 365
             )
             SELECT MAX(n) FROM dates
             WHERE EXISTS (SELECT 1 FROM settlements WHERE settle_date = d)
                OR n = 0",
            [],
            |r| r.get::<_, Option<i64>>(0),
        )
        .unwrap_or(None)
        .unwrap_or(0);

    Ok(StatsOverview {
        total_focus_minutes: focus_min,
        total_sessions: sessions,
        total_pomodoros: pomodoros,
        total_tasks_completed: tasks_completed,
        avg_daily_focus: avg_daily,
        best_grade_count: s_count,
        current_streak: streak,
    })
}

//! 日结算命令 — settle_day / get_settlement / get_yesterday_summary。

use chrono::Utc;
use rusqlite::params;
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use crate::db::Db;
use crate::models::settlement::{Settlement, YesterdaySummary};
use crate::models::settings;
use crate::utils::datetime::current_logical_date;
use crate::utils::errors::{AppError, AppResult};

/// 评级逻辑(纯函数,可单测)
pub fn compute_grade(planned: i64, completed: i64, extra_completed: i64) -> String {
    if planned == 0 {
        return "C".into();
    }
    let rate = completed as f64 / planned as f64;
    if rate >= 1.0 && extra_completed >= 1 {
        "S".into()
    } else if rate >= 1.0 {
        "A".into()
    } else if rate >= 0.7 {
        "B".into()
    } else {
        "C".into()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettleInput {
    pub plan_date: Option<String>,
    pub trigger_type: Option<String>,
    pub user_reflection: Option<String>,
}

/// 执行日结算:计算评级 + 写 settlements 行 + carry-over 未完成任务
#[tauri::command]
pub fn settle_day(input: SettleInput, db: State<'_, Db>) -> AppResult<Settlement> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let boundary = settings::get_boundary_hour(&conn)?;
    let target_date = input
        .plan_date
        .unwrap_or_else(|| current_logical_date(boundary).to_string());

    // 已结算过则返回错误
    let existing: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM settlements WHERE settle_date = ?1",
            params![target_date],
            |r| r.get(0),
        )
        .unwrap_or(false);
    if existing {
        return Err(AppError::Custom("今天已经结算过了".into()));
    }

    // 1) 查 daily_task_assignments 指标
    let (planned, completed, extra, extra_completed, shelved): (i64, i64, i64, i64, i64) = conn
        .query_row(
            "SELECT
              COALESCE(SUM(CASE WHEN is_planned = 1 THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN is_planned = 1 AND day_status = 'completed' THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN is_planned = 0 THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN is_planned = 0 AND day_status = 'completed' THEN 1 ELSE 0 END), 0),
              COALESCE(SUM(CASE WHEN day_status = 'shelved' THEN 1 ELSE 0 END), 0)
             FROM daily_task_assignments WHERE plan_date = ?1",
            params![target_date],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .unwrap_or((0, 0, 0, 0, 0));

    // 2) 查 sessions 指标
    let (focus_min, pomodoros): (i64, i64) = conn
        .query_row(
            "SELECT
              COALESCE(SUM(actual_duration_minutes), 0),
              COALESCE(SUM(CASE WHEN mode = 'pomodoro' AND status = 'completed' THEN 1 ELSE 0 END), 0)
             FROM sessions
             WHERE DATE(start_time) = ?1",
            params![target_date],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap_or((0, 0));

    // 3) 中断数
    let interruptions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM interruptions i
             JOIN sessions s ON s.id = i.session_id
             WHERE DATE(s.start_time) = ?1",
            params![target_date],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // 4) 最长专注任务
    let longest: Option<(String, i64)> = conn
        .query_row(
            "SELECT task_id, actual_duration_minutes FROM sessions
             WHERE DATE(start_time) = ?1 AND status = 'completed'
             ORDER BY actual_duration_minutes DESC LIMIT 1",
            params![target_date],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();

    // 5) 评级
    let rate = if planned > 0 {
        completed as f64 / planned as f64
    } else {
        0.0
    };
    let grade = compute_grade(planned, completed, extra_completed);

    // 6) INSERT
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let trigger = input.trigger_type.unwrap_or_else(|| "manual".into());
    conn.execute(
        "INSERT INTO settlements
            (id, settle_date, total_tasks, completed_tasks, extra_tasks, shelved_tasks,
             completion_rate, total_focus_minutes, total_pomodoros, total_interruptions,
             grade, longest_focus_task_id, longest_focus_minutes,
             user_reflection, trigger_type, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
        params![
            id,
            target_date,
            planned + extra,
            completed + extra_completed,
            extra,
            shelved,
            rate,
            focus_min,
            pomodoros,
            interruptions,
            grade,
            longest.as_ref().map(|(tid, _)| tid.as_str()),
            longest.as_ref().map(|(_, min)| *min),
            input.user_reflection,
            trigger,
            now,
        ],
    )?;

    // 7) Carry-over 未完成的 planned tasks → 下一逻辑日
    let next_date = {
        let d = chrono::NaiveDate::parse_from_str(&target_date, "%Y-%m-%d")
            .map_err(|e| AppError::Custom(format!("parse date: {e}")))?;
        (d + chrono::Duration::days(1)).to_string()
    };
    let mut carry_stmt = conn.prepare(
        "SELECT task_id FROM daily_task_assignments
         WHERE plan_date = ?1 AND is_planned = 1 AND day_status = 'pending'",
    )?;
    let carry_task_ids: Vec<String> = carry_stmt
        .query_map(params![target_date], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let _carry_count = carry_task_ids.len();
    for tid in &carry_task_ids {
        let cid = Uuid::new_v4().to_string();
        let _ = conn.execute(
            "INSERT OR IGNORE INTO daily_task_assignments
                (id, plan_date, task_id, is_planned, source, day_status, added_at, sort_order)
             VALUES (?1, ?2, ?3, 1, 'carried_over', 'pending', ?4, 0)",
            params![cid, next_date, tid, now],
        );
        // 标原 dta 为 carried_forward
        conn.execute(
            "UPDATE daily_task_assignments SET day_status = 'carried_forward'
             WHERE plan_date = ?1 AND task_id = ?2 AND day_status = 'pending'",
            params![target_date, tid],
        )?;
    }

    Ok(Settlement {
        id,
        settle_date: target_date,
        total_tasks: planned + extra,
        completed_tasks: completed + extra_completed,
        extra_tasks: extra,
        shelved_tasks: shelved,
        completion_rate: rate,
        total_focus_minutes: focus_min,
        total_pomodoros: pomodoros,
        total_interruptions: interruptions,
        grade,
        longest_focus_task_id: longest.as_ref().map(|(tid, _)| tid.clone()),
        longest_focus_minutes: longest.map(|(_, min)| min),
        ai_summary: None,
        user_reflection: input.user_reflection,
        trigger_type: trigger,
        created_at: now,
    })
}

/// 查询某天的结算(如果已结算)
#[tauri::command]
pub fn get_settlement(plan_date: Option<String>, db: State<'_, Db>) -> AppResult<Option<Settlement>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let boundary = settings::get_boundary_hour(&conn)?;
    let target = plan_date.unwrap_or_else(|| current_logical_date(boundary).to_string());

    let result = conn.query_row(
        "SELECT id, settle_date, total_tasks, completed_tasks, extra_tasks, shelved_tasks,
                completion_rate, total_focus_minutes, total_pomodoros, total_interruptions,
                grade, longest_focus_task_id, longest_focus_minutes,
                ai_summary, user_reflection, trigger_type, created_at
         FROM settlements WHERE settle_date = ?1",
        params![target],
        |r| {
            Ok(Settlement {
                id: r.get(0)?,
                settle_date: r.get(1)?,
                total_tasks: r.get(2)?,
                completed_tasks: r.get(3)?,
                extra_tasks: r.get(4)?,
                shelved_tasks: r.get(5)?,
                completion_rate: r.get(6)?,
                total_focus_minutes: r.get(7)?,
                total_pomodoros: r.get(8)?,
                total_interruptions: r.get(9)?,
                grade: r.get(10)?,
                longest_focus_task_id: r.get(11)?,
                longest_focus_minutes: r.get(12)?,
                ai_summary: r.get(13)?,
                user_reflection: r.get(14)?,
                trigger_type: r.get(15)?,
                created_at: r.get(16)?,
            })
        },
    );

    match result {
        Ok(s) => Ok(Some(s)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Custom(e.to_string())),
    }
}

/// 获取昨日摘要(打开应用时的 YesterdayCard 用)
#[tauri::command]
pub fn get_yesterday_summary(db: State<'_, Db>) -> AppResult<Option<YesterdaySummary>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let boundary = settings::get_boundary_hour(&conn)?;
    let today = current_logical_date(boundary);
    let yesterday = (today - chrono::Duration::days(1)).to_string();

    let result = conn.query_row(
        "SELECT settle_date, completed_tasks, total_tasks, completion_rate, grade,
                total_focus_minutes, total_pomodoros, longest_focus_task_id, longest_focus_minutes
         FROM settlements WHERE settle_date = ?1",
        params![yesterday],
        |r| {
            let longest_tid: Option<String> = r.get(7)?;
            let longest_min: Option<i64> = r.get(8)?;
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, f64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, i64>(6)?,
                longest_tid,
                longest_min,
            ))
        },
    );

    let (settle_date, completed, total, rate, grade, focus_min, pomodoros, longest_tid, _longest_min) =
        match result {
            Ok(r) => r,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(AppError::Custom(e.to_string())),
        };

    // 查最长专注任务名
    let longest_name: Option<String> = longest_tid.and_then(|tid| {
        conn.query_row("SELECT name FROM tasks WHERE id = ?1", params![tid], |r| r.get(0)).ok()
    });

    // 查 carry-over 数量(昨天 pending 被搬到今天)
    let carried: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM daily_task_assignments
             WHERE plan_date = ?1 AND source = 'carried_over'",
            params![today.to_string()],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(Some(YesterdaySummary {
        settle_date,
        completed_tasks: completed,
        total_tasks: total,
        completion_rate: rate,
        grade,
        total_focus_minutes: focus_min,
        total_pomodoros: pomodoros,
        longest_focus_task_name: longest_name,
        carried_over_count: carried,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grade_s() {
        assert_eq!(compute_grade(5, 5, 1), "S");
    }
    #[test]
    fn grade_a() {
        assert_eq!(compute_grade(5, 5, 0), "A");
    }
    #[test]
    fn grade_b() {
        assert_eq!(compute_grade(10, 7, 0), "B");
    }
    #[test]
    fn grade_c() {
        assert_eq!(compute_grade(10, 3, 0), "C");
    }
    #[test]
    fn grade_zero_planned() {
        assert_eq!(compute_grade(0, 0, 0), "C");
    }
}

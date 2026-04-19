//! 数据导出命令 — 任务 + 会话导出为 JSON/CSV。

use std::io::Write;
use tauri::State;

use crate::db::Db;
use crate::utils::errors::{AppError, AppResult};

/// 导出任务列表为 JSON
#[tauri::command]
pub fn export_tasks_json(path: String, db: State<'_, Db>) -> AppResult<String> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let mut stmt = conn.prepare(
        "SELECT id, name, description, quadrant, status, estimated_minutes,
                due_date, is_background, created_at, updated_at, completed_at
         FROM tasks WHERE shelved_at IS NULL
         ORDER BY created_at DESC",
    )?;

    let rows: Vec<serde_json::Value> = stmt
        .query_map([], |r| {
            Ok(serde_json::json!({
                "id": r.get::<_, String>(0)?,
                "name": r.get::<_, String>(1)?,
                "description": r.get::<_, Option<String>>(2)?,
                "quadrant": r.get::<_, String>(3)?,
                "status": r.get::<_, String>(4)?,
                "estimated_minutes": r.get::<_, Option<i64>>(5)?,
                "due_date": r.get::<_, Option<String>>(6)?,
                "is_background": r.get::<_, bool>(7)?,
                "created_at": r.get::<_, String>(8)?,
                "updated_at": r.get::<_, String>(9)?,
                "completed_at": r.get::<_, Option<String>>(10)?,
            }))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let json = serde_json::to_string_pretty(&rows)
        .map_err(|e| AppError::Custom(format!("JSON 序列化失败: {e}")))?;

    let mut f = std::fs::File::create(&path)
        .map_err(|e| AppError::Custom(format!("无法创建文件: {e}")))?;
    f.write_all(json.as_bytes())
        .map_err(|e| AppError::Custom(format!("写入失败: {e}")))?;

    Ok(format!("已导出 {} 条任务到 {}", rows.len(), path))
}

/// 导出会话记录为 CSV
#[tauri::command]
pub fn export_sessions_csv(path: String, db: State<'_, Db>) -> AppResult<String> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let mut stmt = conn.prepare(
        "SELECT s.id, s.task_id, t.name, s.start_time, s.end_time,
                s.actual_duration_minutes, s.mode, s.pomodoro_preset,
                s.status, s.is_manual_entry, s.created_at
         FROM sessions s
         LEFT JOIN tasks t ON t.id = s.task_id
         ORDER BY s.start_time DESC",
    )?;

    let mut csv = String::from("id,task_id,task_name,start_time,end_time,duration_minutes,mode,preset,status,is_manual,created_at\n");
    let mut count = 0i64;

    stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, Option<String>>(4)?,
            r.get::<_, Option<i64>>(5)?,
            r.get::<_, String>(6)?,
            r.get::<_, Option<String>>(7)?,
            r.get::<_, String>(8)?,
            r.get::<_, bool>(9)?,
            r.get::<_, String>(10)?,
        ))
    })?
    .for_each(|row| {
        if let Ok((id, tid, tname, start, end, dur, mode, preset, status, manual, created)) = row {
            let escape = |s: &str| {
                if s.contains(',') || s.contains('"') || s.contains('\n') {
                    format!("\"{}\"", s.replace('"', "\"\""))
                } else {
                    s.to_string()
                }
            };
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{}\n",
                id,
                tid,
                escape(tname.as_deref().unwrap_or("")),
                start,
                end.as_deref().unwrap_or(""),
                dur.unwrap_or(0),
                mode,
                preset.as_deref().unwrap_or(""),
                status,
                if manual { 1 } else { 0 },
                created,
            ));
            count += 1;
        }
    });

    let mut f = std::fs::File::create(&path)
        .map_err(|e| AppError::Custom(format!("无法创建文件: {e}")))?;

    // BOM for Excel compatibility
    f.write_all(&[0xEF, 0xBB, 0xBF])
        .map_err(|e| AppError::Custom(format!("写入 BOM 失败: {e}")))?;
    f.write_all(csv.as_bytes())
        .map_err(|e| AppError::Custom(format!("写入失败: {e}")))?;

    Ok(format!("已导出 {} 条记录到 {}", count, path))
}

//! AI 命令 — 配置 / 测试连接 / 任务拆解 / 结算叙事。

use rusqlite::params;
use serde::Deserialize;
use tauri::State;

use crate::ai::prompt_templates;
use crate::ai::{AIService, CompletionOptions, Message, ResponseValidator};
use crate::db::Db;
use crate::models::settings;
use crate::utils::errors::{AppError, AppResult};

// ---------- 工具函数 ----------

/// 读取单个 settings KV，失败时返回默认值
fn get_setting_or(conn: &rusqlite::Connection, key: &str, default: &str) -> String {
    settings::get(conn, key)
        .ok()
        .flatten()
        .unwrap_or_else(|| default.to_string())
}

/// 检查全局 AI 开关，关闭时返回 Err
fn check_ai_enabled(conn: &rusqlite::Connection) -> AppResult<()> {
    let enabled = get_setting_or(conn, "ai_enabled", "1");
    if enabled == "0" {
        Err(AppError::Custom("AI 功能已关闭，请在设置中开启".into()))
    } else {
        Ok(())
    }
}

/// 根据 ai_tone_intensity (1-5) 映射 temperature 和 max_tokens
fn intensity_to_opts(intensity: i32, base_max_tokens: u32) -> CompletionOptions {
    let (temperature, scale) = match intensity {
        1 => (0.30_f64, 0.36_f64),
        2 => (0.45, 0.64),
        3 => (0.60, 1.00),
        4 => (0.75, 1.36),
        5 => (0.90, 1.79),
        _ => (0.60, 1.00),
    };
    let max_tokens = ((base_max_tokens as f64) * scale).round() as u32;
    CompletionOptions {
        temperature: Some(temperature),
        max_tokens: Some(max_tokens.max(50)),
    }
}

/// 从 settings 读取 tone 和 tone_custom
fn load_tone(conn: &rusqlite::Connection) -> (String, String) {
    let tone = get_setting_or(conn, "ai_tone", "academic");
    let custom = get_setting_or(conn, "ai_tone_custom", "");
    (tone, custom)
}

/// 从 settings 读取 intensity，返回 1-5 的值
fn load_intensity(conn: &rusqlite::Connection) -> i32 {
    let raw = get_setting_or(conn, "ai_tone_intensity", "3");
    raw.parse::<i32>().unwrap_or(3).clamp(1, 5)
}

// ---------- 配置命令 ----------

/// 配置 AI provider（从 settings KV 读取或前端直接传入）
#[tauri::command]
pub async fn configure_ai(
    provider: String,
    api_format: String,
    base_url: String,
    api_key: String,
    model: String,
    enabled: Option<String>,
    tone: Option<String>,
    tone_custom: Option<String>,
    intensity: Option<String>,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<()> {
    // 持久化到 settings
    {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let now = chrono::Utc::now().to_rfc3339();

        let mut pairs: Vec<(&str, String)> = vec![
            ("ai_provider", provider.clone()),
            ("ai_api_format", api_format.clone()),
            ("ai_base_url", base_url.clone()),
            ("ai_api_key", api_key.clone()),
            ("ai_model", model.clone()),
        ];
        if let Some(v) = &enabled    { pairs.push(("ai_enabled", v.clone())); }
        if let Some(v) = &tone       { pairs.push(("ai_tone", v.clone())); }
        if let Some(v) = &tone_custom { pairs.push(("ai_tone_custom", v.clone())); }
        if let Some(v) = &intensity  { pairs.push(("ai_tone_intensity", v.clone())); }

        for (k, v) in &pairs {
            conn.execute(
                "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![k, v, now],
            )?;
        }
    }

    ai.configure(&provider, &api_format, &base_url, &api_key, &model).await;
    tracing::info!("AI configured: provider={provider} api_format={api_format} model={model}");
    Ok(())
}

/// 测试 AI 连接（不受全局开关限制）
#[tauri::command]
pub async fn test_ai_connection(
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<String> {
    // 测试连接后更新状态
    let messages = vec![Message {
        role: "user".into(),
        content: "请回复'连接成功'四个字。".into(),
    }];
    let result = ai
        .complete(messages, CompletionOptions { max_tokens: Some(20), ..Default::default() })
        .await?;

    // 写入连接状态
    {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let now = chrono::Utc::now().to_rfc3339();
        let _ = conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params!["ai_connection_status", "ok", &now],
        );
        let _ = conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params!["ai_connection_checked_at", &now, &now],
        );
    }
    Ok(result)
}

// ---------- AI 任务拆解 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecomposeInput {
    pub task_name: String,
    pub description: Option<String>,
}

#[tauri::command]
pub async fn ai_decompose_task(
    input: DecomposeInput,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<String> {
    {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;
    }
    let prompt = prompt_templates::decompose_prompt(
        &input.task_name,
        input.description.as_deref().unwrap_or(""),
    );
    let messages = vec![Message { role: "user".into(), content: prompt }];
    // 任务拆解固定参数，不受 intensity 影响
    let result = ai
        .complete(messages, CompletionOptions { temperature: Some(0.7), max_tokens: Some(800) })
        .await?;
    Ok(ResponseValidator::validate_decompose(&result))
}

// ---------- AI 结算叙事 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NarrativeInput {
    pub grade: String,
    pub completed: i64,
    pub total: i64,
    pub focus_minutes: i64,
    pub tone: Option<String>,
}

#[tauri::command]
pub async fn ai_settlement_narrative(
    input: NarrativeInput,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<String> {
    let (tone, _custom, intensity) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;
        let (t, c) = load_tone(&conn);
        let i = load_intensity(&conn);
        (t, c, i)
    };
    // 前端传入的 tone 优先级高于 settings（向后兼容）
    let effective_tone = input.tone.as_deref().unwrap_or(&tone);
    let prompt = prompt_templates::settlement_prompt(
        &input.grade,
        input.completed,
        input.total,
        input.focus_minutes,
        effective_tone,
    );
    let messages = vec![Message { role: "user".into(), content: prompt }];
    let opts = intensity_to_opts(intensity, 200);
    let result = ai.complete(messages, opts).await?;
    Ok(ResponseValidator::validate_narrative(&result))
}

// ---------- AI 每日建议 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailySuggestionInput {
    pub energy_level: Option<String>,
}

#[tauri::command]
pub async fn ai_daily_suggestions(
    input: DailySuggestionInput,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<String> {
    let (yesterday_summary, pending_tasks, tone, intensity) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;

        // 昨日摘要
        let yesterday: String = conn
            .query_row(
                "SELECT COALESCE(grade || ' 级, 完成 ' || completed_tasks || '/' || total_tasks || ' 项, 专注 ' || total_focus_minutes || ' 分钟', '无昨日数据')
                 FROM settlements ORDER BY settle_date DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "无昨日数据".into());

        // 待处理任务列表
        let mut stmt = conn
            .prepare(
                "SELECT name FROM tasks WHERE status IN ('pending','in_progress') AND shelved_at IS NULL ORDER BY created_at DESC LIMIT 20",
            )
            .map_err(|e| AppError::Custom(e.to_string()))?;
        let names: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| AppError::Custom(e.to_string()))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| AppError::Custom(e.to_string()))?;
        let pending = if names.is_empty() { "无待办任务".into() } else { names.join("、") };

        let (t, _c) = load_tone(&conn);
        let i = load_intensity(&conn);
        (yesterday, pending, t, i)
    };

    let energy = input.energy_level.as_deref().unwrap_or("正常");
    let prompt = prompt_templates::daily_suggestion_prompt(
        &yesterday_summary,
        &pending_tasks,
        energy,
        &tone,
    );
    let messages = vec![Message { role: "user".into(), content: prompt }];
    let opts = intensity_to_opts(intensity, 400);
    let result = ai.complete(messages, opts).await?;
    Ok(result)
}

// ---------- AI 四象限自动分类 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassifyInput {
    pub task_name: String,
    pub description: Option<String>,
}

#[tauri::command]
pub async fn ai_classify_quadrant(
    input: ClassifyInput,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<String> {
    {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;
    }
    let prompt = prompt_templates::classify_quadrant_prompt(
        &input.task_name,
        input.description.as_deref().unwrap_or(""),
    );
    let messages = vec![Message { role: "user".into(), content: prompt }];
    // 四象限分类固定参数，不受 intensity 影响
    let result = ai
        .complete(messages, CompletionOptions { temperature: Some(0.3), max_tokens: Some(50) })
        .await?;
    let trimmed = result.trim().to_lowercase();
    let valid = [
        "important_urgent",
        "important_not_urgent",
        "not_important_urgent",
        "not_important_not_urgent",
    ];
    if valid.contains(&trimmed.as_str()) {
        Ok(trimmed)
    } else {
        Ok("important_not_urgent".into())
    }
}

// ---------- AI 速记便签优化 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickNoteInput {
    pub raw_text: String,
}

#[tauri::command]
pub async fn ai_optimize_quick_note(
    input: QuickNoteInput,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<String> {
    let intensity = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;
        load_intensity(&conn)
    };
    let prompt = prompt_templates::quick_note_optimization_prompt(&input.raw_text);
    let messages = vec![Message { role: "user".into(), content: prompt }];
    let opts = intensity_to_opts(intensity, 600);
    let result = ai.complete(messages, opts).await?;
    Ok(ResponseValidator::validate_quick_note(&result))
}

// ---------- AI 未完成任务温和提醒 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnfinishedReminderInput {
    /// 未完成任务列表，逗号分隔
    pub unfinished_tasks: String,
    /// 今日已完成摘要（如 "完成 3/5 项"）
    pub completed_summary: String,
    /// 明日可用时间描述（如 "上午 2 小时"），可空
    pub available_time: Option<String>,
}

#[tauri::command]
pub async fn ai_unfinished_reminder(
    input: UnfinishedReminderInput,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<String> {
    let (tone, custom, intensity) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;
        let (t, c) = load_tone(&conn);
        let i = load_intensity(&conn);
        (t, c, i)
    };
    let available_time = input.available_time.as_deref().unwrap_or("待定");
    let prompt = prompt_templates::unfinished_reminder_prompt(
        &input.unfinished_tasks,
        &input.completed_summary,
        available_time,
        &tone,
        &custom,
    );
    let messages = vec![Message { role: "user".into(), content: prompt }];
    let opts = intensity_to_opts(intensity, 200);
    let result = ai.complete(messages, opts).await?;
    // 基本校验：必须含 "message" 字段
    if result.contains("\"message\"") {
        Ok(result)
    } else {
        Ok(format!(
            "{{\"message\":\"今天已有不少收获，未完成的任务明天继续加油！\",\"next_step\":\"选一项最小的任务先开始\",\"tone\":\"gentle\"}}"
        ))
    }
}

// ---------- AI 任务完成正反馈 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskFeedbackInput {
    pub task_name: String,
    pub estimated_minutes: Option<i64>,
    pub actual_minutes: Option<i64>,
    pub quadrant: Option<String>,
}

#[tauri::command]
pub async fn ai_task_feedback(
    input: TaskFeedbackInput,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<String> {
    let (tone, custom, intensity) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;
        let (t, c) = load_tone(&conn);
        let i = load_intensity(&conn);
        (t, c, i)
    };
    let estimated = input.estimated_minutes.unwrap_or(0);
    let actual = input.actual_minutes.unwrap_or(0);
    let quadrant = input.quadrant.as_deref().unwrap_or("important_not_urgent");
    let prompt = prompt_templates::task_feedback_prompt(
        &input.task_name,
        estimated,
        actual,
        quadrant,
        &tone,
        &custom,
    );
    let messages = vec![Message { role: "user".into(), content: prompt }];
    let opts = intensity_to_opts(intensity, 150);
    let result = ai.complete(messages, opts).await?;
    if result.contains("\"message\"") {
        Ok(result)
    } else {
        Ok(format!(
            "{{\"message\":\"「{}」完成了，继续保持！\",\"badge\":\"✅\",\"tone\":\"encouraging\"}}",
            input.task_name
        ))
    }
}

// ---------- AI 里程碑拆解 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneBreakdownInput {
    pub goal_name: String,
    pub goal_description: Option<String>,
    pub total_deadline: Option<String>,
    pub current_progress: Option<String>,
}

#[tauri::command]
pub async fn ai_milestone_breakdown(
    input: MilestoneBreakdownInput,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<String> {
    {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;
    }
    let description = input.goal_description.as_deref().unwrap_or("");
    let deadline = input.total_deadline.as_deref().unwrap_or("未设定");
    let progress = input.current_progress.as_deref().unwrap_or("尚未开始");
    let prompt = prompt_templates::milestone_breakdown_prompt(
        &input.goal_name,
        description,
        deadline,
        progress,
    );
    let messages = vec![Message { role: "user".into(), content: prompt }];
    // 里程碑拆解固定稍高 temperature 以保持创意，但不受 intensity 影响
    let result = ai
        .complete(messages, CompletionOptions { temperature: Some(0.7), max_tokens: Some(1000) })
        .await?;
    if result.contains("\"milestones\"") {
        Ok(result)
    } else {
        Err(AppError::Custom("AI 返回格式异常，请重试".into()))
    }
}

// ---------- AI 周度小结 ----------

#[tauri::command]
pub async fn ai_weekly_summary(
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<String> {
    let (focus_min, pomodoros, completed, avg_grade, top_task, tone, intensity) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;

        let (fm, pm): (i64, i64) = conn
            .query_row(
                "SELECT COALESCE(SUM(actual_duration_minutes),0), COUNT(*) FROM sessions
                 WHERE status='completed' AND start_time >= datetime('now','-7 days')",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap_or((0, 0));

        let ct: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tasks WHERE status='completed' AND updated_at >= datetime('now','-7 days')",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let ag: String = conn
            .query_row(
                "SELECT COALESCE(
                    (SELECT grade FROM settlements ORDER BY settle_date DESC LIMIT 1),
                    'B'
                )",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "B".into());

        let tt: String = conn
            .query_row(
                "SELECT COALESCE(t.name, '无')
                 FROM sessions s JOIN tasks t ON t.id = s.task_id
                 WHERE s.start_time >= datetime('now','-7 days')
                 GROUP BY s.task_id ORDER BY SUM(s.actual_duration_minutes) DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "无".into());

        let (t, _c) = load_tone(&conn);
        let i = load_intensity(&conn);
        (fm, pm, ct, ag, tt, t, i)
    };

    let prompt = prompt_templates::weekly_summary_prompt(
        focus_min, pomodoros, completed, &avg_grade, &top_task, &tone,
    );
    let messages = vec![Message { role: "user".into(), content: prompt }];
    let opts = intensity_to_opts(intensity, 300);
    let result = ai.complete(messages, opts).await?;
    Ok(result)
}

// ---------- AI 任务预估时长 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EstimateTaskDurationInput {
    pub task_name: String,
    pub description: Option<String>,
}

#[tauri::command]
pub async fn ai_estimate_task_duration(
    input: EstimateTaskDurationInput,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<String> {
    let intensity = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;
        load_intensity(&conn)
    };

    // 查询历史相似任务的实际用时（按任务名相似度，取最近 10 条）
    let similar_history: String = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let task_keyword = format!("%{}%", &input.task_name.chars().take(10).collect::<String>());
        let mut stmt = conn
            .prepare(
                "SELECT t.name, COALESCE(SUM(s.actual_duration_minutes), 0) as total_min
                 FROM tasks t
                 JOIN sessions s ON s.task_id = t.id
                 WHERE t.status = 'completed'
                   AND s.status = 'completed'
                   AND (t.name LIKE ?1 OR t.estimated_minutes IS NOT NULL)
                 GROUP BY t.id
                 ORDER BY t.updated_at DESC
                 LIMIT 10",
            )
            .map_err(|e| AppError::Custom(e.to_string()))?;
        let rows: Vec<String> = stmt
            .query_map(rusqlite::params![task_keyword], |r| {
                let name: String = r.get(0)?;
                let mins: i64 = r.get(1)?;
                Ok(format!("  - {name} → {mins} 分钟"))
            })
            .map_err(|e| AppError::Custom(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        if rows.is_empty() {
            "  （暂无历史数据）".to_string()
        } else {
            rows.join("\n")
        }
    };

    let description = input.description.as_deref().unwrap_or("");
    let prompt = prompt_templates::task_duration_prompt(
        &input.task_name,
        description,
        &similar_history,
    );
    let messages = vec![Message { role: "user".into(), content: prompt }];
    let opts = intensity_to_opts(intensity, 200);
    let result = ai.complete(messages, opts).await?;
    // 校验：必须包含 estimated_minutes
    if result.contains("\"estimated_minutes\"") {
        Ok(result)
    } else {
        Ok("{\"estimated_minutes\":30,\"confidence\":\"low\",\"reasoning\":\"AI返回格式异常，已使用默认值\",\"range\":{\"min\":15,\"max\":60}}".to_string())
    }
}

// ---------- AI 里程碑风险预警 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneRiskInput {
    pub milestone_name: String,
    pub goal_name: String,
    pub target_date: String,
    pub remaining_days: i64,
    pub done_subtasks: i64,
    pub total_subtasks: i64,
    /// 近 7 天该里程碑关联任务的专注时长描述
    pub milestone_id: Option<String>,
}

#[tauri::command]
pub async fn ai_milestone_risk(
    input: MilestoneRiskInput,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<String> {
    let intensity = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;
        load_intensity(&conn)
    };

    // 查询近 7 天该里程碑关联任务的专注分钟数
    let recent_activity: String = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        if let Some(ref ms_id) = input.milestone_id {
            let mins: i64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(s.actual_duration_minutes), 0)
                     FROM sessions s
                     JOIN tasks t ON t.id = s.task_id
                     WHERE t.milestone_id = ?1
                       AND s.status = 'completed'
                       AND s.start_time >= datetime('now', '-7 days')",
                    rusqlite::params![ms_id],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            format!("{mins} 分钟")
        } else {
            "暂无记录".to_string()
        }
    };

    let prompt = prompt_templates::milestone_risk_prompt(
        &input.milestone_name,
        &input.goal_name,
        &input.target_date,
        input.remaining_days,
        input.done_subtasks,
        input.total_subtasks,
        &recent_activity,
    );
    let messages = vec![Message { role: "user".into(), content: prompt }];
    let opts = intensity_to_opts(intensity, 300);
    let result = ai.complete(messages, opts).await?;
    // 校验：必须含 risk_level
    if result.contains("\"risk_level\"") {
        Ok(result)
    } else {
        Ok("{\"risk_level\":\"high\",\"summary\":\"当前进度偏慢，存在延期风险\",\"actions\":[\"优先完成最核心子任务\",\"评估是否可调整截止日期\"]}".to_string())
    }
}

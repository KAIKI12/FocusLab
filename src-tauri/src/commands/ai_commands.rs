//! AI 命令 — 配置 / 测试连接 / 任务拆解 / 结算叙事。

use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::ai::prompt_templates;
use crate::ai::{local_quotes, AIService, CompletionOptions, Message, ResponseValidator};
use crate::db::Db;
use crate::models::settings;
use crate::utils::errors::{AppError, AppResult};

/// 抠出字符串中第一段 `{...}` JSON 对象;失败返回原文。
fn extract_json_blob(raw: &str) -> String {
    let s = raw.trim();
    if let (Some(start), Some(end)) = (s.find('{'), s.rfind('}')) {
        if end > start {
            return s[start..=end].to_string();
        }
    }
    s.to_string()
}

fn get_setting_or(conn: &rusqlite::Connection, key: &str, default: &str) -> String {
    settings::get(conn, key)
        .ok()
        .flatten()
        .unwrap_or_else(|| default.to_string())
}

fn check_ai_enabled(conn: &rusqlite::Connection) -> AppResult<()> {
    let enabled = get_setting_or(conn, "ai_enabled", "1");
    if enabled == "0" {
        Err(AppError::Custom("AI 功能已关闭，请在设置中开启".into()))
    } else {
        Ok(())
    }
}

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
        model_override: None,
    }
}

fn load_tone(conn: &rusqlite::Connection) -> (String, String) {
    let tone = get_setting_or(conn, "ai_tone", "academic");
    let custom = get_setting_or(conn, "ai_tone_custom", "");
    (tone, custom)
}

fn load_intensity(conn: &rusqlite::Connection) -> i32 {
    let raw = get_setting_or(conn, "ai_tone_intensity", "3");
    raw.parse::<i32>().unwrap_or(3).clamp(1, 5)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AiSceneModelTier {
    Fast,
    Strong,
}

fn scene_tier(scene: &str) -> AiSceneModelTier {
    match scene {
        "ai_classify_quadrant"
        | "ai_settlement_narrative"
        | "ai_optimize_quick_note"
        | "ai_unfinished_reminder"
        | "ai_task_feedback"
        | "test_ai_connection"
        | "inspiration_rerank"
        | "ai_suggest_goal_for_inspiration"
        | "ai_draft_followup_experiment"
        | "ai_analyze_correction" => AiSceneModelTier::Fast,
        "ai_decompose_task"
        | "ai_daily_suggestions"
        | "ai_milestone_breakdown"
        | "ai_weekly_summary"
        | "ai_estimate_task_duration"
        | "ai_milestone_risk" => AiSceneModelTier::Strong,
        _ => AiSceneModelTier::Fast,
    }
}

fn resolve_model_for_tier(
    tier: AiSceneModelTier,
    fast_model: Option<&str>,
    strong_model: Option<&str>,
    legacy_model: Option<&str>,
) -> String {
    let preferred = match tier {
        AiSceneModelTier::Fast => fast_model,
        AiSceneModelTier::Strong => strong_model,
    }
    .filter(|value| !value.trim().is_empty());

    let legacy = legacy_model.filter(|value| !value.trim().is_empty());

    preferred.or(legacy).unwrap_or_default().to_string()
}

fn load_model_for_scene(conn: &rusqlite::Connection, scene: &str) -> String {
    resolve_model_for_tier(
        scene_tier(scene),
        settings::get(conn, "ai_model_fast")
            .ok()
            .flatten()
            .as_deref(),
        settings::get(conn, "ai_model_strong")
            .ok()
            .flatten()
            .as_deref(),
        settings::get(conn, "ai_model").ok().flatten().as_deref(),
    )
}

fn with_model_override(opts: CompletionOptions, model: String) -> CompletionOptions {
    if model.trim().is_empty() {
        return opts;
    }
    CompletionOptions {
        model_override: Some(model),
        ..opts
    }
}

#[tauri::command]
pub async fn configure_ai(
    provider: String,
    api_format: String,
    base_url: String,
    api_key: String,
    model: String,
    fast_model: Option<String>,
    strong_model: Option<String>,
    enabled: Option<String>,
    tone: Option<String>,
    tone_custom: Option<String>,
    intensity: Option<String>,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<()> {
    let effective_fast_model = fast_model
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(&model)
        .to_string();
    let effective_strong_model = strong_model
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(&model)
        .to_string();
    let effective_model = if model.trim().is_empty() {
        effective_fast_model.clone()
    } else {
        model.clone()
    };

    {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let now = chrono::Utc::now().to_rfc3339();
        let mut pairs: Vec<(&str, String)> = vec![
            ("ai_provider", provider.clone()),
            ("ai_api_format", api_format.clone()),
            ("ai_base_url", base_url.clone()),
            ("ai_api_key", api_key.clone()),
            ("ai_model", effective_model.clone()),
            ("ai_model_fast", effective_fast_model.clone()),
            ("ai_model_strong", effective_strong_model.clone()),
        ];
        if let Some(v) = &enabled {
            pairs.push(("ai_enabled", v.clone()));
        }
        if let Some(v) = &tone {
            pairs.push(("ai_tone", v.clone()));
        }
        if let Some(v) = &tone_custom {
            pairs.push(("ai_tone_custom", v.clone()));
        }
        if let Some(v) = &intensity {
            pairs.push(("ai_tone_intensity", v.clone()));
        }
        for (k, v) in &pairs {
            conn.execute(
                "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![k, v, now],
            )?;
        }
    }

    ai.configure(
        &provider,
        &api_format,
        &base_url,
        &api_key,
        &effective_model,
    )
    .await;
    Ok(())
}

#[tauri::command]
pub async fn configure_embedding(
    base_url: String,
    api_key: String,
    model: String,
    enabled: Option<String>,
    app: AppHandle,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<()> {
    // B3: 检测 embedding model 是否变化,变了就重建索引。
    // 只比较 model;base_url/api_key 变化不一定意味着维度变化(可能是同 model 换 endpoint)。
    let model_changed = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let prev_model = settings::get(&conn, "ai_embedding_model")?.unwrap_or_default();
        !prev_model.is_empty() && prev_model != model
    };
    {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let now = chrono::Utc::now().to_rfc3339();
        let mut pairs: Vec<(&str, String)> = vec![
            ("ai_embedding_base_url", base_url.clone()),
            ("ai_embedding_api_key", api_key.clone()),
            ("ai_embedding_model", model.clone()),
        ];
        if let Some(v) = enabled {
            pairs.push(("ai_embedding_enabled", v));
        }
        for (k, v) in &pairs {
            conn.execute(
                "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![k, v, now],
            )?;
        }

        // B3: model 变化时,清空旧向量并把所有灵感置 pending,等待重新索引。
        if model_changed {
            conn.execute("DELETE FROM inspiration_embeddings", [])?;
            conn.execute(
                "UPDATE inspirations SET embedding_status = 'pending', updated_at = ?1",
                params![now],
            )?;
        }
    }
    ai.configure_embedding(&base_url, &api_key, &model).await;

    if model_changed {
        // 通知前端: 索引已失效,可调 batch_embed_pending 重建。
        let _ = app.emit(
            "inspiration://embedding_model_changed",
            serde_json::json!({ "newModel": model }),
        );
    }
    Ok(())
}

async fn request_ai_connection(ai: &AIService, scene_model: String) -> AppResult<String> {
    let messages = vec![Message {
        role: "user".into(),
        content: "请回复'连接成功'四个字。".into(),
    }];
    ai.complete_required(
        messages,
        with_model_override(
            CompletionOptions {
                max_tokens: Some(20),
                ..Default::default()
            },
            scene_model,
        ),
    )
    .await
}

#[tauri::command]
pub async fn test_ai_connection(ai: State<'_, AIService>, db: State<'_, Db>) -> AppResult<String> {
    let scene_model = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        load_model_for_scene(&conn, "test_ai_connection")
    };
    let result = request_ai_connection(&ai, scene_model).await?;
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

// keep the rest of file unchanged

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
    let scene_model = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;
        load_model_for_scene(&conn, "ai_decompose_task")
    };
    let prompt = prompt_templates::decompose_prompt(
        &input.task_name,
        input.description.as_deref().unwrap_or(""),
    );
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let result = ai
        .complete(
            messages,
            with_model_override(
                CompletionOptions {
                    temperature: Some(0.7),
                    max_tokens: Some(2000),
                    ..Default::default()
                },
                scene_model,
            ),
        )
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
    let (enabled, tone, _custom, intensity, scene_model) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let enabled = get_setting_or(&conn, "ai_enabled", "1") != "0";
        let (t, c) = load_tone(&conn);
        let i = load_intensity(&conn);
        let m = load_model_for_scene(&conn, "ai_settlement_narrative");
        (enabled, t, c, i, m)
    };
    // 关闭 AI 时直接走本地语录,保持文案非空
    if !enabled {
        return Ok(local_quotes::pick_settlement_narrative(
            &input.grade,
            input.completed,
            input.total,
            input.focus_minutes,
        ));
    }
    // 前端传入的 tone 优先级高于 settings（向后兼容）
    let effective_tone = input.tone.as_deref().unwrap_or(&tone);
    let prompt = prompt_templates::settlement_prompt(
        &input.grade,
        input.completed,
        input.total,
        input.focus_minutes,
        effective_tone,
    );
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let opts = intensity_to_opts(intensity, 200);
    match ai
        .complete(messages, with_model_override(opts, scene_model))
        .await
    {
        Ok(result) => Ok(ResponseValidator::validate_narrative(&result)),
        Err(_) => Ok(local_quotes::pick_settlement_narrative(
            &input.grade,
            input.completed,
            input.total,
            input.focus_minutes,
        )),
    }
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
    let (enabled, yesterday_summary, pending_tasks, tone, intensity, scene_model) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let enabled = get_setting_or(&conn, "ai_enabled", "1") != "0";

        let yesterday: String = conn
            .query_row(
                "SELECT COALESCE(grade || ' 级, 完成 ' || completed_tasks || '/' || total_tasks || ' 项, 专注 ' || total_focus_minutes || ' 分钟', '无昨日数据')
                 FROM settlements ORDER BY settle_date DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "无昨日数据".into());

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
        let pending = if names.is_empty() {
            "无待办任务".into()
        } else {
            names.join("、")
        };

        let (t, _c) = load_tone(&conn);
        let i = load_intensity(&conn);
        let m = load_model_for_scene(&conn, "ai_daily_suggestions");
        (enabled, yesterday, pending, t, i, m)
    };

    let energy = input.energy_level.as_deref().unwrap_or("正常");
    let local_fallback =
        || local_quotes::pick_daily_suggestion(energy, &pending_tasks, &yesterday_summary);
    if !enabled {
        return Ok(local_fallback());
    }
    let prompt = prompt_templates::daily_suggestion_prompt(
        &yesterday_summary,
        &pending_tasks,
        energy,
        &tone,
    );
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let opts = intensity_to_opts(intensity, 400);
    match ai
        .complete(messages, with_model_override(opts, scene_model))
        .await
    {
        Ok(result) if !result.trim().is_empty() => Ok(result),
        _ => Ok(local_fallback()),
    }
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
    let scene_model = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;
        load_model_for_scene(&conn, "ai_classify_quadrant")
    };
    let prompt = prompt_templates::classify_quadrant_prompt(
        &input.task_name,
        input.description.as_deref().unwrap_or(""),
    );
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let result = ai
        .complete(
            messages,
            with_model_override(
                CompletionOptions {
                    temperature: Some(0.3),
                    max_tokens: Some(50),
                    ..Default::default()
                },
                scene_model,
            ),
        )
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
    let (intensity, scene_model) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;
        (
            load_intensity(&conn),
            load_model_for_scene(&conn, "ai_optimize_quick_note"),
        )
    };
    let prompt = prompt_templates::quick_note_optimization_prompt(&input.raw_text);
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let opts = intensity_to_opts(intensity, 2000);
    let result = ai
        .complete_required(messages, with_model_override(opts, scene_model))
        .await?;
    ResponseValidator::validate_quick_note(&result)
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
    let (enabled, tone, custom, intensity) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let enabled = get_setting_or(&conn, "ai_enabled", "1") != "0";
        let (t, c) = load_tone(&conn);
        let i = load_intensity(&conn);
        (enabled, t, c, i)
    };
    let available_time = input.available_time.as_deref().unwrap_or("待定");
    let local_fallback = || {
        local_quotes::pick_unfinished_reminder(
            &input.unfinished_tasks,
            &input.completed_summary,
            available_time,
        )
    };
    if !enabled {
        return Ok(local_fallback());
    }
    let prompt = prompt_templates::unfinished_reminder_prompt(
        &input.unfinished_tasks,
        &input.completed_summary,
        available_time,
        &tone,
        &custom,
    );
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let opts = intensity_to_opts(intensity, 200);
    match ai.complete(messages, opts).await {
        Ok(result) if result.contains("\"message\"") => Ok(result),
        _ => Ok(local_fallback()),
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
    let (enabled, tone, custom, intensity) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let enabled = get_setting_or(&conn, "ai_enabled", "1") != "0";
        let (t, c) = load_tone(&conn);
        let i = load_intensity(&conn);
        (enabled, t, c, i)
    };
    let estimated = input.estimated_minutes.unwrap_or(0);
    let actual = input.actual_minutes.unwrap_or(0);
    let quadrant = input.quadrant.as_deref().unwrap_or("important_not_urgent");
    let local_fallback =
        || local_quotes::pick_task_feedback(&input.task_name, estimated, actual, quadrant);
    if !enabled {
        return Ok(local_fallback());
    }
    let prompt = prompt_templates::task_feedback_prompt(
        &input.task_name,
        estimated,
        actual,
        quadrant,
        &tone,
        &custom,
    );
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let opts = intensity_to_opts(intensity, 150);
    match ai.complete(messages, opts).await {
        Ok(result) if result.contains("\"message\"") => Ok(result),
        _ => Ok(local_fallback()),
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
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    // 里程碑拆解固定稍高 temperature 以保持创意，但不受 intensity 影响
    let result = ai
        .complete(
            messages,
            CompletionOptions {
                temperature: Some(0.7),
                max_tokens: Some(2500),
                ..Default::default()
            },
        )
        .await?;
    if result.contains("\"milestones\"") {
        Ok(result)
    } else {
        Err(AppError::Custom("AI 返回格式异常，请重试".into()))
    }
}

// ---------- AI 周度小结 ----------

#[tauri::command]
pub async fn ai_weekly_summary(ai: State<'_, AIService>, db: State<'_, Db>) -> AppResult<String> {
    let (enabled, focus_min, pomodoros, completed, avg_grade, top_task, tone, intensity) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let enabled = get_setting_or(&conn, "ai_enabled", "1") != "0";

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
        (enabled, fm, pm, ct, ag, tt, t, i)
    };

    let local_fallback = || {
        local_quotes::pick_weekly_summary(focus_min, pomodoros, completed, &avg_grade, &top_task)
    };
    if !enabled {
        return Ok(local_fallback());
    }
    let prompt = prompt_templates::weekly_summary_prompt(
        focus_min, pomodoros, completed, &avg_grade, &top_task, &tone,
    );
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let opts = intensity_to_opts(intensity, 300);
    match ai.complete(messages, opts).await {
        Ok(result) if !result.trim().is_empty() => Ok(result),
        _ => Ok(local_fallback()),
    }
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
        let task_keyword = format!(
            "%{}%",
            &input.task_name.chars().take(10).collect::<String>()
        );
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
    let prompt =
        prompt_templates::task_duration_prompt(&input.task_name, description, &similar_history);
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let opts = intensity_to_opts(intensity, 800);
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
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let opts = intensity_to_opts(intensity, 800);
    let result = ai.complete(messages, opts).await?;
    // 校验：必须含 risk_level
    if result.contains("\"risk_level\"") {
        Ok(result)
    } else {
        Ok("{\"risk_level\":\"high\",\"summary\":\"当前进度偏慢，存在延期风险\",\"actions\":[\"优先完成最核心子任务\",\"评估是否可调整截止日期\"]}".to_string())
    }
}

// ---------- 获取可用模型列表 ----------

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelEntry>,
}

#[derive(Deserialize, Serialize)]
struct ModelEntry {
    id: String,
}

#[tauri::command]
pub async fn fetch_ai_models(
    base_url: String,
    api_key: String,
    api_format: String,
) -> AppResult<Vec<String>> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/models", base_url.trim_end_matches('/'));

    let mut req = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .header("User-Agent", "FocusLab/1.0");

    if api_format == "claude" {
        req = req
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01");
    } else if !api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", api_key));
    }

    let resp = req
        .send()
        .await
        .map_err(|e| AppError::Custom(format!("请求失败: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::Custom(format!("获取模型失败 {status}: {text}")));
    }

    let data: ModelsResponse = resp
        .json()
        .await
        .map_err(|e| AppError::Custom(format!("解析模型列表失败: {e}")))?;

    let mut models: Vec<String> = data.data.into_iter().map(|m| m.id).collect();
    models.sort();
    Ok(models)
}

// ---------- AI 灵感: 推荐归属目标 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestGoalInput {
    pub inspiration_content: String,
    /// 候选目标列表 (id, name)
    pub goals: Vec<(String, String)>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestGoalResult {
    pub goal_id: Option<String>,
    pub reason: String,
}

#[tauri::command]
pub async fn ai_suggest_goal_for_inspiration(
    input: SuggestGoalInput,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<SuggestGoalResult> {
    let scene_model = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;
        load_model_for_scene(&conn, "ai_suggest_goal_for_inspiration")
    };
    if input.goals.is_empty() {
        return Ok(SuggestGoalResult {
            goal_id: None,
            reason: "暂无可选目标".into(),
        });
    }
    let goal_lines: String = input
        .goals
        .iter()
        .map(|(id, name)| format!("{id}|{name}"))
        .collect::<Vec<_>>()
        .join("\n");
    let prompt = prompt_templates::suggest_goal_prompt(&input.inspiration_content, &goal_lines);
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let raw = ai
        .complete(
            messages,
            with_model_override(
                CompletionOptions {
                    temperature: Some(0.3),
                    max_tokens: Some(200),
                    ..Default::default()
                },
                scene_model,
            ),
        )
        .await?;

    let trimmed = raw.trim();
    let json_text = extract_json_blob(trimmed);
    let parsed: serde_json::Value = serde_json::from_str(&json_text)
        .map_err(|e| AppError::Custom(format!("AI 返回解析失败: {e}; 原文: {raw}")))?;
    let raw_goal = parsed
        .get("goalId")
        .and_then(|v| v.as_str())
        .unwrap_or("none")
        .trim()
        .to_string();
    let reason = parsed
        .get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let valid_ids: std::collections::HashSet<&str> =
        input.goals.iter().map(|(id, _)| id.as_str()).collect();
    let goal_id =
        if raw_goal == "none" || raw_goal.is_empty() || !valid_ids.contains(raw_goal.as_str()) {
            None
        } else {
            Some(raw_goal)
        };
    Ok(SuggestGoalResult { goal_id, reason })
}

// ---------- AI 灵感: 起草后续实验 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftFollowupInput {
    pub parent_content: String,
}

#[tauri::command]
pub async fn ai_draft_followup_experiment(
    input: DraftFollowupInput,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<String> {
    let scene_model = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;
        load_model_for_scene(&conn, "ai_draft_followup_experiment")
    };
    let prompt = prompt_templates::draft_followup_prompt(&input.parent_content);
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let raw = ai
        .complete(
            messages,
            with_model_override(
                CompletionOptions {
                    temperature: Some(0.6),
                    max_tokens: Some(200),
                    ..Default::default()
                },
                scene_model,
            ),
        )
        .await?;
    let mut text = raw.trim().trim_matches('"').trim().to_string();
    if !text.starts_with("[后续实验]") {
        text = format!("[后续实验] {text}");
    }
    Ok(text)
}

// ---------- AI 灵感: 纠偏分析 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeCorrectionInput {
    pub old_content: String,
    pub new_content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeCorrectionResult {
    pub summary: String,
    pub old_judgment: String,
    pub new_evidence: String,
    pub suggestion: String,
}

#[tauri::command]
pub async fn ai_analyze_correction(
    input: AnalyzeCorrectionInput,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<AnalyzeCorrectionResult> {
    let scene_model = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        check_ai_enabled(&conn)?;
        load_model_for_scene(&conn, "ai_analyze_correction")
    };
    let prompt =
        prompt_templates::correction_analysis_prompt(&input.old_content, &input.new_content);
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let raw = ai
        .complete(
            messages,
            with_model_override(
                CompletionOptions {
                    temperature: Some(0.4),
                    max_tokens: Some(400),
                    ..Default::default()
                },
                scene_model,
            ),
        )
        .await?;
    let trimmed = raw.trim();
    let json_text = extract_json_blob(trimmed);
    let parsed: serde_json::Value = serde_json::from_str(&json_text)
        .map_err(|e| AppError::Custom(format!("AI 返回解析失败: {e}; 原文: {raw}")))?;
    Ok(AnalyzeCorrectionResult {
        summary: parsed
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        old_judgment: parsed
            .get("oldJudgment")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        new_evidence: parsed
            .get("newEvidence")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        suggestion: parsed
            .get("suggestion")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

#[cfg(test)]
mod tests {
    use crate::ai::AIService;

    #[tokio::test]
    async fn connection_test_fails_without_provider_instead_of_using_offline_fallback() {
        let ai = AIService::new();
        let result = super::request_ai_connection(&ai, String::new()).await;

        assert!(result.is_err());
    }
}

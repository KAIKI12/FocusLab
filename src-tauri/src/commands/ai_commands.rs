//! AI 命令 — 配置 / 测试连接 / 任务拆解 / 结算叙事。

use rusqlite::params;
use serde::Deserialize;
use tauri::State;

use crate::ai::prompt_templates;
use crate::ai::{AIService, CompletionOptions, Message};
use crate::db::Db;
use crate::utils::errors::{AppError, AppResult};

/// 配置 AI provider(从 settings KV 读取或前端直接传入)
#[tauri::command]
pub async fn configure_ai(
    provider: String,
    base_url: String,
    api_key: String,
    model: String,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<()> {
    // 持久化到 settings
    {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let now = chrono::Utc::now().to_rfc3339();
        for (k, v) in [
            ("ai_provider", provider.as_str()),
            ("ai_base_url", base_url.as_str()),
            ("ai_api_key", api_key.as_str()),
            ("ai_model", model.as_str()),
        ] {
            conn.execute(
                "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![k, v, now],
            )?;
        }
    }

    ai.configure(&provider, &base_url, &api_key, &model).await;
    tracing::info!("AI configured: provider={provider} model={model}");
    Ok(())
}

/// 测试 AI 连接
#[tauri::command]
pub async fn test_ai_connection(ai: State<'_, AIService>) -> AppResult<String> {
    let messages = vec![Message {
        role: "user".into(),
        content: "请回复'连接成功'四个字。".into(),
    }];
    let result = ai
        .complete(messages, CompletionOptions { max_tokens: Some(20), ..Default::default() })
        .await?;
    Ok(result)
}

/// AI 任务拆解
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
) -> AppResult<String> {
    let prompt = prompt_templates::decompose_prompt(
        &input.task_name,
        input.description.as_deref().unwrap_or(""),
    );
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let result = ai
        .complete(messages, CompletionOptions {
            temperature: Some(0.7),
            max_tokens: Some(800),
        })
        .await?;
    Ok(result)
}

/// AI 结算叙事
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
) -> AppResult<String> {
    let tone = input.tone.as_deref().unwrap_or("academic");
    let prompt = prompt_templates::settlement_prompt(
        &input.grade,
        input.completed,
        input.total,
        input.focus_minutes,
        tone,
    );
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let result = ai
        .complete(messages, CompletionOptions {
            temperature: Some(0.8),
            max_tokens: Some(200),
        })
        .await?;
    Ok(result)
}

// ---------- AI 每日建议 + 四象限分类 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailySuggestionInput {
    pub energy_level: Option<String>,
}

/// AI 每日建议 — 根据昨日情况 + 待办任务推荐今日优先级
#[tauri::command]
pub async fn ai_daily_suggestions(
    input: DailySuggestionInput,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<String> {
    let (yesterday_summary, pending_tasks) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;

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
        let pending = if names.is_empty() {
            "无待办任务".into()
        } else {
            names.join("、")
        };

        (yesterday, pending)
    };

    let energy = input.energy_level.as_deref().unwrap_or("正常");
    let prompt = prompt_templates::daily_suggestion_prompt(&yesterday_summary, &pending_tasks, energy);
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let result = ai
        .complete(messages, CompletionOptions {
            temperature: Some(0.7),
            max_tokens: Some(400),
        })
        .await?;
    Ok(result)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassifyInput {
    pub task_name: String,
    pub description: Option<String>,
}

/// AI 四象限自动分类 — 返回象限标识字符串
#[tauri::command]
pub async fn ai_classify_quadrant(
    input: ClassifyInput,
    ai: State<'_, AIService>,
) -> AppResult<String> {
    let prompt = prompt_templates::classify_quadrant_prompt(
        &input.task_name,
        input.description.as_deref().unwrap_or(""),
    );
    let messages = vec![Message {
        role: "user".into(),
        content: prompt,
    }];
    let result = ai
        .complete(messages, CompletionOptions {
            temperature: Some(0.3),
            max_tokens: Some(50),
        })
        .await?;
    // 规范化返回值
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

/// AI 周度小结
#[tauri::command]
pub async fn ai_weekly_summary(
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<String> {
    let (focus_min, pomodoros, completed, avg_grade, top_task) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;

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

        (fm, pm, ct, ag, tt)
    };

    let prompt = prompt_templates::weekly_summary_prompt(
        focus_min, pomodoros, completed, &avg_grade, &top_task,
    );
    let messages = vec![Message { role: "user".into(), content: prompt }];
    let result = ai
        .complete(messages, CompletionOptions { temperature: Some(0.7), max_tokens: Some(300) })
        .await?;
    Ok(result)
}

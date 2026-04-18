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

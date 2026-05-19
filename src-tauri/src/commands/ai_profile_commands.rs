//! AI Profile CRUD + 激活 - chat / embedding 双独立列表。
//!
//! 激活时:
//! 1. 写 active id 到 settings (`ai_active_chat_profile_id` / `ai_active_embedding_profile_id`)
//! 2. 把字段镜像回旧 `ai_*` keys (向后兼容,见 lib.rs 启动 hook)
//! 3. 调 AIService::configure / configure_embedding 立即更新运行态

use serde::Deserialize;
use tauri::State;

use crate::ai::AIService;
use crate::db::Db;
use crate::models::ai_profile::{self, ChatProfile, EmbeddingProfile};
use crate::utils::errors::{AppError, AppResult};

// ---------- list ----------

#[tauri::command]
pub async fn list_chat_profiles(db: State<'_, Db>) -> AppResult<Vec<ChatProfile>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    ai_profile::list_chat(&conn)
}

#[tauri::command]
pub async fn list_embedding_profiles(db: State<'_, Db>) -> AppResult<Vec<EmbeddingProfile>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    ai_profile::list_embedding(&conn)
}

#[tauri::command]
pub async fn get_active_chat_profile_id(db: State<'_, Db>) -> AppResult<Option<String>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    ai_profile::active_chat_id(&conn)
}

#[tauri::command]
pub async fn get_active_embedding_profile_id(db: State<'_, Db>) -> AppResult<Option<String>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    ai_profile::active_embedding_id(&conn)
}

// ---------- create ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChatProfileInput {
    pub name: String,
    pub provider: String,
    pub api_format: String,
    pub base_url: String,
    pub api_key: String,
    pub model_fast: String,
    pub model_strong: String,
    pub selected_models: String,
}

#[tauri::command]
pub async fn create_chat_profile(
    input: CreateChatProfileInput,
    db: State<'_, Db>,
) -> AppResult<String> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    ai_profile::insert_chat(
        &conn,
        &input.name,
        &input.provider,
        &input.api_format,
        &input.base_url,
        &input.api_key,
        &input.model_fast,
        &input.model_strong,
        &input.selected_models,
    )
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEmbeddingProfileInput {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[tauri::command]
pub async fn create_embedding_profile(
    input: CreateEmbeddingProfileInput,
    db: State<'_, Db>,
) -> AppResult<String> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    ai_profile::insert_embedding(
        &conn,
        &input.name,
        &input.base_url,
        &input.api_key,
        &input.model,
    )
}

// ---------- update ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChatProfileInput {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub api_format: String,
    pub base_url: String,
    pub api_key: String,
    pub model_fast: String,
    pub model_strong: String,
    pub selected_models: String,
}

#[tauri::command]
pub async fn update_chat_profile(
    input: UpdateChatProfileInput,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<()> {
    let active_now: Option<String> = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        ai_profile::update_chat(
            &conn,
            &input.id,
            &input.name,
            &input.provider,
            &input.api_format,
            &input.base_url,
            &input.api_key,
            &input.model_fast,
            &input.model_strong,
            &input.selected_models,
        )?;
        ai_profile::active_chat_id(&conn)?
    };
    // 编辑的是当前激活 profile 时,立即把新值同步给 AIService + 旧 keys
    if active_now.as_deref() == Some(input.id.as_str()) {
        {
            let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
            ai_profile::set_active_chat(&conn, &input.id)?;
        }
        let primary_model = if !input.model_fast.is_empty() {
            input.model_fast.clone()
        } else {
            input.model_strong.clone()
        };
        ai.configure(
            &input.provider,
            &input.api_format,
            &input.base_url,
            &input.api_key,
            &primary_model,
        )
        .await;
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEmbeddingProfileInput {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[tauri::command]
pub async fn update_embedding_profile(
    input: UpdateEmbeddingProfileInput,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<()> {
    let active_now: Option<String> = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        ai_profile::update_embedding(
            &conn,
            &input.id,
            &input.name,
            &input.base_url,
            &input.api_key,
            &input.model,
        )?;
        ai_profile::active_embedding_id(&conn)?
    };
    if active_now.as_deref() == Some(input.id.as_str()) {
        {
            let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
            ai_profile::set_active_embedding(&conn, &input.id)?;
        }
        ai.configure_embedding(&input.base_url, &input.api_key, &input.model)
            .await;
    }
    Ok(())
}

// ---------- delete ----------

#[tauri::command]
pub async fn delete_chat_profile(
    id: String,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<()> {
    let next_active: Option<ChatProfile> = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        ai_profile::delete_chat_with_failover(&conn, &id)?;
        match ai_profile::active_chat_id(&conn)? {
            Some(active_id) if !active_id.is_empty() => ai_profile::get_chat(&conn, &active_id)?,
            _ => None,
        }
    };
    if let Some(p) = next_active {
        let primary = if !p.model_fast.is_empty() {
            p.model_fast
        } else {
            p.model_strong
        };
        ai.configure(
            &p.provider,
            &p.api_format,
            &p.base_url,
            &p.api_key,
            &primary,
        )
        .await;
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_embedding_profile(
    id: String,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<()> {
    let next_active: Option<EmbeddingProfile> = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        ai_profile::delete_embedding_with_failover(&conn, &id)?;
        match ai_profile::active_embedding_id(&conn)? {
            Some(active_id) if !active_id.is_empty() => {
                ai_profile::get_embedding(&conn, &active_id)?
            }
            _ => None,
        }
    };
    if let Some(p) = next_active {
        ai.configure_embedding(&p.base_url, &p.api_key, &p.model)
            .await;
    }
    Ok(())
}

// ---------- activate ----------

#[tauri::command]
pub async fn activate_chat_profile(
    id: String,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<()> {
    let profile: ChatProfile = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        ai_profile::set_active_chat(&conn, &id)?;
        ai_profile::get_chat(&conn, &id)?
            .ok_or_else(|| AppError::Custom("chat profile 不存在".into()))?
    };
    let primary = if !profile.model_fast.is_empty() {
        profile.model_fast
    } else {
        profile.model_strong
    };
    ai.configure(
        &profile.provider,
        &profile.api_format,
        &profile.base_url,
        &profile.api_key,
        &primary,
    )
    .await;
    Ok(())
}

#[tauri::command]
pub async fn activate_embedding_profile(
    id: String,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<()> {
    let profile: EmbeddingProfile = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        ai_profile::set_active_embedding(&conn, &id)?;
        ai_profile::get_embedding(&conn, &id)?
            .ok_or_else(|| AppError::Custom("embedding profile 不存在".into()))?
    };
    ai.configure_embedding(&profile.base_url, &profile.api_key, &profile.model)
        .await;
    Ok(())
}

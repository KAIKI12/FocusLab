//! AI 聊天 - 会话与消息 CRUD 命令 + 流式 send/abort。

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tauri::State;
use tokio::sync::mpsc;

use crate::ai::streaming::StreamChunk;
use crate::ai::{AIService, CompletionOptions, Message};
use crate::db::Db;
use crate::models::ai_chat::{self, ChatMessage, Conversation};
use crate::models::ai_profile;
use crate::utils::errors::{AppError, AppResult};

/// 每条会话最多一个 in-flight 流。abort 信号通过 `Arc<AtomicBool>` 协作传递。
#[derive(Default)]
pub struct AbortRegistry(pub Mutex<HashMap<String, Arc<AtomicBool>>>);

impl AbortRegistry {
    fn insert(&self, conv_id: &str, flag: Arc<AtomicBool>) {
        if let Ok(mut g) = self.0.lock() {
            g.insert(conv_id.to_string(), flag);
        }
    }
    fn remove(&self, conv_id: &str) {
        if let Ok(mut g) = self.0.lock() {
            g.remove(conv_id);
        }
    }
    fn signal(&self, conv_id: &str) -> bool {
        if let Ok(g) = self.0.lock() {
            if let Some(flag) = g.get(conv_id) {
                flag.store(true, Ordering::Relaxed);
                return true;
            }
        }
        false
    }
}

#[tauri::command]
pub async fn list_conversations(
    include_archived: Option<bool>,
    db: State<'_, Db>,
) -> AppResult<Vec<Conversation>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    ai_chat::list_conversations(&conn, include_archived.unwrap_or(false))
}

#[tauri::command]
pub async fn get_conversation(id: String, db: State<'_, Db>) -> AppResult<Option<Conversation>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    ai_chat::get_conversation(&conn, &id)
}

#[tauri::command]
pub async fn list_chat_messages(
    conversation_id: String,
    db: State<'_, Db>,
) -> AppResult<Vec<ChatMessage>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    ai_chat::list_messages(&conn, &conversation_id)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConversationInput {
    pub title: Option<String>,
    pub origin_type: Option<String>,
    pub origin_id: Option<String>,
    pub system_prompt: Option<String>,
    pub model: Option<String>,
}

/// 创建空会话(不写任何 message)。provider/api_format 从当前 active chat profile 快照,
/// model 优先采用入参,其次会话级回退给 profile 默认。
#[tauri::command]
pub async fn create_conversation(
    input: CreateConversationInput,
    db: State<'_, Db>,
) -> AppResult<String> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let active = ai_profile::active_chat_id(&conn)?
        .filter(|s| !s.is_empty())
        .and_then(|id| ai_profile::get_chat(&conn, &id).ok().flatten());
    let (provider, api_format) = match active.as_ref() {
        Some(p) => (p.provider.clone(), p.api_format.clone()),
        None => (String::new(), String::new()),
    };
    let model = input.model.unwrap_or_default();
    let title = input.title.unwrap_or_else(|| "新会话".to_string());
    let origin_type = input.origin_type.unwrap_or_else(|| "manual".to_string());
    let system_prompt = input.system_prompt.unwrap_or_default();

    ai_chat::insert_conversation(
        &conn,
        &title,
        &origin_type,
        input.origin_id.as_deref(),
        &provider,
        &api_format,
        &model,
        &system_prompt,
    )
}

#[tauri::command]
pub async fn rename_conversation(id: String, title: String, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    ai_chat::rename_conversation(&conn, &id, &title)?;
    Ok(())
}

#[tauri::command]
pub async fn set_conversation_model(id: String, model: String, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    ai_chat::set_conversation_model(&conn, &id, &model)?;
    Ok(())
}

#[tauri::command]
pub async fn pin_conversation(id: String, pinned: bool, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    ai_chat::set_conversation_pinned(&conn, &id, pinned)?;
    Ok(())
}

#[tauri::command]
pub async fn archive_conversation(
    id: String,
    archived: bool,
    db: State<'_, Db>,
) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    ai_chat::set_conversation_archived(&conn, &id, archived)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_conversation(id: String, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    ai_chat::delete_conversation(&conn, &id)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_all_conversations(db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    ai_chat::delete_all_conversations(&conn)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_chat_message(id: String, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    ai_chat::delete_message(&conn, &id)?;
    Ok(())
}

/// 当前是否已配置可用的 chat provider(给前端发送前自检用)。
#[tauri::command]
pub async fn ai_chat_is_configured(ai: State<'_, AIService>) -> AppResult<bool> {
    Ok(ai.is_configured().await)
}

// ---------- 流式发送 ----------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResult {
    pub user_message_id: String,
    pub assistant_message_id: String,
    /// 最终状态: "ok" | "aborted" | "error"
    pub status: String,
    /// 失败时的错误描述
    pub error: Option<String>,
}

/// 装上下文: system_prompt(若非空) + 历史 user/assistant + 新 user。
fn build_context(conv: &Conversation, history: &[ChatMessage], new_user: &str) -> Vec<Message> {
    let mut out: Vec<Message> = Vec::with_capacity(history.len() + 2);
    if !conv.system_prompt.trim().is_empty() {
        out.push(Message {
            role: "system".into(),
            content: conv.system_prompt.clone(),
        });
    }
    for m in history {
        if m.role == "user" || m.role == "assistant" {
            // 忽略空 assistant(不可能,但防御)
            if m.role == "assistant" && m.content.is_empty() {
                continue;
            }
            out.push(Message {
                role: m.role.clone(),
                content: m.content.clone(),
            });
        }
    }
    out.push(Message {
        role: "user".into(),
        content: new_user.to_string(),
    });
    out
}

/// 标题自动生成: 首条 user 消息前 30 字(不含换行)。
fn auto_title_from(content: &str) -> String {
    let line: String = content
        .chars()
        .filter(|c| !c.is_control())
        .take(30)
        .collect();
    if line.is_empty() {
        "新会话".into()
    } else {
        line
    }
}

/// 发送一条 user 消息并启动流式 assistant 回复。
///
/// 流程:
/// 1. 写 user message
/// 2. 写占位 assistant message (status=streaming)
/// 3. 装上下文,调 ai.stream_required,把 delta 桥接到 Tauri Channel
/// 4. 完成 / 中止 / 失败时 finalize 该 assistant message,推 done/aborted/error 帧
///
/// 返回值在流完成后才返回 — 前端不必等,会通过 Channel 看到全部 delta。
#[tauri::command]
pub async fn send_chat_message(
    conversation_id: String,
    content: String,
    on_chunk: Channel<StreamChunk>,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
    registry: State<'_, AbortRegistry>,
) -> AppResult<SendMessageResult> {
    // ---- 写 user message + 拿上下文 ----
    let (conversation, history, user_message_id, assistant_message_id, model_for_msg) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let conv = ai_chat::get_conversation(&conn, &conversation_id)?
            .ok_or_else(|| AppError::Custom("会话不存在".into()))?;

        let history = ai_chat::list_messages(&conn, &conversation_id)?;

        let user_id = ai_chat::insert_message(&conn, &conversation_id, "user", &content, None, "ok")?;

        // 第一次有 user 消息且 title 还是默认值 → 自动改名
        if conv.title == "新会话" && history.iter().all(|m| m.role != "user") {
            let _ = ai_chat::rename_conversation(&conn, &conversation_id, &auto_title_from(&content));
        }

        let model_str = if conv.model.is_empty() { None } else { Some(conv.model.clone()) };
        let model_param: Option<&str> = model_str.as_deref();
        let assistant_id =
            ai_chat::insert_message(&conn, &conversation_id, "assistant", "", model_param, "streaming")?;

        (conv, history, user_id, assistant_id, model_str)
    };

    let messages = build_context(&conversation, &history, &content);

    // ---- 注册 abort flag ----
    let abort = Arc::new(AtomicBool::new(false));
    registry.insert(&conversation_id, abort.clone());

    // ---- mpsc 桥到 Tauri Channel ----
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let chan_clone = on_chunk.clone();
    let bridge = tokio::spawn(async move {
        while let Some(delta) = rx.recv().await {
            let _ = chan_clone.send(StreamChunk::delta(&delta));
        }
    });

    // ---- 调流式 ----
    let opts = CompletionOptions {
        temperature: Some(0.7),
        max_tokens: Some(2048),
        model_override: model_for_msg,
    };
    let stream_result = ai.stream_required(messages, opts, &tx, abort.clone()).await;

    // 让 bridge 退出
    drop(tx);
    let _ = bridge.await;

    // ---- 收尾 ----
    registry.remove(&conversation_id);
    let aborted = abort.load(Ordering::Relaxed);

    let (status, error_msg, final_content) = match stream_result {
        Ok(full) if aborted => ("aborted".to_string(), None, full),
        Ok(full) => ("ok".to_string(), None, full),
        Err(e) => ("error".to_string(), Some(e.to_string()), String::new()),
    };

    {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        ai_chat::finalize_message(
            &conn,
            &assistant_message_id,
            &final_content,
            &status,
            error_msg.as_deref(),
            None,
            None,
        )?;
        let _ = ai_chat::touch_conversation(&conn, &conversation_id);
    }

    let final_chunk = match status.as_str() {
        "ok" => StreamChunk::done(),
        "aborted" => StreamChunk::aborted(),
        _ => StreamChunk::error(error_msg.clone().unwrap_or_default()),
    };
    let _ = on_chunk.send(final_chunk);

    Ok(SendMessageResult {
        user_message_id,
        assistant_message_id,
        status,
        error: error_msg,
    })
}

/// 中止指定会话当前正在进行的流式回复。
#[tauri::command]
pub async fn abort_chat_message(
    conversation_id: String,
    registry: State<'_, AbortRegistry>,
) -> AppResult<bool> {
    Ok(registry.signal(&conversation_id))
}

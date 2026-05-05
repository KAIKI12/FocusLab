//! AI 聊天 - 会话(conversation) + 消息(message) 数据层。
//!
//! 设计:
//! - `system_prompt` 存在 conversation 上(不作为独立 message),发送时拼接
//! - `model` 空字符串表示沿用当前 active profile 的默认 model
//! - `tool_calls` / `tool_results` 当前只占位,send_message 不消费

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::errors::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub origin_type: String,
    pub origin_id: Option<String>,
    pub provider: String,
    pub api_format: String,
    pub model: String,
    pub system_prompt: String,
    pub message_count: i64,
    pub pinned: bool,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub model: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub tool_calls: Option<String>,
    pub tool_results: Option<String>,
    pub tokens_in: Option<i64>,
    pub tokens_out: Option<i64>,
    pub created_at: String,
}

fn map_conversation(r: &rusqlite::Row<'_>) -> rusqlite::Result<Conversation> {
    Ok(Conversation {
        id: r.get(0)?,
        title: r.get(1)?,
        origin_type: r.get(2)?,
        origin_id: r.get(3)?,
        provider: r.get(4)?,
        api_format: r.get(5)?,
        model: r.get(6)?,
        system_prompt: r.get(7)?,
        message_count: r.get(8)?,
        pinned: r.get::<_, i64>(9)? != 0,
        archived: r.get::<_, i64>(10)? != 0,
        created_at: r.get(11)?,
        updated_at: r.get(12)?,
    })
}

const CONV_COLUMNS: &str = "id, title, origin_type, origin_id, provider, api_format, model, \
                            system_prompt, message_count, pinned, archived, created_at, updated_at";

pub fn list_conversations(conn: &Connection, include_archived: bool) -> AppResult<Vec<Conversation>> {
    let sql = if include_archived {
        format!(
            "SELECT {CONV_COLUMNS} FROM ai_conversations \
             ORDER BY pinned DESC, updated_at DESC"
        )
    } else {
        format!(
            "SELECT {CONV_COLUMNS} FROM ai_conversations \
             WHERE archived = 0 \
             ORDER BY pinned DESC, updated_at DESC"
        )
    };
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_conversation)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_conversation(conn: &Connection, id: &str) -> AppResult<Option<Conversation>> {
    let sql = format!("SELECT {CONV_COLUMNS} FROM ai_conversations WHERE id = ?1");
    let v = conn.query_row(&sql, params![id], map_conversation).optional()?;
    Ok(v)
}

#[allow(clippy::too_many_arguments)]
pub fn insert_conversation(
    conn: &Connection,
    title: &str,
    origin_type: &str,
    origin_id: Option<&str>,
    provider: &str,
    api_format: &str,
    model: &str,
    system_prompt: &str,
) -> AppResult<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO ai_conversations
         (id, title, origin_type, origin_id, provider, api_format, model,
          system_prompt, message_count, pinned, archived, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, 0, 0, ?9, ?9)",
        params![id, title, origin_type, origin_id, provider, api_format, model, system_prompt, now],
    )?;
    Ok(id)
}

pub fn rename_conversation(conn: &Connection, id: &str, title: &str) -> AppResult<usize> {
    let now = chrono::Utc::now().to_rfc3339();
    Ok(conn.execute(
        "UPDATE ai_conversations SET title = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, title, now],
    )?)
}

pub fn set_conversation_model(conn: &Connection, id: &str, model: &str) -> AppResult<usize> {
    let now = chrono::Utc::now().to_rfc3339();
    Ok(conn.execute(
        "UPDATE ai_conversations SET model = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, model, now],
    )?)
}

pub fn set_conversation_pinned(conn: &Connection, id: &str, pinned: bool) -> AppResult<usize> {
    let now = chrono::Utc::now().to_rfc3339();
    Ok(conn.execute(
        "UPDATE ai_conversations SET pinned = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, if pinned { 1 } else { 0 }, now],
    )?)
}

pub fn set_conversation_archived(conn: &Connection, id: &str, archived: bool) -> AppResult<usize> {
    let now = chrono::Utc::now().to_rfc3339();
    Ok(conn.execute(
        "UPDATE ai_conversations SET archived = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, if archived { 1 } else { 0 }, now],
    )?)
}

pub fn delete_conversation(conn: &Connection, id: &str) -> AppResult<usize> {
    Ok(conn.execute("DELETE FROM ai_conversations WHERE id = ?1", params![id])?)
}

pub fn delete_all_conversations(conn: &Connection) -> AppResult<usize> {
    conn.execute("DELETE FROM ai_chat_messages", [])?;
    Ok(conn.execute("DELETE FROM ai_conversations", [])?)
}

pub fn touch_conversation(conn: &Connection, id: &str) -> AppResult<usize> {
    let now = chrono::Utc::now().to_rfc3339();
    Ok(conn.execute(
        "UPDATE ai_conversations SET updated_at = ?2 WHERE id = ?1",
        params![id, now],
    )?)
}

fn map_message(r: &rusqlite::Row<'_>) -> rusqlite::Result<ChatMessage> {
    Ok(ChatMessage {
        id: r.get(0)?,
        conversation_id: r.get(1)?,
        role: r.get(2)?,
        content: r.get(3)?,
        model: r.get(4)?,
        status: r.get(5)?,
        error_message: r.get(6)?,
        tool_calls: r.get(7)?,
        tool_results: r.get(8)?,
        tokens_in: r.get(9)?,
        tokens_out: r.get(10)?,
        created_at: r.get(11)?,
    })
}

const MSG_COLUMNS: &str = "id, conversation_id, role, content, model, status, error_message, \
                           tool_calls, tool_results, tokens_in, tokens_out, created_at";

pub fn list_messages(conn: &Connection, conversation_id: &str) -> AppResult<Vec<ChatMessage>> {
    let sql = format!(
        "SELECT {MSG_COLUMNS} FROM ai_messages \
         WHERE conversation_id = ?1 \
         ORDER BY created_at ASC, id ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![conversation_id], map_message)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_message(conn: &Connection, id: &str) -> AppResult<Option<ChatMessage>> {
    let sql = format!("SELECT {MSG_COLUMNS} FROM ai_messages WHERE id = ?1");
    let v = conn.query_row(&sql, params![id], map_message).optional()?;
    Ok(v)
}

pub fn insert_message(
    conn: &Connection,
    conversation_id: &str,
    role: &str,
    content: &str,
    model: Option<&str>,
    status: &str,
) -> AppResult<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO ai_messages
         (id, conversation_id, role, content, model, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, conversation_id, role, content, model, status, now],
    )?;
    conn.execute(
        "UPDATE ai_conversations SET message_count = message_count + 1, updated_at = ?2
         WHERE id = ?1",
        params![conversation_id, now],
    )?;
    Ok(id)
}

#[allow(clippy::too_many_arguments)]
pub fn finalize_message(
    conn: &Connection,
    id: &str,
    content: &str,
    status: &str,
    error_message: Option<&str>,
    tokens_in: Option<i64>,
    tokens_out: Option<i64>,
) -> AppResult<usize> {
    Ok(conn.execute(
        "UPDATE ai_messages
         SET content = ?2, status = ?3, error_message = ?4,
             tokens_in = ?5, tokens_out = ?6
         WHERE id = ?1",
        params![id, content, status, error_message, tokens_in, tokens_out],
    )?)
}

pub fn delete_message(conn: &Connection, id: &str) -> AppResult<usize> {
    let conv: Option<String> = conn
        .query_row(
            "SELECT conversation_id FROM ai_messages WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .optional()?;
    let n = conn.execute("DELETE FROM ai_messages WHERE id = ?1", params![id])?;
    if n > 0 {
        if let Some(cid) = conv {
            conn.execute(
                "UPDATE ai_conversations
                 SET message_count = MAX(message_count - 1, 0)
                 WHERE id = ?1",
                params![cid],
            )?;
        }
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        conn.execute_batch(
            "CREATE TABLE ai_conversations (
               id TEXT PRIMARY KEY, title TEXT NOT NULL DEFAULT 'x',
               origin_type TEXT NOT NULL DEFAULT 'manual', origin_id TEXT,
               provider TEXT NOT NULL DEFAULT '', api_format TEXT NOT NULL DEFAULT '',
               model TEXT NOT NULL DEFAULT '', system_prompt TEXT NOT NULL DEFAULT '',
               message_count INTEGER NOT NULL DEFAULT 0,
               pinned INTEGER NOT NULL DEFAULT 0, archived INTEGER NOT NULL DEFAULT 0,
               created_at TEXT NOT NULL, updated_at TEXT NOT NULL
             );
             CREATE TABLE ai_messages (
               id TEXT PRIMARY KEY, conversation_id TEXT NOT NULL,
               role TEXT NOT NULL, content TEXT NOT NULL DEFAULT '',
               model TEXT, status TEXT NOT NULL DEFAULT 'ok', error_message TEXT,
               tool_calls TEXT, tool_results TEXT,
               tokens_in INTEGER, tokens_out INTEGER, created_at TEXT NOT NULL,
               FOREIGN KEY (conversation_id) REFERENCES ai_conversations(id) ON DELETE CASCADE
             );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn conversation_round_trip() {
        let conn = fresh_conn();
        let id = insert_conversation(
            &conn, "测试会话", "manual", None, "openai", "openai", "gpt-4o-mini", "你是助手",
        )
        .unwrap();
        let one = get_conversation(&conn, &id).unwrap().unwrap();
        assert_eq!(one.title, "测试会话");
        assert_eq!(one.message_count, 0);
        assert!(!one.pinned && !one.archived);

        rename_conversation(&conn, &id, "改名").unwrap();
        set_conversation_model(&conn, &id, "gpt-4o").unwrap();
        set_conversation_pinned(&conn, &id, true).unwrap();
        let two = get_conversation(&conn, &id).unwrap().unwrap();
        assert_eq!(two.title, "改名");
        assert_eq!(two.model, "gpt-4o");
        assert!(two.pinned);
    }

    #[test]
    fn list_excludes_archived_by_default() {
        let conn = fresh_conn();
        let a = insert_conversation(&conn, "A", "manual", None, "", "", "", "").unwrap();
        let b = insert_conversation(&conn, "B", "manual", None, "", "", "", "").unwrap();
        set_conversation_archived(&conn, &b, true).unwrap();

        let visible = list_conversations(&conn, false).unwrap();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].id, a);

        let all = list_conversations(&conn, true).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn pinned_sort_first() {
        let conn = fresh_conn();
        let a = insert_conversation(&conn, "A", "manual", None, "", "", "", "").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let b = insert_conversation(&conn, "B", "manual", None, "", "", "", "").unwrap();
        set_conversation_pinned(&conn, &a, true).unwrap();

        let listed = list_conversations(&conn, false).unwrap();
        assert_eq!(listed[0].id, a);
        assert_eq!(listed[1].id, b);
    }

    #[test]
    fn message_insert_increments_count_and_finalize_updates() {
        let conn = fresh_conn();
        let cid = insert_conversation(&conn, "T", "manual", None, "", "", "", "").unwrap();

        let m1 = insert_message(&conn, &cid, "user", "hi", None, "ok").unwrap();
        let m2 = insert_message(&conn, &cid, "assistant", "", Some("gpt-4o"), "streaming").unwrap();

        let conv = get_conversation(&conn, &cid).unwrap().unwrap();
        assert_eq!(conv.message_count, 2);

        finalize_message(&conn, &m2, "你好", "ok", None, Some(3), Some(8)).unwrap();
        let msg = get_message(&conn, &m2).unwrap().unwrap();
        assert_eq!(msg.content, "你好");
        assert_eq!(msg.status, "ok");
        assert_eq!(msg.tokens_out, Some(8));

        let messages = list_messages(&conn, &cid).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].id, m1);
        assert_eq!(messages[1].id, m2);
    }

    #[test]
    fn delete_message_decrements_count() {
        let conn = fresh_conn();
        let cid = insert_conversation(&conn, "T", "manual", None, "", "", "", "").unwrap();
        let m1 = insert_message(&conn, &cid, "user", "x", None, "ok").unwrap();
        insert_message(&conn, &cid, "assistant", "y", None, "ok").unwrap();
        delete_message(&conn, &m1).unwrap();
        let conv = get_conversation(&conn, &cid).unwrap().unwrap();
        assert_eq!(conv.message_count, 1);
    }

    #[test]
    fn delete_conversation_cascades_messages() {
        let conn = fresh_conn();
        let cid = insert_conversation(&conn, "T", "manual", None, "", "", "", "").unwrap();
        insert_message(&conn, &cid, "user", "x", None, "ok").unwrap();
        insert_message(&conn, &cid, "assistant", "y", None, "ok").unwrap();

        delete_conversation(&conn, &cid).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM ai_messages", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}

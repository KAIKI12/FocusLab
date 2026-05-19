//! AI Provider Profile - chat / embedding 双独立列表 + 一次性默认 profile 迁移。

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::errors::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatProfile {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub api_format: String,
    pub base_url: String,
    pub api_key: String,
    pub model_fast: String,
    pub model_strong: String,
    pub selected_models: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingProfile {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub created_at: String,
    pub updated_at: String,
}

const MIGRATED_FLAG_KEY: &str = "ai_default_profile_migrated";
const ACTIVE_CHAT_KEY: &str = "ai_active_chat_profile_id";
const ACTIVE_EMBEDDING_KEY: &str = "ai_active_embedding_profile_id";

// ---------- CRUD ----------

pub fn list_chat(conn: &Connection) -> AppResult<Vec<ChatProfile>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, provider, api_format, base_url, api_key, model_fast, model_strong, selected_models, created_at, updated_at
         FROM ai_chat_profiles ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(ChatProfile {
            id: r.get(0)?,
            name: r.get(1)?,
            provider: r.get(2)?,
            api_format: r.get(3)?,
            base_url: r.get(4)?,
            api_key: r.get(5)?,
            model_fast: r.get(6)?,
            model_strong: r.get(7)?,
            selected_models: r
                .get::<_, Option<String>>(8)?
                .unwrap_or_else(|| "[]".into()),
            created_at: r.get(9)?,
            updated_at: r.get(10)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn list_embedding(conn: &Connection) -> AppResult<Vec<EmbeddingProfile>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, base_url, api_key, model, created_at, updated_at
         FROM ai_embedding_profiles ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(EmbeddingProfile {
            id: r.get(0)?,
            name: r.get(1)?,
            base_url: r.get(2)?,
            api_key: r.get(3)?,
            model: r.get(4)?,
            created_at: r.get(5)?,
            updated_at: r.get(6)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_chat(conn: &Connection, id: &str) -> AppResult<Option<ChatProfile>> {
    let v = conn
        .query_row(
            "SELECT id, name, provider, api_format, base_url, api_key, model_fast, model_strong, selected_models, created_at, updated_at
             FROM ai_chat_profiles WHERE id = ?1",
            params![id],
            |r| {
                Ok(ChatProfile {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    provider: r.get(2)?,
                    api_format: r.get(3)?,
                    base_url: r.get(4)?,
                    api_key: r.get(5)?,
                    model_fast: r.get(6)?,
                    model_strong: r.get(7)?,
                    selected_models: r.get::<_, Option<String>>(8)?.unwrap_or_else(|| "[]".into()),
                    created_at: r.get(9)?,
                    updated_at: r.get(10)?,
                })
            },
        )
        .optional()?;
    Ok(v)
}

pub fn get_embedding(conn: &Connection, id: &str) -> AppResult<Option<EmbeddingProfile>> {
    let v = conn
        .query_row(
            "SELECT id, name, base_url, api_key, model, created_at, updated_at
             FROM ai_embedding_profiles WHERE id = ?1",
            params![id],
            |r| {
                Ok(EmbeddingProfile {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    base_url: r.get(2)?,
                    api_key: r.get(3)?,
                    model: r.get(4)?,
                    created_at: r.get(5)?,
                    updated_at: r.get(6)?,
                })
            },
        )
        .optional()?;
    Ok(v)
}

#[allow(clippy::too_many_arguments)]
pub fn insert_chat(
    conn: &Connection,
    name: &str,
    provider: &str,
    api_format: &str,
    base_url: &str,
    api_key: &str,
    model_fast: &str,
    model_strong: &str,
    selected_models: &str,
) -> AppResult<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO ai_chat_profiles
         (id, name, provider, api_format, base_url, api_key, model_fast, model_strong, selected_models, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
        params![id, name, provider, api_format, base_url, api_key, model_fast, model_strong, selected_models, now],
    )?;
    Ok(id)
}

pub fn insert_embedding(
    conn: &Connection,
    name: &str,
    base_url: &str,
    api_key: &str,
    model: &str,
) -> AppResult<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO ai_embedding_profiles
         (id, name, base_url, api_key, model, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![id, name, base_url, api_key, model, now],
    )?;
    Ok(id)
}

#[allow(clippy::too_many_arguments)]
pub fn update_chat(
    conn: &Connection,
    id: &str,
    name: &str,
    provider: &str,
    api_format: &str,
    base_url: &str,
    api_key: &str,
    model_fast: &str,
    model_strong: &str,
    selected_models: &str,
) -> AppResult<usize> {
    let now = chrono::Utc::now().to_rfc3339();
    Ok(conn.execute(
        "UPDATE ai_chat_profiles
         SET name = ?2, provider = ?3, api_format = ?4, base_url = ?5, api_key = ?6,
             model_fast = ?7, model_strong = ?8, selected_models = ?9, updated_at = ?10
         WHERE id = ?1",
        params![
            id,
            name,
            provider,
            api_format,
            base_url,
            api_key,
            model_fast,
            model_strong,
            selected_models,
            now
        ],
    )?)
}

pub fn update_embedding(
    conn: &Connection,
    id: &str,
    name: &str,
    base_url: &str,
    api_key: &str,
    model: &str,
) -> AppResult<usize> {
    let now = chrono::Utc::now().to_rfc3339();
    Ok(conn.execute(
        "UPDATE ai_embedding_profiles
         SET name = ?2, base_url = ?3, api_key = ?4, model = ?5, updated_at = ?6
         WHERE id = ?1",
        params![id, name, base_url, api_key, model, now],
    )?)
}

pub fn delete_chat(conn: &Connection, id: &str) -> AppResult<usize> {
    Ok(conn.execute("DELETE FROM ai_chat_profiles WHERE id = ?1", params![id])?)
}

pub fn delete_embedding(conn: &Connection, id: &str) -> AppResult<usize> {
    Ok(conn.execute(
        "DELETE FROM ai_embedding_profiles WHERE id = ?1",
        params![id],
    )?)
}

pub fn active_chat_id(conn: &Connection) -> AppResult<Option<String>> {
    read_setting(conn, ACTIVE_CHAT_KEY)
}

pub fn active_embedding_id(conn: &Connection) -> AppResult<Option<String>> {
    read_setting(conn, ACTIVE_EMBEDDING_KEY)
}

/// 切换激活的 chat profile,并把字段镜像回旧 settings keys (向后兼容)。
pub fn set_active_chat(conn: &Connection, id: &str) -> AppResult<()> {
    let profile = get_chat(conn, id)?
        .ok_or_else(|| crate::utils::errors::AppError::Custom("chat profile 不存在".into()))?;
    let now = chrono::Utc::now().to_rfc3339();
    write_setting(conn, ACTIVE_CHAT_KEY, &profile.id, &now)?;
    write_setting(conn, "ai_provider", &profile.provider, &now)?;
    write_setting(conn, "ai_api_format", &profile.api_format, &now)?;
    write_setting(conn, "ai_base_url", &profile.base_url, &now)?;
    write_setting(conn, "ai_api_key", &profile.api_key, &now)?;
    let primary = if !profile.model_fast.is_empty() {
        profile.model_fast.clone()
    } else {
        profile.model_strong.clone()
    };
    write_setting(conn, "ai_model", &primary, &now)?;
    write_setting(conn, "ai_model_fast", &profile.model_fast, &now)?;
    write_setting(conn, "ai_model_strong", &profile.model_strong, &now)?;
    Ok(())
}

pub fn set_active_embedding(conn: &Connection, id: &str) -> AppResult<()> {
    let profile = get_embedding(conn, id)?
        .ok_or_else(|| crate::utils::errors::AppError::Custom("embedding profile 不存在".into()))?;
    let now = chrono::Utc::now().to_rfc3339();
    write_setting(conn, ACTIVE_EMBEDDING_KEY, &profile.id, &now)?;
    write_setting(conn, "ai_embedding_base_url", &profile.base_url, &now)?;
    write_setting(conn, "ai_embedding_api_key", &profile.api_key, &now)?;
    write_setting(conn, "ai_embedding_model", &profile.model, &now)?;
    Ok(())
}

/// 删除 chat profile;若该 profile 是当前 active,自动切到剩余里最早的一条,
/// 没有剩余则清空 active id。
pub fn delete_chat_with_failover(conn: &Connection, id: &str) -> AppResult<()> {
    delete_chat(conn, id)?;
    let active = active_chat_id(conn)?;
    if active.as_deref() == Some(id) {
        let remaining = list_chat(conn)?;
        if let Some(first) = remaining.first() {
            set_active_chat(conn, &first.id)?;
        } else {
            let now = chrono::Utc::now().to_rfc3339();
            write_setting(conn, ACTIVE_CHAT_KEY, "", &now)?;
        }
    }
    Ok(())
}

pub fn delete_embedding_with_failover(conn: &Connection, id: &str) -> AppResult<()> {
    delete_embedding(conn, id)?;
    let active = active_embedding_id(conn)?;
    if active.as_deref() == Some(id) {
        let remaining = list_embedding(conn)?;
        if let Some(first) = remaining.first() {
            set_active_embedding(conn, &first.id)?;
        } else {
            let now = chrono::Utc::now().to_rfc3339();
            write_setting(conn, ACTIVE_EMBEDDING_KEY, "", &now)?;
        }
    }
    Ok(())
}

// ---------- 一次性默认 profile 迁移 ----------

/// 把旧的 `ai_*` settings keys 迁成"默认"profile,只跑一次。
///
/// 幂等保证: `ai_default_profile_migrated=1` flag。
/// 用户后续删除"默认"profile 不会被回灌。
pub fn ensure_default_profile(conn: &Connection) -> AppResult<()> {
    let already: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![MIGRATED_FLAG_KEY],
            |r| r.get(0),
        )
        .optional()?;
    if already.as_deref() == Some("1") {
        return Ok(());
    }

    let now = chrono::Utc::now().to_rfc3339();

    let chat_base_url = read_setting(conn, "ai_base_url")?.unwrap_or_default();
    let chat_api_key = read_setting(conn, "ai_api_key")?.unwrap_or_default();
    let chat_model = read_setting(conn, "ai_model")?.unwrap_or_default();
    let chat_provider = read_setting(conn, "ai_provider")?.unwrap_or_else(|| "openai".into());
    let chat_api_format = read_setting(conn, "ai_api_format")?.unwrap_or_else(|| "openai".into());
    let chat_fast = read_setting(conn, "ai_model_fast")?.unwrap_or_else(|| chat_model.clone());
    let chat_strong = read_setting(conn, "ai_model_strong")?.unwrap_or_else(|| chat_model.clone());

    if !chat_base_url.is_empty() || !chat_api_key.is_empty() {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO ai_chat_profiles
             (id, name, provider, api_format, base_url, api_key, model_fast, model_strong, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            params![
                id,
                "默认",
                chat_provider,
                chat_api_format,
                chat_base_url,
                chat_api_key,
                chat_fast,
                chat_strong,
                now,
            ],
        )?;
        write_setting(conn, ACTIVE_CHAT_KEY, &id, &now)?;
    }

    let emb_base_url = read_setting(conn, "ai_embedding_base_url")?.unwrap_or_default();
    let emb_api_key = read_setting(conn, "ai_embedding_api_key")?.unwrap_or_default();
    let emb_model = read_setting(conn, "ai_embedding_model")?.unwrap_or_default();
    if !emb_base_url.is_empty() || !emb_api_key.is_empty() {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO ai_embedding_profiles
             (id, name, base_url, api_key, model, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                id,
                "默认 Embedding",
                emb_base_url,
                emb_api_key,
                emb_model,
                now
            ],
        )?;
        write_setting(conn, ACTIVE_EMBEDDING_KEY, &id, &now)?;
    }

    write_setting(conn, MIGRATED_FLAG_KEY, "1", &now)?;
    Ok(())
}

fn read_setting(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    let v: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |r| r.get(0),
        )
        .optional()?;
    Ok(v)
}

fn write_setting(conn: &Connection, key: &str, value: &str, now: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value, now],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT, updated_at TEXT);
             CREATE TABLE ai_chat_profiles (
               id TEXT PRIMARY KEY, name TEXT NOT NULL, provider TEXT NOT NULL,
               api_format TEXT NOT NULL, base_url TEXT NOT NULL, api_key TEXT NOT NULL,
               model_fast TEXT NOT NULL DEFAULT '', model_strong TEXT NOT NULL DEFAULT '',
               selected_models TEXT NOT NULL DEFAULT '[]',
               created_at TEXT NOT NULL, updated_at TEXT NOT NULL
             );
             CREATE TABLE ai_embedding_profiles (
               id TEXT PRIMARY KEY, name TEXT NOT NULL,
               base_url TEXT NOT NULL, api_key TEXT NOT NULL, model TEXT NOT NULL DEFAULT '',
               created_at TEXT NOT NULL, updated_at TEXT NOT NULL
             );",
        )
        .unwrap();
        conn
    }

    fn count(conn: &Connection, table: &str) -> i64 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn empty_db_creates_no_profile_but_sets_flag() {
        let conn = fresh_conn();
        ensure_default_profile(&conn).unwrap();

        assert_eq!(count(&conn, "ai_chat_profiles"), 0);
        assert_eq!(count(&conn, "ai_embedding_profiles"), 0);
        assert_eq!(
            read_setting(&conn, MIGRATED_FLAG_KEY).unwrap(),
            Some("1".to_string())
        );
    }

    #[test]
    fn old_keys_get_seeded_into_default_profile() {
        let conn = fresh_conn();
        let now = chrono::Utc::now().to_rfc3339();
        write_setting(&conn, "ai_provider", "deepseek", &now).unwrap();
        write_setting(&conn, "ai_api_format", "openai", &now).unwrap();
        write_setting(&conn, "ai_base_url", "https://api.deepseek.com", &now).unwrap();
        write_setting(&conn, "ai_api_key", "sk-xxx", &now).unwrap();
        write_setting(&conn, "ai_model", "deepseek-chat", &now).unwrap();
        write_setting(
            &conn,
            "ai_embedding_base_url",
            "https://api.openai.com",
            &now,
        )
        .unwrap();
        write_setting(&conn, "ai_embedding_api_key", "sk-yyy", &now).unwrap();
        write_setting(&conn, "ai_embedding_model", "text-embedding-3-small", &now).unwrap();

        ensure_default_profile(&conn).unwrap();

        assert_eq!(count(&conn, "ai_chat_profiles"), 1);
        assert_eq!(count(&conn, "ai_embedding_profiles"), 1);

        let active_chat = read_setting(&conn, ACTIVE_CHAT_KEY).unwrap();
        let active_emb = read_setting(&conn, ACTIVE_EMBEDDING_KEY).unwrap();
        assert!(active_chat.is_some());
        assert!(active_emb.is_some());

        let chat_name: String = conn
            .query_row(
                "SELECT name FROM ai_chat_profiles WHERE id = ?1",
                params![active_chat.unwrap()],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(chat_name, "默认");
    }

    #[test]
    fn migration_is_idempotent_after_user_deletes_default() {
        let conn = fresh_conn();
        let now = chrono::Utc::now().to_rfc3339();
        write_setting(&conn, "ai_base_url", "https://api.openai.com", &now).unwrap();
        write_setting(&conn, "ai_api_key", "sk-x", &now).unwrap();

        ensure_default_profile(&conn).unwrap();
        assert_eq!(count(&conn, "ai_chat_profiles"), 1);

        // 模拟用户删除"默认"profile
        conn.execute("DELETE FROM ai_chat_profiles", []).unwrap();
        assert_eq!(count(&conn, "ai_chat_profiles"), 0);

        // 再次启动 - 不应被回灌
        ensure_default_profile(&conn).unwrap();
        assert_eq!(count(&conn, "ai_chat_profiles"), 0);
    }

    #[test]
    fn crud_round_trip_for_chat_profile() {
        let conn = fresh_conn();
        let id = insert_chat(
            &conn,
            "我的 DeepSeek",
            "deepseek",
            "openai",
            "https://api.deepseek.com",
            "sk-x",
            "deepseek-chat",
            "deepseek-reasoner",
            "[]",
        )
        .unwrap();

        let listed = list_chat(&conn).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "我的 DeepSeek");

        update_chat(
            &conn,
            &id,
            "我的 DeepSeek (改名)",
            "deepseek",
            "openai",
            "https://api.deepseek.com",
            "sk-y",
            "deepseek-chat",
            "deepseek-reasoner",
            "[]",
        )
        .unwrap();

        let one = get_chat(&conn, &id).unwrap().unwrap();
        assert_eq!(one.name, "我的 DeepSeek (改名)");
        assert_eq!(one.api_key, "sk-y");

        delete_chat(&conn, &id).unwrap();
        assert_eq!(list_chat(&conn).unwrap().len(), 0);
    }

    #[test]
    fn set_active_chat_mirrors_legacy_keys() {
        let conn = fresh_conn();
        let id = insert_chat(
            &conn,
            "qwen",
            "qwen",
            "openai",
            "https://dashscope.aliyuncs.com",
            "sk-q",
            "qwen-turbo",
            "qwen-max",
            "[]",
        )
        .unwrap();
        set_active_chat(&conn, &id).unwrap();

        assert_eq!(
            read_setting(&conn, "ai_provider").unwrap(),
            Some("qwen".into())
        );
        assert_eq!(
            read_setting(&conn, "ai_base_url").unwrap(),
            Some("https://dashscope.aliyuncs.com".into())
        );
        assert_eq!(
            read_setting(&conn, "ai_model").unwrap(),
            Some("qwen-turbo".into())
        );
        assert_eq!(
            read_setting(&conn, "ai_model_fast").unwrap(),
            Some("qwen-turbo".into())
        );
        assert_eq!(
            read_setting(&conn, "ai_model_strong").unwrap(),
            Some("qwen-max".into())
        );
        assert_eq!(active_chat_id(&conn).unwrap(), Some(id));
    }

    #[test]
    fn delete_active_chat_falls_over_to_first_remaining() {
        let conn = fresh_conn();
        let a = insert_chat(&conn, "A", "openai", "openai", "u", "k", "m", "m", "[]").unwrap();
        let b = insert_chat(&conn, "B", "openai", "openai", "u2", "k2", "m2", "m2", "[]").unwrap();
        set_active_chat(&conn, &a).unwrap();

        delete_chat_with_failover(&conn, &a).unwrap();

        assert_eq!(active_chat_id(&conn).unwrap(), Some(b));
    }

    #[test]
    fn delete_last_chat_clears_active() {
        let conn = fresh_conn();
        let a = insert_chat(&conn, "Solo", "openai", "openai", "u", "k", "m", "m", "[]").unwrap();
        set_active_chat(&conn, &a).unwrap();

        delete_chat_with_failover(&conn, &a).unwrap();

        assert_eq!(active_chat_id(&conn).unwrap(), Some("".into()));
    }
}

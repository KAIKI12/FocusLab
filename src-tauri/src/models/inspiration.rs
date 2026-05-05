//! Inspiration 模型 + links helpers。

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::errors::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspirationRecord {
    pub id: String,
    pub content: String,
    pub goal_id: Option<String>,
    pub summary: Option<String>,
    /// 关键词列表(对外接口),DB 内存为 JSON 字符串,model 层负责反序列化。
    pub keywords: Vec<String>,
    pub verification: String,
    pub embedding_status: String,
    pub converted_task_id: Option<String>,
    pub converted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 把 DB 中存储的 JSON 字符串反序列化成 Vec<String>。失败时返回空数组(异常数据宽容)。
fn parse_keywords(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        return Vec::new();
    }
    serde_json::from_str::<Vec<String>>(raw).unwrap_or_default()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspirationLink {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation: String,
    pub source_type: String,
    pub reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspirationEmbedding {
    pub inspiration_id: String,
    pub model: String,
    pub dim: i64,
    pub vector: Vec<f32>,
    pub created_at: String,
}

fn encode_embedding(vector: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vector.len() * 4);
    for value in vector {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn decode_embedding(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

pub fn get_embedding(conn: &Connection, inspiration_id: &str) -> AppResult<Option<InspirationEmbedding>> {
    let row = conn.query_row(
        "SELECT inspiration_id, model, dim, vector, created_at FROM inspiration_embeddings WHERE inspiration_id = ?1",
        params![inspiration_id],
        |r| {
            let raw: Vec<u8> = r.get("vector")?;
            Ok(InspirationEmbedding {
                inspiration_id: r.get("inspiration_id")?,
                model: r.get("model")?,
                dim: r.get("dim")?,
                vector: decode_embedding(&raw),
                created_at: r.get("created_at")?,
            })
        },
    );
    match row {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

pub fn upsert_embedding(conn: &Connection, inspiration_id: &str, model: &str, vector: &[f32]) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    let raw = encode_embedding(vector);
    conn.execute(
        "INSERT INTO inspiration_embeddings (inspiration_id, model, dim, vector, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(inspiration_id) DO UPDATE SET
           model = excluded.model,
           dim = excluded.dim,
           vector = excluded.vector,
           created_at = excluded.created_at",
        params![inspiration_id, model, vector.len() as i64, raw, now],
    )?;
    conn.execute(
        "UPDATE inspirations SET embedding_status = 'done', updated_at = ?2 WHERE id = ?1",
        params![inspiration_id, now],
    )?;
    Ok(())
}

pub fn mark_embedding_failed(conn: &Connection, inspiration_id: &str) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE inspirations SET embedding_status = 'failed', updated_at = ?2 WHERE id = ?1",
        params![inspiration_id, now],
    )?;
    Ok(())
}

pub fn list_inspirations(conn: &Connection) -> AppResult<Vec<InspirationRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, goal_id, summary, keywords, verification, embedding_status,
                converted_task_id, converted_at, created_at, updated_at
         FROM inspirations
         ORDER BY created_at DESC",
    )?;
    let rows = stmt
        .query_map([], |r| {
            let keywords_raw: String = r.get("keywords")?;
            Ok(InspirationRecord {
                id: r.get("id")?,
                content: r.get("content")?,
                goal_id: r.get("goal_id")?,
                summary: r.get("summary")?,
                keywords: parse_keywords(&keywords_raw),
                verification: r.get("verification")?,
                embedding_status: r.get("embedding_status")?,
                converted_task_id: r.get("converted_task_id")?,
                converted_at: r.get("converted_at")?,
                created_at: r.get("created_at")?,
                updated_at: r.get("updated_at")?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn create_inspiration(conn: &Connection, content: &str, goal_id: Option<&str>) -> AppResult<InspirationRecord> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO inspirations (
            id, content, goal_id, summary, keywords, verification, embedding_status,
            converted_task_id, converted_at, created_at, updated_at
         ) VALUES (?1, ?2, ?3, NULL, '[]', 'none', 'pending', NULL, NULL, ?4, ?4)",
        params![id, content, goal_id, now],
    )?;
    Ok(InspirationRecord {
        id,
        content: content.to_string(),
        goal_id: goal_id.map(str::to_string),
        summary: None,
        keywords: Vec::new(),
        verification: "none".into(),
        embedding_status: "pending".into(),
        converted_task_id: None,
        converted_at: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn update_inspiration_goal(conn: &Connection, id: &str, goal_id: Option<&str>) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE inspirations SET goal_id = ?1, updated_at = ?2 WHERE id = ?3",
        params![goal_id, now, id],
    )?;
    Ok(())
}

/// B2: 一次性清空某个 goal 下所有灵感的 goal_id。
/// 用于 goal 归档/删除时同步,避免灵感继续指向已归档 goal。
/// 返回影响行数(被清空关联的灵感数)。
pub fn clear_inspirations_for_goal(conn: &Connection, goal_id: &str) -> AppResult<usize> {
    let now = Utc::now().to_rfc3339();
    let affected = conn.execute(
        "UPDATE inspirations SET goal_id = NULL, updated_at = ?1 WHERE goal_id = ?2",
        params![now, goal_id],
    )?;
    Ok(affected)
}

pub fn update_inspiration_verification(conn: &Connection, id: &str, verification: &str) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE inspirations SET verification = ?1, updated_at = ?2 WHERE id = ?3",
        params![verification, now, id],
    )?;
    Ok(())
}

pub fn update_keywords_summary(conn: &Connection, id: &str, keywords: &str, summary: Option<&str>) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE inspirations SET keywords = ?1, summary = ?2, updated_at = ?3 WHERE id = ?4",
        params![keywords, summary, now, id],
    )?;
    Ok(())
}

pub fn delete_inspiration(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM inspirations WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn mark_converted(conn: &Connection, id: &str, task_id: &str) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE inspirations SET converted_task_id = ?1, converted_at = ?2, updated_at = ?2 WHERE id = ?3",
        params![task_id, now, id],
    )?;
    Ok(())
}

pub fn list_links_for_inspiration(conn: &Connection, inspiration_id: &str) -> AppResult<Vec<InspirationLink>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_id, target_id, relation, source_type, reason, created_at
         FROM inspiration_links
         WHERE source_id = ?1 OR target_id = ?1
         ORDER BY created_at DESC",
    )?;
    let rows = stmt
        .query_map(params![inspiration_id], |r| {
            Ok(InspirationLink {
                id: r.get("id")?,
                source_id: r.get("source_id")?,
                target_id: r.get("target_id")?,
                relation: r.get("relation")?,
                source_type: r.get("source_type")?,
                reason: r.get("reason")?,
                created_at: r.get("created_at")?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn create_link(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
    relation: &str,
    source_type: &str,
    reason: Option<&str>,
) -> AppResult<InspirationLink> {
    let (left, right) = if source_id <= target_id {
        (source_id, target_id)
    } else {
        (target_id, source_id)
    };
    // D4: UNIQUE 冲突时返回带"已存在"语义的明确错误,前端可精准提示。
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM inspiration_links WHERE source_id = ?1 AND target_id = ?2",
            params![left, right],
            |row| row.get(0),
        )
        .ok();
    if existing.is_some() {
        return Err(crate::utils::errors::AppError::Custom(
            "DUPLICATE_LINK:这两条灵感已经关联过了".into(),
        ));
    }
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO inspiration_links (id, source_id, target_id, relation, source_type, reason, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, left, right, relation, source_type, reason, now],
    )?;
    Ok(InspirationLink {
        id,
        source_id: left.to_string(),
        target_id: right.to_string(),
        relation: relation.to_string(),
        source_type: source_type.to_string(),
        reason: reason.map(str::to_string),
        created_at: now,
    })
}

pub fn delete_link(conn: &Connection, source_id: &str, target_id: &str) -> AppResult<()> {
    let (left, right) = if source_id <= target_id {
        (source_id, target_id)
    } else {
        (target_id, source_id)
    };
    conn.execute(
        "DELETE FROM inspiration_links WHERE source_id = ?1 AND target_id = ?2",
        params![left, right],
    )?;
    Ok(())
}

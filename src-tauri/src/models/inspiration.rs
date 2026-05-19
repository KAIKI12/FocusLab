//! Inspiration 模型 + links helpers。

use std::collections::HashSet;

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::errors::{AppError, AppResult};

const SELF_LINK_ERROR: &str = "SELF_LINK:不能关联同一条灵感";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspirationRecord {
    pub id: String,
    pub content: String,
    pub goal_id: Option<String>,
    pub image_path: Option<String>,
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

pub fn get_embedding(
    conn: &Connection,
    inspiration_id: &str,
) -> AppResult<Option<InspirationEmbedding>> {
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

pub fn upsert_embedding(
    conn: &Connection,
    inspiration_id: &str,
    model: &str,
    vector: &[f32],
) -> AppResult<()> {
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
        "SELECT id, content, goal_id, image_path, summary, keywords, verification, embedding_status,
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
                image_path: r.get("image_path")?,
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

pub fn create_inspiration(
    conn: &Connection,
    content: &str,
    goal_id: Option<&str>,
    image_path: Option<&str>,
) -> AppResult<InspirationRecord> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let embedding_status = if content.trim().is_empty() {
        "done"
    } else {
        "pending"
    };
    conn.execute(
        "INSERT INTO inspirations (
            id, content, goal_id, image_path, summary, keywords, verification, embedding_status,
            converted_task_id, converted_at, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, NULL, '[]', 'none', ?5, NULL, NULL, ?6, ?6)",
        params![id, content, goal_id, image_path, embedding_status, now],
    )?;
    Ok(InspirationRecord {
        id,
        content: content.to_string(),
        goal_id: goal_id.map(str::to_string),
        image_path: image_path.map(str::to_string),
        summary: None,
        keywords: Vec::new(),
        verification: "none".into(),
        embedding_status: embedding_status.into(),
        converted_task_id: None,
        converted_at: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn get_inspiration(conn: &Connection, id: &str) -> AppResult<Option<InspirationRecord>> {
    let mut items = conn.prepare(
        "SELECT id, content, goal_id, image_path, summary, keywords, verification, embedding_status,
                converted_task_id, converted_at, created_at, updated_at
         FROM inspirations
         WHERE id = ?1
         LIMIT 1",
    )?;
    let mut rows = items.query(params![id])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
    let keywords_raw: String = row.get("keywords")?;
    Ok(Some(InspirationRecord {
        id: row.get("id")?,
        content: row.get("content")?,
        goal_id: row.get("goal_id")?,
        image_path: row.get("image_path")?,
        summary: row.get("summary")?,
        keywords: parse_keywords(&keywords_raw),
        verification: row.get("verification")?,
        embedding_status: row.get("embedding_status")?,
        converted_task_id: row.get("converted_task_id")?,
        converted_at: row.get("converted_at")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    }))
}

pub fn update_inspiration_goal(
    conn: &Connection,
    id: &str,
    goal_id: Option<&str>,
) -> AppResult<()> {
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

pub fn update_inspiration_verification(
    conn: &Connection,
    id: &str,
    verification: &str,
) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE inspirations SET verification = ?1, updated_at = ?2 WHERE id = ?3",
        params![verification, now, id],
    )?;
    Ok(())
}

pub fn update_keywords_summary(
    conn: &Connection,
    id: &str,
    keywords: &str,
    summary: Option<&str>,
) -> AppResult<()> {
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

pub fn list_links_for_inspiration(
    conn: &Connection,
    inspiration_id: &str,
) -> AppResult<Vec<InspirationLink>> {
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
    if source_id == target_id {
        return Err(AppError::Custom(SELF_LINK_ERROR.into()));
    }

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

pub fn list_linked_peer_ids(conn: &Connection, inspiration_id: &str) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT CASE
            WHEN source_id = ?1 THEN target_id
            ELSE source_id
         END AS peer_id
         FROM inspiration_links
         WHERE source_id = ?1 OR target_id = ?1
         ORDER BY created_at DESC",
    )?;
    let rows = stmt
        .query_map(params![inspiration_id], |row| row.get("peer_id"))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(rows)
}

pub fn ignore_recommendation(
    conn: &Connection,
    source_id: &str,
    candidate_id: &str,
    relation: &str,
) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR IGNORE INTO inspiration_ignored_recommendations (
            source_id, candidate_id, relation, ignored_at
         ) VALUES (?1, ?2, ?3, ?4)",
        params![source_id, candidate_id, relation, now],
    )?;
    Ok(())
}

pub fn list_ignored_recommendation_keys(
    conn: &Connection,
    source_id: &str,
) -> AppResult<HashSet<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT candidate_id, relation
         FROM inspiration_ignored_recommendations
         WHERE source_id = ?1",
    )?;
    let rows = stmt
        .query_map(params![source_id], |row| {
            Ok((
                row.get::<_, String>("candidate_id")?,
                row.get::<_, String>("relation")?,
            ))
        })?
        .collect::<rusqlite::Result<HashSet<_>>>()?;
    Ok(rows)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn mem_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE goals (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
        )
        .unwrap();
        conn.execute_batch(include_str!("../db/migrations/004_inspiration_graph.sql"))
            .unwrap();
        conn.execute_batch(include_str!(
            "../db/migrations/008_inspiration_ignored_recommendations.sql"
        ))
        .unwrap();
        conn.execute_batch(include_str!("../db/migrations/009_inspiration_images.sql"))
            .unwrap();
        conn
    }

    #[test]
    fn create_link_rejects_self_link() {
        let conn = mem_db();
        let item = create_inspiration(&conn, "idea", None, None).unwrap();

        let result = create_link(&conn, &item.id, &item.id, "related", "manual", None);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("SELF_LINK"));
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM inspiration_links", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn list_linked_peer_ids_returns_both_directions() {
        let conn = mem_db();
        let first = create_inspiration(&conn, "first", None, None).unwrap();
        let second = create_inspiration(&conn, "second", None, None).unwrap();
        create_link(&conn, &second.id, &first.id, "related", "manual", None).unwrap();

        let peers = list_linked_peer_ids(&conn, &first.id).unwrap();

        assert_eq!(peers, vec![second.id]);
    }

    #[test]
    fn ignore_recommendation_is_idempotent_per_relation() {
        let conn = mem_db();
        let source = create_inspiration(&conn, "source", None, None).unwrap();
        let candidate = create_inspiration(&conn, "candidate", None, None).unwrap();

        ignore_recommendation(&conn, &source.id, &candidate.id, "related").unwrap();
        ignore_recommendation(&conn, &source.id, &candidate.id, "related").unwrap();
        ignore_recommendation(&conn, &source.id, &candidate.id, "contradicts").unwrap();

        let ignored = list_ignored_recommendation_keys(&conn, &source.id).unwrap();

        assert_eq!(ignored.len(), 2);
        assert!(ignored.contains(&(candidate.id.clone(), "related".into())));
        assert!(ignored.contains(&(candidate.id, "contradicts".into())));
    }

    #[test]
    fn create_and_get_inspiration_preserves_image_path() {
        let conn = mem_db();

        let created = create_inspiration(&conn, "", None, Some("C:\\images\\idea.png")).unwrap();
        let loaded = get_inspiration(&conn, &created.id).unwrap().unwrap();

        assert_eq!(loaded.image_path.as_deref(), Some("C:\\images\\idea.png"));
        assert_eq!(loaded.content, "");
    }
}

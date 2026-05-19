//! Inspiration graph CRUD / link / recommendation 命令。

use std::collections::HashSet;
use std::path::PathBuf;

use serde::Deserialize;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::ai::AIService;
use crate::db::Db;
use crate::models::inspiration::{self, InspirationLink, InspirationRecord};
use crate::services::inspiration_service::{self, InspirationRecommendation};
use crate::utils::errors::{AppError, AppResult};

const MAX_IMAGE_BYTES: usize = 8 * 1024 * 1024;

fn is_valid_relation(relation: &str) -> bool {
    relation == "related" || relation == "contradicts"
}

fn filter_ignored_recommendations(
    recommendations: Vec<InspirationRecommendation>,
    ignored_keys: &HashSet<(String, String)>,
) -> Vec<InspirationRecommendation> {
    recommendations
        .into_iter()
        .filter(|item| !ignored_keys.contains(&(item.candidate_id.clone(), item.relation.clone())))
        .collect()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInspirationInput {
    pub content: String,
    pub goal_id: Option<String>,
    pub image_bytes: Option<Vec<u8>>,
    pub image_mime_type: Option<String>,
}

fn image_extension_for_mime(mime: &str) -> AppResult<&'static str> {
    match mime.trim() {
        "image/png" => Ok("png"),
        "image/jpeg" => Ok("jpg"),
        "image/webp" => Ok("webp"),
        "image/gif" => Ok("gif"),
        other => Err(AppError::Custom(format!("暂不支持的图片格式: {other}"))),
    }
}

fn save_inspiration_image(
    app: &AppHandle,
    image_bytes: &[u8],
    image_mime_type: &str,
) -> AppResult<String> {
    if image_bytes.is_empty() {
        return Err(AppError::Custom("图片内容不能为空".into()));
    }
    if image_bytes.len() > MAX_IMAGE_BYTES {
        return Err(AppError::Custom("图片不能超过 8 MB".into()));
    }
    let ext = image_extension_for_mime(image_mime_type)?;
    let image_dir = inspiration_image_dir(app)?;
    std::fs::create_dir_all(&image_dir)?;

    let filename = format!("{}.{}", Uuid::new_v4(), ext);
    let full_path = image_dir.join(filename);
    std::fs::write(&full_path, image_bytes)?;
    Ok(full_path.to_string_lossy().into_owned())
}

fn inspiration_image_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Custom(format!("cannot resolve app_data_dir: {e}")))?;
    Ok(app_data.join("inspiration-images"))
}

fn delete_saved_image(app: &AppHandle, path: &PathBuf) -> AppResult<()> {
    if !path.exists() {
        return Ok(());
    }
    let image_dir = inspiration_image_dir(app)?;
    std::fs::create_dir_all(&image_dir)?;
    let canonical_dir = image_dir.canonicalize()?;
    let canonical_path = path.canonicalize()?;
    if !canonical_path.starts_with(&canonical_dir) {
        return Err(AppError::Custom("拒绝删除应用图片目录外的文件".into()));
    }
    std::fs::remove_file(canonical_path)?;
    Ok(())
}

#[tauri::command]
pub fn list_inspirations(db: State<'_, Db>) -> AppResult<Vec<InspirationRecord>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    inspiration::list_inspirations(&conn)
}

#[tauri::command]
pub fn create_inspiration(
    input: CreateInspirationInput,
    app: AppHandle,
    db: State<'_, Db>,
) -> AppResult<InspirationRecord> {
    let content = input.content.trim();
    let image_bytes = input.image_bytes.unwrap_or_default();
    let image_mime_type = input.image_mime_type.unwrap_or_default();
    let has_image = !image_bytes.is_empty();
    if content.is_empty() && !has_image {
        return Err(AppError::Custom("灵感内容和图片不能同时为空".into()));
    }
    if !has_image && !image_mime_type.trim().is_empty() {
        return Err(AppError::Custom("缺少图片内容".into()));
    }
    if has_image && image_mime_type.trim().is_empty() {
        return Err(AppError::Custom("缺少图片 MIME 类型".into()));
    }
    let saved_image_path = if has_image {
        Some(PathBuf::from(save_inspiration_image(&app, &image_bytes, &image_mime_type)?))
    } else {
        None
    };
    let saved_image_path_text = saved_image_path
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned());
    let record = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        match inspiration::create_inspiration(
            &conn,
            content,
            input.goal_id.as_deref(),
            saved_image_path_text.as_deref(),
        ) {
            Ok(record) => record,
            Err(error) => {
                if let Some(path) = saved_image_path.as_ref() {
                    if let Err(cleanup_error) = delete_saved_image(&app, path) {
                        tracing::error!(
                            "failed to cleanup saved inspiration image after db error: {}",
                            cleanup_error
                        );
                    }
                }
                return Err(error);
            }
        }
    };

    // 注意:不再创建时自动跑 LLM 推荐,改为用户主动点击「分析关联」按钮触发
    // (走 suggest_related_inspirations 命令)。
    // embedding 由 batch_embed_pending 后台保证 — 不影响语义检索基础设施。

    Ok(record)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInspirationGoalInput {
    pub id: String,
    pub goal_id: Option<String>,
}

#[tauri::command]
pub fn update_inspiration_goal(
    input: UpdateInspirationGoalInput,
    db: State<'_, Db>,
) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    inspiration::update_inspiration_goal(&conn, &input.id, input.goal_id.as_deref())
}

/// B2: goal 归档/删除时一次性清空所有挂载该 goal 的灵感关联。
#[tauri::command]
pub fn clear_inspirations_for_goal(goal_id: String, db: State<'_, Db>) -> AppResult<usize> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    inspiration::clear_inspirations_for_goal(&conn, &goal_id)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInspirationVerificationInput {
    pub id: String,
    pub verification: String,
}

#[tauri::command]
pub fn update_inspiration_verification(
    input: UpdateInspirationVerificationInput,
    db: State<'_, Db>,
) -> AppResult<()> {
    let verification = input.verification.trim();
    // B6: 5 态 — v0.3 原始设计的 verification 模型。
    // none               未标注
    // needs_check        待复查 (用户主动 / AI 纠偏触发)
    // possibly_wrong     可能错误 (AI 纠偏分析"建议"级结论)
    // verified           已验证
    // overturned         已被推翻 (有实验/证据反驳)
    const ALLOWED: &[&str] = &[
        "none",
        "needs_check",
        "possibly_wrong",
        "verified",
        "overturned",
        "resolved",
    ];
    if !ALLOWED.contains(&verification) {
        return Err(AppError::Custom(format!(
            "非法 verification 状态: {verification}"
        )));
    }
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    inspiration::update_inspiration_verification(&conn, &input.id, verification)
}

#[tauri::command]
pub fn delete_inspiration(id: String, app: AppHandle, db: State<'_, Db>) -> AppResult<()> {
    let image_path = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        inspiration::get_inspiration(&conn, &id)?.and_then(|item| item.image_path)
    };
    {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        inspiration::delete_inspiration(&conn, &id)?;
    }
    if let Some(path) = image_path {
        delete_saved_image(&app, &PathBuf::from(path))?;
    }
    Ok(())
}

#[tauri::command]
pub fn mark_inspiration_converted(id: String, task_id: String, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    inspiration::mark_converted(&conn, &id, &task_id)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkInspirationsInput {
    pub source_id: String,
    pub target_id: String,
    pub relation: Option<String>,
    pub source_type: Option<String>,
    pub reason: Option<String>,
}

#[tauri::command]
pub fn link_inspirations(
    input: LinkInspirationsInput,
    db: State<'_, Db>,
) -> AppResult<InspirationLink> {
    let relation = input.relation.as_deref().unwrap_or("related");
    let source_type = input.source_type.as_deref().unwrap_or("manual");
    if !is_valid_relation(relation) {
        return Err(AppError::Custom("非法 relation 类型".into()));
    }
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    inspiration::create_link(
        &conn,
        &input.source_id,
        &input.target_id,
        relation,
        source_type,
        input.reason.as_deref(),
    )
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlinkInspirationsInput {
    pub source_id: String,
    pub target_id: String,
}

#[tauri::command]
pub fn unlink_inspirations(input: UnlinkInspirationsInput, db: State<'_, Db>) -> AppResult<()> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    inspiration::delete_link(&conn, &input.source_id, &input.target_id)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IgnoreInspirationRecommendationInput {
    pub source_id: String,
    pub candidate_id: String,
    pub relation: String,
}

#[tauri::command]
pub fn ignore_inspiration_recommendation(
    input: IgnoreInspirationRecommendationInput,
    db: State<'_, Db>,
) -> AppResult<()> {
    let relation = input.relation.trim();
    if !is_valid_relation(relation) {
        return Err(AppError::Custom("非法 relation 类型".into()));
    }
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    inspiration::ignore_recommendation(&conn, &input.source_id, &input.candidate_id, relation)
}

#[tauri::command]
pub fn list_inspiration_links(
    inspiration_id: String,
    db: State<'_, Db>,
) -> AppResult<Vec<InspirationLink>> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    inspiration::list_links_for_inspiration(&conn, &inspiration_id)
}

#[tauri::command]
pub async fn suggest_related_inspirations(
    inspiration_id: String,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<Vec<InspirationRecommendation>> {
    let (items, current, embedding_model, rerank_model, existing_embeddings, ignored_keys) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        // D3: 用户主动关闭 embedding 时,显式返回空数组而非报错。
        // 这是显式行为(用户知情),不算 silent fallback。
        let embedding_enabled = crate::models::settings::get(&conn, "ai_embedding_enabled")?
            .unwrap_or_else(|| "1".into());
        if embedding_enabled == "0" {
            return Ok(Vec::new());
        }
        let items = inspiration::list_inspirations(&conn)?;
        let current = items.iter().find(|item| item.id == inspiration_id).cloned();
        let Some(current) = current else {
            return Err(AppError::Custom("灵感不存在".into()));
        };
        if current.content.trim().is_empty() {
            return Ok(Vec::new());
        }
        let linked_peer_ids = inspiration::list_linked_peer_ids(&conn, &inspiration_id)?
            .into_iter()
            .collect::<HashSet<_>>();
        let items = items
            .into_iter()
            .filter(|item| item.id == current.id || !linked_peer_ids.contains(&item.id))
            .collect::<Vec<_>>();
        let ignored_keys = inspiration::list_ignored_recommendation_keys(&conn, &inspiration_id)?;
        let embedding_model = crate::models::settings::get(&conn, "ai_embedding_model")?
            .unwrap_or_else(|| "text-embedding-3-small".into());
        let rerank_model = crate::models::settings::get(&conn, "ai_model_fast")?
            .or(crate::models::settings::get(&conn, "ai_model")?);
        let mut existing_embeddings = Vec::new();
        for item in &items {
            if let Some(found) = inspiration::get_embedding(&conn, &item.id)? {
                existing_embeddings.push(found);
            }
        }
        (
            items,
            current,
            embedding_model,
            rerank_model,
            existing_embeddings,
            ignored_keys,
        )
    };

    let (recommendations, updates) = inspiration_service::suggest_related(
        &ai,
        &current,
        &items,
        embedding_model.clone(),
        rerank_model,
        &existing_embeddings,
    )
    .await?;
    let recommendations = filter_ignored_recommendations(recommendations, &ignored_keys);

    if !updates.is_empty() {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        for (inspiration_id, vector) in updates {
            inspiration::upsert_embedding(&conn, &inspiration_id, &embedding_model, &vector)?;
        }
    }

    Ok(recommendations)
}

// ---------- localStorage → SQLite 一次性迁移 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrateInspirationItem {
    pub id: String,
    pub content: String,
    pub goal_id: Option<String>,
    pub image_path: Option<String>,
    pub converted_task_id: Option<String>,
    pub converted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub fn migrate_inspirations_from_local(
    items: Vec<MigrateInspirationItem>,
    db: State<'_, Db>,
) -> AppResult<u32> {
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let mut count = 0u32;
    for item in &items {
        let content = item.content.trim();
        if content.is_empty() && item.image_path.is_none() {
            continue;
        }
        let embedding_status = if content.is_empty() { "done" } else { "pending" };
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM inspirations WHERE id = ?1",
                rusqlite::params![item.id],
                |r| r.get(0),
            )
            .unwrap_or(false);
        if exists {
            continue;
        }
        conn.execute(
            "INSERT INTO inspirations (
                id, content, goal_id, image_path, summary, keywords, verification, embedding_status,
                converted_task_id, converted_at, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, NULL, '[]', 'none', ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                item.id,
                content,
                item.goal_id,
                item.image_path,
                embedding_status,
                item.converted_task_id,
                item.converted_at,
                item.created_at,
                item.updated_at,
            ],
        )?;
        count += 1;
    }
    Ok(count)
}

// ---------- 批量 embedding 索引 ----------

#[tauri::command]
pub async fn batch_embed_pending(ai: State<'_, AIService>, db: State<'_, Db>) -> AppResult<u32> {
    let (pending_items, embedding_model) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let embedding_enabled = crate::models::settings::get(&conn, "ai_embedding_enabled")?
            .unwrap_or_else(|| "1".into());
        if embedding_enabled == "0" {
            return Ok(0);
        }
        let mut stmt = conn.prepare(
            "SELECT id, content
             FROM inspirations
             WHERE embedding_status IN ('pending', 'failed')
               AND trim(content) <> ''
             LIMIT 10",
        ).map_err(|e| AppError::Custom(e.to_string()))?;
        let rows: Vec<(String, String)> = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| AppError::Custom(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        let model = crate::models::settings::get(&conn, "ai_embedding_model")?
            .unwrap_or_else(|| "text-embedding-3-small".into());
        (rows, model)
    };

    if pending_items.is_empty() {
        return Ok(0);
    }

    let texts: Vec<String> = pending_items.iter().map(|(_, c)| c.clone()).collect();
    let ids: Vec<String> = pending_items.iter().map(|(id, _)| id.clone()).collect();

    match ai.embed(texts, Some(embedding_model.clone())).await {
        Ok(vectors) => {
            let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
            let mut count = 0u32;
            for (i, vector) in vectors.iter().enumerate() {
                if i < ids.len() {
                    inspiration::upsert_embedding(&conn, &ids[i], &embedding_model, vector)?;
                    count += 1;
                }
            }
            Ok(count)
        }
        Err(e) => {
            let conn = db.0.lock().map_err(|e2| AppError::Custom(e2.to_string()))?;
            for id in &ids {
                let _ = inspiration::mark_embedding_failed(&conn, id);
            }
            Err(AppError::Custom(format!("Embedding 批量索引失败: {e}")))
        }
    }
}

/// 单条重试 embedding。无视当前 status 强制再算一次,成功 → done,失败 → failed。
#[tauri::command]
pub async fn retry_embed_inspiration(
    id: String,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<()> {
    let (content, embedding_model) = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let items = inspiration::list_inspirations(&conn)?;
        let target = items.into_iter().find(|i| i.id == id);
        let Some(target) = target else {
            return Err(AppError::Custom("灵感不存在".into()));
        };
        if target.content.trim().is_empty() {
            return Err(AppError::Custom("纯图片灵感不支持语义索引".into()));
        }
        let model = crate::models::settings::get(&conn, "ai_embedding_model")?
            .unwrap_or_else(|| "text-embedding-3-small".into());
        (target.content, model)
    };
    match ai.embed(vec![content], Some(embedding_model.clone())).await {
        Ok(vectors) => {
            let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
            if let Some(v) = vectors.into_iter().next() {
                inspiration::upsert_embedding(&conn, &id, &embedding_model, &v)?;
            }
            Ok(())
        }
        Err(e) => {
            let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
            let _ = inspiration::mark_embedding_failed(&conn, &id);
            Err(AppError::Custom(format!("embedding 重试失败: {e}")))
        }
    }
}

// ---------- AI 提取 keyword + summary ----------

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeywordResult {
    pub keywords: Vec<String>,
    pub summary: Option<String>,
}

#[tauri::command]
pub async fn extract_inspiration_keywords(
    id: String,
    ai: State<'_, AIService>,
    db: State<'_, Db>,
) -> AppResult<KeywordResult> {
    let content = {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let items = inspiration::list_inspirations(&conn)?;
        items.into_iter().find(|i| i.id == id).map(|i| i.content)
    };
    let Some(content) = content else {
        return Err(AppError::Custom("灵感不存在".into()));
    };

    let prompt = format!(
        "从以下灵感文本中提取 2-4 个关键词和一句话摘要。\n\
        严格按 JSON 返回，格式：{{\"keywords\":[\"关键词1\",\"关键词2\"],\"summary\":\"一句话摘要\"}}\n\n\
        灵感原文：{content}"
    );
    let messages = vec![crate::ai::Message {
        role: "user".into(),
        content: prompt,
    }];
    let result = ai
        .complete(
            messages,
            crate::ai::CompletionOptions {
                temperature: Some(0.3),
                max_tokens: Some(150),
                model_override: None,
            },
        )
        .await?;

    let cleaned = result
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(cleaned) {
        let kw_arr: Vec<String> = parsed
            .get("keywords")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let keywords_json = serde_json::to_string(&kw_arr).unwrap_or_else(|_| "[]".into());
        let summary = parsed
            .get("summary")
            .and_then(|v| v.as_str())
            .map(String::from);
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        inspiration::update_keywords_summary(&conn, &id, &keywords_json, summary.as_deref())?;
        return Ok(KeywordResult {
            keywords: kw_arr,
            summary,
        });
    }
    Ok(KeywordResult {
        keywords: vec![],
        summary: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recommendation(candidate_id: &str, relation: &str) -> InspirationRecommendation {
        InspirationRecommendation {
            candidate_id: candidate_id.to_string(),
            candidate_content: "candidate".into(),
            relation: relation.to_string(),
            reason: "reason".into(),
            confidence: 0.9,
        }
    }

    #[test]
    fn filter_ignored_recommendations_removes_matching_relation_only() {
        let ignored = HashSet::from([("old-1".to_string(), "related".to_string())]);
        let raw = vec![
            recommendation("old-1", "related"),
            recommendation("old-1", "contradicts"),
            recommendation("old-2", "related"),
        ];

        let filtered = filter_ignored_recommendations(raw, &ignored);

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].candidate_id, "old-1");
        assert_eq!(filtered[0].relation, "contradicts");
        assert_eq!(filtered[1].candidate_id, "old-2");
    }

    #[test]
    fn image_extension_for_mime_supports_png() {
        assert_eq!(image_extension_for_mime("image/png").unwrap(), "png");
    }

    #[test]
    fn image_extension_for_mime_rejects_unsupported_type() {
        assert!(image_extension_for_mime("image/tiff").is_err());
    }
}

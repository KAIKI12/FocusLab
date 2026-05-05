//! 灵感推荐服务：embedding + 余弦召回 + LLM 精排。

use serde::{Deserialize, Serialize};

use crate::ai::{AIService, CompletionOptions, Message};
use crate::models::inspiration::{InspirationEmbedding, InspirationRecord};
use crate::utils::errors::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspirationRecommendation {
    pub candidate_id: String,
    pub candidate_content: String,
    pub relation: String,
    pub reason: String,
    pub confidence: f64,
}

pub fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.is_empty() || right.is_empty() || left.len() != right.len() {
        return 0.0;
    }
    let dot = left.iter().zip(right.iter()).map(|(a, b)| a * b).sum::<f32>();
    let left_norm = left.iter().map(|v| v * v).sum::<f32>().sqrt();
    let right_norm = right.iter().map(|v| v * v).sum::<f32>().sqrt();
    if left_norm == 0.0 || right_norm == 0.0 {
        return 0.0;
    }
    dot / (left_norm * right_norm)
}

pub async fn suggest_related(
    ai: &AIService,
    current: &InspirationRecord,
    items: &[InspirationRecord],
    embedding_model: String,
    rerank_model: Option<String>,
    existing_embeddings: &[InspirationEmbedding],
) -> AppResult<(Vec<InspirationRecommendation>, Vec<(String, Vec<f32>)>)> {
    let mut updates: Vec<(String, Vec<f32>)> = Vec::new();

    let current_vec = match existing_embeddings
        .iter()
        .find(|item| item.inspiration_id == current.id && item.model == embedding_model)
    {
        Some(found) => found.vector.clone(),
        None => {
            let generated = ai.embed(vec![current.content.clone()], Some(embedding_model.clone())).await?;
            let vector = generated
                .into_iter()
                .next()
                .ok_or_else(|| AppError::Custom("embedding 返回空结果".into()))?;
            updates.push((current.id.clone(), vector.clone()));
            vector
        }
    };

    // B4: 性能护栏 — 当 items 数量较大时,先按 (同 goal 优先) + (时间近优先) 预筛选 Top 50,
    // 再做 cosine 排序,避免大数据下全量调 embed/cosine。
    // 50 个仍能给 LLM 提供足够 recall,而把计算量限制在常数级。
    const PREFILTER_LIMIT: usize = 50;
    let candidates: Vec<&InspirationRecord> = if items.len() > PREFILTER_LIMIT {
        let mut filtered: Vec<&InspirationRecord> = items
            .iter()
            .filter(|it| it.id != current.id)
            .collect();
        filtered.sort_by(|a, b| {
            // 同 goal 排前面;同状态下按 created_at 倒序
            let a_same_goal = current.goal_id.is_some() && current.goal_id == a.goal_id;
            let b_same_goal = current.goal_id.is_some() && current.goal_id == b.goal_id;
            match (a_same_goal, b_same_goal) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b.created_at.cmp(&a.created_at),
            }
        });
        filtered.truncate(PREFILTER_LIMIT);
        filtered
    } else {
        items.iter().filter(|it| it.id != current.id).collect()
    };

    let mut ranked: Vec<(f32, &InspirationRecord)> = Vec::new();
    for item in candidates {
        let other_vec = match existing_embeddings
            .iter()
            .find(|embedding| embedding.inspiration_id == item.id && embedding.model == embedding_model)
        {
            Some(found) => found.vector.clone(),
            None => {
                let generated = ai.embed(vec![item.content.clone()], Some(embedding_model.clone())).await?;
                let vector = generated
                    .into_iter()
                    .next()
                    .ok_or_else(|| AppError::Custom("embedding 返回空结果".into()))?;
                updates.push((item.id.clone(), vector.clone()));
                vector
            }
        };

        let mut score = cosine_similarity(&current_vec, &other_vec);
        if current.goal_id.is_some() && current.goal_id == item.goal_id {
            score += 0.05;
        }
        ranked.push((score, item));
    }
    ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(10);

    let candidates_text = ranked
        .iter()
        .enumerate()
        .map(|(idx, (score, item))| format!("候选{}\nID: {}\n内容: {}\n相似度: {:.3}", idx + 1, item.id, item.content, score))
        .collect::<Vec<_>>()
        .join("\n\n");

    let prompt = format!(
        "你是科研灵感关联助手。\n当前新灵感:\nID: {}\n内容: {}\n\n以下是候选旧灵感:\n{}\n\n请从中挑出最多 3 条真正有价值的关系。只允许 relation 为 related 或 contradicts。\n输出严格 JSON: {{\"recommendations\":[{{\"candidateId\":\"\",\"candidateContent\":\"\",\"relation\":\"related|contradicts\",\"reason\":\"\",\"confidence\":0.0}}]}}",
        current.id,
        current.content,
        candidates_text
    );

    let raw = ai
        .complete(
            vec![Message {
                role: "user".into(),
                content: prompt,
            }],
            CompletionOptions {
                temperature: Some(0.2),
                max_tokens: Some(600),
                model_override: rerank_model,
            },
        )
        .await?;

    #[derive(Debug, Deserialize)]
    struct RecommendationEnvelope {
        recommendations: Vec<InspirationRecommendation>,
    }

    let parsed: RecommendationEnvelope = serde_json::from_str(raw.trim())
        .map_err(|e| AppError::Custom(format!("推荐结果解析失败: {e}")))?;

    let recommendations = parsed
        .recommendations
        .into_iter()
        .filter(|item| item.confidence >= 0.7)
        .take(3)
        .collect();

    Ok((recommendations, updates))
}

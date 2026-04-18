//! Task 模型 — 最小字段版本,覆盖 Week 1a CRUD 需求。
//!
//! 对齐 docs/04 §7.2 `tasks` 表结构;可选/默认字段暂不暴露到前端。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub quadrant: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

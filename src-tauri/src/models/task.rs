//! Task 模型 — 覆盖 Week 1a CRUD + Week 2b 编辑/删除需求。
//!
//! 对齐 docs/04 §7.2 `tasks` 表结构。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub quadrant: String,
    pub status: String,
    pub estimated_minutes: Option<i64>,
    pub due_date: Option<String>,
    pub is_background: bool,
    pub shelved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

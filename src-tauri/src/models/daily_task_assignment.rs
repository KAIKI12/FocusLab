//! DailyTaskAssignment · 任务与"哪一天的计划"的显式关联。
//!
//! 对齐 docs/04 §7.2 `daily_task_assignments` DDL。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyTaskAssignment {
    pub id: String,
    /// 逻辑日 "YYYY-MM-DD"
    pub plan_date: String,
    pub task_id: String,
    pub is_planned: bool,
    /// manual | carried_over | ai_suggested | guided | recurring | auto_due_pinned
    pub source: String,
    /// pending | completed | carried_forward | shelved | cancelled
    pub day_status: String,
    pub added_at: String,
    pub completed_at: Option<String>,
    pub sort_order: i64,
}

/// 返回给前端的"任务 × 当日分配"联表结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignmentWithTask {
    pub id: String,
    pub plan_date: String,
    pub task_id: String,
    pub task_name: String,
    pub task_quadrant: String,
    /// 任务全局生命周期:pending | in_progress | completed
    pub task_status: String,
    pub is_planned: bool,
    pub source: String,
    pub day_status: String,
    pub added_at: String,
    pub completed_at: Option<String>,
    pub sort_order: i64,
}

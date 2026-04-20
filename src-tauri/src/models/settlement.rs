//! Settlement 模型。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settlement {
    pub id: String,
    pub settle_date: String,
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub extra_tasks: i64,
    pub shelved_tasks: i64,
    pub completion_rate: f64,
    pub total_focus_minutes: i64,
    pub total_pomodoros: i64,
    pub total_interruptions: i64,
    pub grade: String,
    pub longest_focus_task_id: Option<String>,
    pub longest_focus_minutes: Option<i64>,
    pub ai_summary: Option<String>,
    pub user_reflection: Option<String>,
    pub trigger_type: String,
    pub created_at: String,
    /// 晚间情绪 1-5 (1=疲惫 .. 5=很好),NULL = 未打卡或开关关闭
    pub evening_mood: Option<i64>,
    /// 早晨意图档位 1-5 (1=保养 .. 5=冲刺),NULL = 未打卡或开关关闭
    pub morning_intent: Option<i64>,
}

/// 昨日摘要(前端 YesterdayCard 用)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YesterdaySummary {
    pub settle_date: String,
    pub completed_tasks: i64,
    pub total_tasks: i64,
    pub completion_rate: f64,
    pub grade: String,
    pub total_focus_minutes: i64,
    pub total_pomodoros: i64,
    pub longest_focus_task_name: Option<String>,
    pub carried_over_count: i64,
}

/// 日历视图用的轻量日摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaySummary {
    pub settle_date: String,
    pub completed_tasks: i64,
    pub total_tasks: i64,
    pub grade: String,
    pub total_focus_minutes: i64,
    pub total_pomodoros: i64,
}

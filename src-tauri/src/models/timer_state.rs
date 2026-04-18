//! TimerState · 单行崩溃恢复表的模型层。
//!
//! 对齐 docs/04 §7.2 和 docs/04 §11 (崩溃恢复)。
//! 表里 id='current' 行永远存在(由 001_init.sql 预置 idle 状态)。

use serde::{Deserialize, Serialize};

/// 完整的计时器快照 — 读出时全字段填充。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub task_id: Option<String>,
    pub session_id: Option<String>,
    /// RFC3339 UTC 时间
    pub start_time: Option<String>,
    pub elapsed_seconds: i64,
    pub planned_seconds: Option<i64>,
    /// pomodoro | free
    pub mode: Option<String>,
    /// classic_25 | deep_45 | immersive_90
    pub pomodoro_preset: Option<String>,
    /// running | paused | break | idle
    pub status: String,
    pub pomodoro_count: i64,
    pub is_break: bool,
    pub break_remaining: Option<i64>,
    pub updated_at: String,
}

/// 部分更新补丁 — 字段都是 Option;None 表示"不动"(只设置非空字段)。
///
/// 设计取舍:本 MVP 只提供"设值"语义,不支持"显式清空为 NULL"。
/// 要回到全空 idle 状态,调 `reset_timer_state` 一次性复位所有字段。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct TimerStatePatch {
    pub task_id: Option<String>,
    pub session_id: Option<String>,
    pub start_time: Option<String>,
    pub elapsed_seconds: Option<i64>,
    pub planned_seconds: Option<i64>,
    pub mode: Option<String>,
    pub pomodoro_preset: Option<String>,
    pub status: Option<String>,
    pub pomodoro_count: Option<i64>,
    pub is_break: Option<bool>,
    pub break_remaining: Option<i64>,
    /// 仅调试用:显式覆盖 updated_at(模拟崩溃场景)。生产代码不要传。
    pub updated_at: Option<String>,
}

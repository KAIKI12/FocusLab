//! 崩溃恢复判定命令。
//!
//! 对齐 docs/04 §11.2 三档策略:
//!   gap < 120s  → AutoResume(前端直接继续计时 + toast)
//!   120-3600s   → AskUser  (前端弹 RecoveryDialog 三选一)
//!   >= 3600s    → AutoEnd  (前端自动结束会话 + toast)
//!
//! 本命令只做**判定**,不做任何写操作 — 保持 "debug-first,失败全暴露"
//! (CLAUDE.md §2.2)。具体动作由前端在收到 recommendation 后再发 IPC。

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use tauri::State;

use crate::commands::timer_commands::get_timer_state;
use crate::db::Db;
use crate::models::timer_state::TimerState;
use crate::utils::errors::{AppError, AppResult};

/// 对前端提示的动作建议 — 纯语义枚举,不带副作用。
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum RecoveryAction {
    AutoResume,
    AskUser,
    AutoEnd,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryInfo {
    pub state: TimerState,
    /// 关联任务的名字,方便 Dialog 里直接展示
    pub task_name: Option<String>,
    pub gap_seconds: i64,
    pub recommendation: RecoveryAction,
}

/// 内部分类逻辑 — 独立出来方便单测,不依赖数据库。
pub(crate) fn classify(gap_seconds: i64) -> RecoveryAction {
    if gap_seconds < 120 {
        RecoveryAction::AutoResume
    } else if gap_seconds < 3600 {
        RecoveryAction::AskUser
    } else {
        RecoveryAction::AutoEnd
    }
}

/// 启动时由前端 App.vue onMounted 调用。
/// - status='idle' → 返回 None(正常启动,无需弹任何提示)
/// - 其他 → 返回 RecoveryInfo,recommendation 告诉前端走哪一档
#[tauri::command]
pub fn check_crash_recovery(db: State<'_, Db>) -> AppResult<Option<RecoveryInfo>> {
    let state = get_timer_state(db.clone())?;
    if state.status == "idle" {
        return Ok(None);
    }

    let updated: DateTime<Utc> = DateTime::parse_from_rfc3339(&state.updated_at)
        .map_err(|e| AppError::Custom(format!("bad updated_at: {e}")))?
        .with_timezone(&Utc);
    let gap = (Utc::now() - updated).num_seconds().max(0);

    // 取 task_name(若有 task_id)
    let task_name: Option<String> = if let Some(task_id) = &state.task_id {
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        conn.query_row(
            "SELECT name FROM tasks WHERE id = ?1",
            params![task_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    } else {
        None
    };

    Ok(Some(RecoveryInfo {
        state,
        task_name,
        gap_seconds: gap,
        recommendation: classify(gap),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gap_zero_is_auto_resume() {
        assert_eq!(classify(0), RecoveryAction::AutoResume);
    }

    #[test]
    fn gap_under_2min_is_auto_resume() {
        assert_eq!(classify(119), RecoveryAction::AutoResume);
    }

    #[test]
    fn gap_exactly_2min_is_ask_user() {
        // 边界:120 秒不再 AutoResume,进入 AskUser
        assert_eq!(classify(120), RecoveryAction::AskUser);
    }

    #[test]
    fn gap_middle_is_ask_user() {
        assert_eq!(classify(1800), RecoveryAction::AskUser);
    }

    #[test]
    fn gap_just_under_1h_is_ask_user() {
        assert_eq!(classify(3599), RecoveryAction::AskUser);
    }

    #[test]
    fn gap_exactly_1h_is_auto_end() {
        assert_eq!(classify(3600), RecoveryAction::AutoEnd);
    }

    #[test]
    fn gap_many_hours_is_auto_end() {
        assert_eq!(classify(86_400), RecoveryAction::AutoEnd);
    }
}

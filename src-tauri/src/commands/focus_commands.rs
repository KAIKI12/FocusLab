//! 番茄钟控制命令 — 对接 TimerService。
//!
//! 命令均为 async,因为 TimerService 的内部状态锁是 tokio 异步锁。

use tauri::State;

use crate::services::timer_service::{TimerService, TimerSnapshot};
use crate::utils::errors::AppResult;

#[tauri::command]
pub async fn start_pomodoro(
    task_id: String,
    preset: String,
    timer: State<'_, TimerService>,
) -> AppResult<TimerSnapshot> {
    timer.start_pomodoro(task_id, preset).await
}

#[tauri::command]
pub async fn pause_timer(timer: State<'_, TimerService>) -> AppResult<TimerSnapshot> {
    timer.pause().await
}

#[tauri::command]
pub async fn resume_timer(timer: State<'_, TimerService>) -> AppResult<TimerSnapshot> {
    timer.resume().await
}

#[tauri::command]
pub async fn abandon_timer(
    reason: Option<String>,
    timer: State<'_, TimerService>,
) -> AppResult<()> {
    timer.abandon(reason).await
}

#[tauri::command]
pub async fn skip_break(timer: State<'_, TimerService>) -> AppResult<()> {
    timer.skip_break().await
}

#[tauri::command]
pub async fn resume_from_crash(timer: State<'_, TimerService>) -> AppResult<TimerSnapshot> {
    timer.resume_from_crash().await
}

#[tauri::command]
pub async fn abandon_from_crash(timer: State<'_, TimerService>) -> AppResult<()> {
    timer.abandon_from_crash().await
}

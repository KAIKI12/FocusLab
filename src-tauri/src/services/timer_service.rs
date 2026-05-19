//! TimerService · 番茄钟后端权威状态机。
//!
//! 架构(对齐 docs/04 §3.4):
//! - 内存权威状态 `Arc<tokio::sync::Mutex<Option<RunningTimer>>>`
//! - tokio 任务每秒 tick,emit `timer:tick` 事件
//! - 每 30s 把快照落盘 `timer_state` 行(崩溃恢复兜底)
//! - 状态迁移(start/pause/resume/abandon/focus→break/break→idle)emit `timer:state_changed`
//! - Week 3 悬浮球窗口可直接订阅事件同步,本服务零改动

use std::sync::Arc;

use chrono::{DateTime, Utc};
use rusqlite::params;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

use crate::db::Db;
use crate::models::session;
use crate::models::timer_state::TimerState;
use crate::utils::errors::{AppError, AppResult};

// ---------- 常量 ----------

/// 每秒一次 tick
const TICK_INTERVAL_SECS: u64 = 1;
/// 每 30s 把内存快照写回 timer_state 表
const PERSIST_INTERVAL_SECS: i64 = 30;
/// 长休息时长(每 4 个番茄触发)— docs/02 §2.1 "15-30 分钟",2a 取 15 分钟
const LONG_BREAK_SECONDS: i64 = 15 * 60;
const MIN_CUSTOM_SECONDS: i64 = 60;
const MAX_CUSTOM_SECONDS: i64 = 180 * 60;

// ---------- 纯函数(单测) ----------

pub fn compute_planned_seconds(preset: &str) -> i64 {
    match preset {
        "classic_25" => 25 * 60,
        "deep_45" => 45 * 60,
        "immersive_90" => 90 * 60,
        _ => 25 * 60,
    }
}

fn resolve_planned_seconds(preset: &str, custom_planned_seconds: Option<i64>) -> AppResult<i64> {
    if preset != "custom" {
        return Ok(compute_planned_seconds(preset));
    }

    let Some(seconds) = custom_planned_seconds else {
        return Err(AppError::Custom("自定义番茄钟需要提供时长".into()));
    };
    if !(MIN_CUSTOM_SECONDS..=MAX_CUSTOM_SECONDS).contains(&seconds) {
        return Err(AppError::Custom(
            "自定义番茄钟时长需在 1-180 分钟之间".into(),
        ));
    }
    Ok(seconds)
}

pub fn compute_break_seconds(preset: &str, pomodoro_count: i64) -> i64 {
    if pomodoro_count > 0 && pomodoro_count % 4 == 0 {
        return LONG_BREAK_SECONDS;
    }
    match preset {
        "classic_25" => 5 * 60,
        "deep_45" => 10 * 60,
        "immersive_90" => 15 * 60,
        "custom" => 5 * 60,
        _ => 5 * 60,
    }
}

// ---------- 数据结构 ----------

struct RunningTimer {
    task_id: String,
    session_id: String,
    mode: String,
    preset: Option<String>,
    status: String, // running | paused | break
    start_time: DateTime<Utc>,
    planned_seconds: i64,
    elapsed_seconds: i64,
    pomodoro_count: i64,
    is_break: bool,
    break_planned_seconds: i64,
    last_persisted_at: DateTime<Utc>,
}

/// 从 timer_state 行读回的原始字段 — 仅在 resume_from_crash 里用。
struct CrashedTimerRow {
    task_id: Option<String>,
    session_id: Option<String>,
    start_time: Option<String>,
    elapsed: i64,
    planned: Option<i64>,
    mode: Option<String>,
    preset: Option<String>,
    status: String,
    pomodoro_count: i64,
    is_break: i64,
    break_remaining: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerSnapshot {
    pub status: String,
    pub task_id: Option<String>,
    pub session_id: Option<String>,
    pub mode: Option<String>,
    pub preset: Option<String>,
    pub elapsed_seconds: i64,
    pub planned_seconds: i64,
    pub pomodoro_count: i64,
    pub is_break: bool,
}

fn snapshot_of(t: &RunningTimer) -> TimerSnapshot {
    TimerSnapshot {
        status: t.status.clone(),
        task_id: Some(t.task_id.clone()),
        session_id: Some(t.session_id.clone()),
        mode: Some(t.mode.clone()),
        preset: t.preset.clone(),
        elapsed_seconds: t.elapsed_seconds,
        planned_seconds: if t.is_break {
            t.break_planned_seconds
        } else {
            t.planned_seconds
        },
        pomodoro_count: t.pomodoro_count,
        is_break: t.is_break,
    }
}

fn idle_snapshot() -> TimerSnapshot {
    TimerSnapshot {
        status: "idle".into(),
        task_id: None,
        session_id: None,
        mode: None,
        preset: None,
        elapsed_seconds: 0,
        planned_seconds: 0,
        pomodoro_count: 0,
        is_break: false,
    }
}

// ---------- 服务 ----------

pub struct TimerService {
    state: Arc<Mutex<Option<RunningTimer>>>,
    tick_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    app: AppHandle,
}

impl TimerService {
    pub fn new(app: AppHandle) -> Self {
        Self {
            state: Arc::new(Mutex::new(None)),
            tick_handle: Arc::new(Mutex::new(None)),
            app,
        }
    }

    /// 启动一个番茄钟 — 创建 session,写入内存 RunningTimer,立即落盘 timer_state,spawn tick
    pub async fn start_pomodoro(
        &self,
        task_id: String,
        preset: String,
        custom_planned_seconds: Option<i64>,
    ) -> AppResult<TimerSnapshot> {
        {
            let guard = self.state.lock().await;
            if guard.is_some() {
                return Err(AppError::Custom("已有计时进行中,请先结束或放弃".into()));
            }
        }

        let planned = resolve_planned_seconds(&preset, custom_planned_seconds)?;
        let now = Utc::now();

        // 创建 session
        let session_id = {
            let db = self.app.state::<Db>();
            let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
            let sid = session::create_session(
                &conn,
                &task_id,
                "pomodoro",
                Some(&preset),
                Some(planned / 60),
            )?;

            // 自动锁定今日计划(首次启动番茄钟时)
            let boundary = crate::models::settings::get_boundary_hour(&conn)?;
            let logical_date = crate::utils::datetime::current_logical_date(boundary).to_string();
            let lock_id = uuid::Uuid::new_v4().to_string();
            let _ = conn.execute(
                "INSERT INTO daily_plans (id, plan_date, plan_locked_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?3, ?3)
                 ON CONFLICT(plan_date) DO UPDATE SET
                   plan_locked_at = COALESCE(daily_plans.plan_locked_at, excluded.plan_locked_at),
                   updated_at = excluded.updated_at",
                params![lock_id, logical_date, now.to_rfc3339()],
            );

            sid
        };

        // 写入内存
        let timer = RunningTimer {
            task_id: task_id.clone(),
            session_id: session_id.clone(),
            mode: "pomodoro".into(),
            preset: Some(preset.clone()),
            status: "running".into(),
            start_time: now,
            planned_seconds: planned,
            elapsed_seconds: 0,
            pomodoro_count: 0,
            is_break: false,
            break_planned_seconds: 0,
            last_persisted_at: now,
        };

        // 立即落盘一次(避免 30s 内崩溃丢失)
        persist_running_to_db(&self.app, &timer)?;

        let snap = snapshot_of(&timer);
        *self.state.lock().await = Some(timer);

        // spawn tick
        self.spawn_tick().await;

        // emit state_changed
        let _ = self.app.emit("timer:state_changed", &snap);
        tracing::info!("pomodoro started: task={} preset={}", task_id, preset);
        Ok(snap)
    }

    pub async fn pause(&self) -> AppResult<TimerSnapshot> {
        let mut guard = self.state.lock().await;
        let t = guard
            .as_mut()
            .ok_or_else(|| AppError::Custom("当前无计时".into()))?;
        if t.status != "running" {
            return Err(AppError::Custom(format!("当前状态 {} 无法暂停", t.status)));
        }
        t.status = "paused".into();
        let snap = snapshot_of(t);
        persist_running_to_db(&self.app, t)?;
        let _ = self.app.emit("timer:state_changed", &snap);
        Ok(snap)
    }

    pub async fn resume(&self) -> AppResult<TimerSnapshot> {
        let mut guard = self.state.lock().await;
        let t = guard
            .as_mut()
            .ok_or_else(|| AppError::Custom("当前无计时".into()))?;
        if t.status != "paused" {
            return Err(AppError::Custom(format!("当前状态 {} 无法继续", t.status)));
        }
        t.status = "running".into();
        let snap = snapshot_of(t);
        persist_running_to_db(&self.app, t)?;
        let _ = self.app.emit("timer:state_changed", &snap);
        Ok(snap)
    }

    /// 主动放弃 — 写 sessions 为 abandoned,清空内存 + timer_state,停 tick
    pub async fn abandon(&self, reason: Option<String>) -> AppResult<()> {
        let taken = {
            let mut guard = self.state.lock().await;
            guard.take()
        };
        self.abort_tick().await;

        let Some(t) = taken else {
            // 已是 idle 也算 ok
            reset_db_timer(&self.app)?;
            let _ = self.app.emit("timer:state_changed", &idle_snapshot());
            return Ok(());
        };

        let actual_min = (t.elapsed_seconds / 60).max(0);
        {
            let db = self.app.state::<Db>();
            let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
            session::abandon_session(&conn, &t.session_id, actual_min, reason.as_deref())?;
        }
        reset_db_timer(&self.app)?;

        let _ = self.app.emit("timer:state_changed", &idle_snapshot());
        tracing::info!("abandoned session {}", t.session_id);
        Ok(())
    }

    /// 手动结束休息 — 直接回 idle
    pub async fn skip_break(&self) -> AppResult<()> {
        let cleared = {
            let mut guard = self.state.lock().await;
            if let Some(t) = guard.as_ref() {
                if !t.is_break {
                    return Err(AppError::Custom("当前不在休息中".into()));
                }
            } else {
                return Err(AppError::Custom("当前无计时".into()));
            }
            *guard = None;
            true
        };
        if cleared {
            self.abort_tick().await;
            reset_db_timer(&self.app)?;
            let _ = self.app.emit("timer:state_changed", &idle_snapshot());
        }
        Ok(())
    }

    // ---------- Week 2b: 休息结束三选一 ----------

    /// 休息结束后继续同一个任务 — 创建新 session,重置为 running
    pub async fn continue_same_task(&self) -> AppResult<TimerSnapshot> {
        let (task_id, preset, planned, count) = {
            let guard = self.state.lock().await;
            let t = guard
                .as_ref()
                .ok_or_else(|| AppError::Custom("无计时状态".into()))?;
            if t.status != "break_ended" {
                return Err(AppError::Custom("当前不在休息结束状态".into()));
            }
            (
                t.task_id.clone(),
                t.preset.clone(),
                t.planned_seconds,
                t.pomodoro_count,
            )
        };

        // 清空旧状态
        self.abort_tick().await;
        *self.state.lock().await = None;

        // 复用 start_pomodoro 的逻辑,但保留 pomodoro_count
        let preset_str = preset.unwrap_or_else(|| "classic_25".into());
        let now = Utc::now();

        let session_id = {
            let db = self.app.state::<Db>();
            let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
            session::create_session(
                &conn,
                &task_id,
                "pomodoro",
                Some(&preset_str),
                Some(planned / 60),
            )?
        };

        let timer = RunningTimer {
            task_id: task_id.clone(),
            session_id: session_id.clone(),
            mode: "pomodoro".into(),
            preset: Some(preset_str),
            status: "running".into(),
            start_time: now,
            planned_seconds: planned,
            elapsed_seconds: 0,
            pomodoro_count: count,
            is_break: false,
            break_planned_seconds: 0,
            last_persisted_at: now,
        };

        persist_running_to_db(&self.app, &timer)?;
        let snap = snapshot_of(&timer);
        *self.state.lock().await = Some(timer);
        self.spawn_tick().await;
        let _ = self.app.emit("timer:state_changed", &snap);
        Ok(snap)
    }

    /// 休息结束后切换到另一个任务
    pub async fn switch_task(&self, new_task_id: String) -> AppResult<TimerSnapshot> {
        let (count, preset_str, planned) = {
            let guard = self.state.lock().await;
            let t = guard
                .as_ref()
                .ok_or_else(|| AppError::Custom("无计时状态".into()))?;
            if t.status != "break_ended" {
                return Err(AppError::Custom("当前不在休息结束状态".into()));
            }
            (
                t.pomodoro_count,
                t.preset.clone().unwrap_or_else(|| "classic_25".into()),
                t.planned_seconds,
            )
        };

        // 清空旧状态,用新 task_id 启动
        self.abort_tick().await;
        *self.state.lock().await = None;

        let now = Utc::now();

        let session_id = {
            let db = self.app.state::<Db>();
            let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
            session::create_session(
                &conn,
                &new_task_id,
                "pomodoro",
                Some(&preset_str),
                Some(planned / 60),
            )?
        };

        let timer = RunningTimer {
            task_id: new_task_id,
            session_id,
            mode: "pomodoro".into(),
            preset: Some(preset_str),
            status: "running".into(),
            start_time: now,
            planned_seconds: planned,
            elapsed_seconds: 0,
            pomodoro_count: count,
            is_break: false,
            break_planned_seconds: 0,
            last_persisted_at: now,
        };

        persist_running_to_db(&self.app, &timer)?;
        let snap = snapshot_of(&timer);
        *self.state.lock().await = Some(timer);
        self.spawn_tick().await;
        let _ = self.app.emit("timer:state_changed", &snap);
        Ok(snap)
    }

    /// 延长休息(额外 extra_seconds 秒)
    pub async fn extend_break(&self, extra_seconds: i64) -> AppResult<TimerSnapshot> {
        let mut guard = self.state.lock().await;
        let t = guard
            .as_mut()
            .ok_or_else(|| AppError::Custom("无计时状态".into()))?;
        if t.status != "break_ended" {
            return Err(AppError::Custom("当前不在休息结束状态".into()));
        }
        // 回到 break 态,增加额度
        t.status = "break".into();
        t.break_planned_seconds += extra_seconds;
        t.last_persisted_at = Utc::now();
        persist_running_to_db(&self.app, t)?;
        let snap = snapshot_of(t);
        drop(guard);
        // 重新启动 tick
        self.spawn_tick().await;
        let _ = self.app.emit("timer:state_changed", &snap);
        Ok(snap)
    }

    // ---------- Week 2b: 自由模式 ----------

    /// 启动自由计时(正计时,无上限)
    pub async fn start_free(&self, task_id: String) -> AppResult<TimerSnapshot> {
        {
            let guard = self.state.lock().await;
            if guard.is_some() {
                return Err(AppError::Custom("已有计时进行中".into()));
            }
        }

        let now = Utc::now();
        let session_id = {
            let db = self.app.state::<Db>();
            let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
            session::create_session(&conn, &task_id, "free", None, None)?
        };

        let timer = RunningTimer {
            task_id: task_id.clone(),
            session_id: session_id.clone(),
            mode: "free".into(),
            preset: None,
            status: "running".into(),
            start_time: now,
            planned_seconds: 86400, // 24h 上限兜底
            elapsed_seconds: 0,
            pomodoro_count: 0,
            is_break: false,
            break_planned_seconds: 0,
            last_persisted_at: now,
        };

        persist_running_to_db(&self.app, &timer)?;
        let snap = snapshot_of(&timer);
        *self.state.lock().await = Some(timer);
        self.spawn_tick().await;
        let _ = self.app.emit("timer:state_changed", &snap);
        tracing::info!("free mode started: task={}", task_id);
        Ok(snap)
    }

    /// 手动完成自由计时
    pub async fn complete_free(&self) -> AppResult<()> {
        let taken = {
            let mut guard = self.state.lock().await;
            guard.take()
        };
        self.abort_tick().await;

        let Some(t) = taken else {
            return Err(AppError::Custom("无计时".into()));
        };
        if t.mode != "free" {
            return Err(AppError::Custom("当前不是自由模式".into()));
        }

        let actual_min = (t.elapsed_seconds / 60).max(1);
        {
            let db = self.app.state::<Db>();
            let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
            session::complete_session(&conn, &t.session_id, actual_min)?;
        }
        reset_db_timer(&self.app)?;
        let _ = self.app.emit("timer:state_changed", &idle_snapshot());
        tracing::info!("free session completed: {}min", actual_min);
        Ok(())
    }

    /// 从崩溃恢复中接管 — 读 timer_state 回填内存 RunningTimer + spawn tick
    ///
    /// 前置条件:调用方(前端 useRecovery)已确认 AutoResume 分支。
    pub async fn resume_from_crash(&self) -> AppResult<TimerSnapshot> {
        {
            let guard = self.state.lock().await;
            if guard.is_some() {
                return Err(AppError::Custom("内存已有计时,不需要崩溃恢复".into()));
            }
        }

        // 用显式作用域确保 rusqlite MutexGuard 在 .await 之前释放(否则 future 非 Send)
        let raw: CrashedTimerRow = {
            let db = self.app.state::<Db>();
            let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
            conn.query_row(
                "SELECT task_id, session_id, start_time, elapsed_seconds, planned_seconds,
                        mode, pomodoro_preset, status, pomodoro_count, is_break, break_remaining
                   FROM timer_state WHERE id = 'current'",
                [],
                |r| {
                    Ok(CrashedTimerRow {
                        task_id: r.get(0)?,
                        session_id: r.get(1)?,
                        start_time: r.get(2)?,
                        elapsed: r.get(3)?,
                        planned: r.get(4)?,
                        mode: r.get(5)?,
                        preset: r.get(6)?,
                        status: r.get(7)?,
                        pomodoro_count: r.get(8)?,
                        is_break: r.get(9)?,
                        break_remaining: r.get(10)?,
                    })
                },
            )?
        };

        let task_id = raw
            .task_id
            .ok_or_else(|| AppError::Custom("timer_state.task_id 为空".into()))?;
        let session_id = raw
            .session_id
            .ok_or_else(|| AppError::Custom("timer_state.session_id 为空".into()))?;
        let start_time = raw
            .start_time
            .as_ref()
            .ok_or_else(|| AppError::Custom("timer_state.start_time 为空".into()))
            .and_then(|s| {
                DateTime::parse_from_rfc3339(s)
                    .map(|d| d.with_timezone(&Utc))
                    .map_err(|e| AppError::Custom(format!("parse start_time: {e}")))
            })?;
        let planned_seconds = raw.planned.unwrap_or(0);
        let break_planned_seconds = raw.break_remaining.unwrap_or(0);
        let mode_str = raw.mode.unwrap_or_else(|| "pomodoro".into());

        let t = RunningTimer {
            task_id,
            session_id,
            mode: mode_str,
            preset: raw.preset,
            status: raw.status,
            start_time,
            planned_seconds,
            elapsed_seconds: raw.elapsed,
            pomodoro_count: raw.pomodoro_count,
            is_break: raw.is_break != 0,
            break_planned_seconds,
            last_persisted_at: Utc::now(),
        };

        let snap = snapshot_of(&t);
        *self.state.lock().await = Some(t);
        self.spawn_tick().await;
        let _ = self.app.emit("timer:state_changed", &snap);
        tracing::info!("timer resumed from crash");
        Ok(snap)
    }

    /// 崩溃且间隔过长 — 后端把 session 标为 abandoned 并清空 timer_state
    pub async fn abandon_from_crash(&self) -> AppResult<()> {
        // 同样用显式作用域把 rusqlite 锁限定在 .await 之前
        let (session_id, elapsed): (Option<String>, i64) = {
            let db = self.app.state::<Db>();
            let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
            conn.query_row(
                "SELECT session_id, elapsed_seconds FROM timer_state WHERE id = 'current'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?
        };
        if let Some(sid) = session_id {
            let db = self.app.state::<Db>();
            let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
            session::abandon_session(&conn, &sid, (elapsed / 60).max(0), Some("crash"))?;
        }
        reset_db_timer(&self.app)?;
        *self.state.lock().await = None;
        let _ = self.app.emit("timer:state_changed", &idle_snapshot());
        Ok(())
    }

    // ---------- 私有:tick ----------

    async fn spawn_tick(&self) {
        self.abort_tick().await;

        let state = self.state.clone();
        let app = self.app.clone();
        let handle = tokio::spawn(async move {
            let mut tk = interval(Duration::from_secs(TICK_INTERVAL_SECS));
            tk.tick().await; // consume immediate first tick
            loop {
                tk.tick().await;
                let mut guard = state.lock().await;
                let Some(t) = guard.as_mut() else {
                    return;
                };

                if t.status == "paused" {
                    continue;
                }
                if t.status != "running" && t.status != "break" {
                    continue;
                }

                t.elapsed_seconds += 1;
                let snap = snapshot_of(t);
                let _ = app.emit("timer:tick", &snap);

                // 30s 落盘
                let now = Utc::now();
                if (now - t.last_persisted_at).num_seconds() >= PERSIST_INTERVAL_SECS {
                    if let Err(e) = persist_running_to_db(&app, t) {
                        tracing::warn!("timer_state 持久化失败: {e}");
                    } else {
                        t.last_persisted_at = now;
                    }
                }

                // 自动迁移
                if t.status == "running"
                    && t.mode != "free"
                    && t.elapsed_seconds >= t.planned_seconds
                {
                    if let Err(e) = transition_focus_to_break(&app, t) {
                        tracing::error!("focus→break 迁移失败: {e}");
                    }
                    let _ = app.emit("timer:state_changed", &snapshot_of(t));
                } else if t.status == "break" && t.elapsed_seconds >= t.break_planned_seconds {
                    // break 结束 → 进入 break_ended(hold 等用户选择)
                    t.status = "break_ended".into();
                    if let Err(e) = persist_running_to_db(&app, t) {
                        tracing::warn!("break_ended 落盘失败: {e}");
                    }
                    let _ = app.emit("timer:state_changed", &snapshot_of(t));
                    return; // 退出 tick 循环
                }
            }
        });
        *self.tick_handle.lock().await = Some(handle);
    }

    async fn abort_tick(&self) {
        if let Some(h) = self.tick_handle.lock().await.take() {
            h.abort();
        }
    }
}

// ---------- 辅助(tick 内部也用) ----------

fn transition_focus_to_break(app: &AppHandle, t: &mut RunningTimer) -> AppResult<()> {
    // 1) 完成 session(actual = planned,因为 elapsed 已到 planned)
    let actual_min = t.elapsed_seconds / 60;
    {
        let db = app.state::<Db>();
        let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        session::complete_session(&conn, &t.session_id, actual_min)?;
    }

    // 2) 番茄计数 +1
    t.pomodoro_count += 1;

    // 3) 切到 break
    let preset = t.preset.clone().unwrap_or_else(|| "classic_25".into());
    t.is_break = true;
    t.status = "break".into();
    t.elapsed_seconds = 0;
    t.break_planned_seconds = compute_break_seconds(&preset, t.pomodoro_count);
    t.last_persisted_at = Utc::now();

    // 4) 落盘(让下次打开应用即便没跑到 30s 也能看到正确 break 态)
    persist_running_to_db(app, t)?;
    tracing::info!(
        "focus→break: pomodoro_count={} break_seconds={}",
        t.pomodoro_count,
        t.break_planned_seconds
    );
    Ok(())
}

fn persist_running_to_db(app: &AppHandle, t: &RunningTimer) -> AppResult<()> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE timer_state
            SET task_id = ?1,
                session_id = ?2,
                start_time = ?3,
                elapsed_seconds = ?4,
                planned_seconds = ?5,
                mode = ?6,
                pomodoro_preset = ?7,
                status = ?8,
                pomodoro_count = ?9,
                is_break = ?10,
                break_remaining = ?11,
                updated_at = ?12
          WHERE id = 'current'",
        params![
            t.task_id,
            t.session_id,
            t.start_time.to_rfc3339(),
            t.elapsed_seconds,
            t.planned_seconds,
            t.mode,
            t.preset,
            t.status,
            t.pomodoro_count,
            if t.is_break { 1i64 } else { 0 },
            if t.is_break {
                Some(t.break_planned_seconds)
            } else {
                None
            },
            now,
        ],
    )?;
    Ok(())
}

fn reset_db_timer(app: &AppHandle) -> AppResult<()> {
    let db = app.state::<Db>();
    let conn = db.0.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE timer_state
            SET task_id = NULL, session_id = NULL, start_time = NULL,
                elapsed_seconds = 0, planned_seconds = NULL,
                mode = NULL, pomodoro_preset = NULL,
                status = 'idle', pomodoro_count = 0,
                is_break = 0, break_remaining = NULL,
                updated_at = ?1
          WHERE id = 'current'",
        params![now],
    )?;
    Ok(())
}

// 消除未使用告警:TimerState 供命令返回用,这里只为了让模块导入的类型出现在同一文件
#[allow(dead_code)]
fn _unused() -> Option<TimerState> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planned_seconds_presets() {
        assert_eq!(compute_planned_seconds("classic_25"), 1500);
        assert_eq!(compute_planned_seconds("deep_45"), 2700);
        assert_eq!(compute_planned_seconds("immersive_90"), 5400);
        assert_eq!(compute_planned_seconds("unknown"), 1500); // fallback
    }

    #[test]
    fn custom_planned_seconds_requires_valid_range() {
        assert_eq!(
            resolve_planned_seconds("custom", Some(30 * 60)).unwrap(),
            1800
        );
        assert!(resolve_planned_seconds("custom", None).is_err());
        assert!(resolve_planned_seconds("custom", Some(59)).is_err());
        assert!(resolve_planned_seconds("custom", Some(181 * 60)).is_err());
    }

    #[test]
    fn break_seconds_short_vs_long() {
        // 第 1/2/3 个番茄 → 短休
        assert_eq!(compute_break_seconds("classic_25", 1), 300);
        assert_eq!(compute_break_seconds("classic_25", 2), 300);
        assert_eq!(compute_break_seconds("classic_25", 3), 300);
        // 第 4 个 → 长休
        assert_eq!(compute_break_seconds("classic_25", 4), 900);
        // 第 8 个 → 长休
        assert_eq!(compute_break_seconds("classic_25", 8), 900);
    }

    #[test]
    fn deep_and_immersive_breaks() {
        assert_eq!(compute_break_seconds("deep_45", 1), 600);
        assert_eq!(compute_break_seconds("immersive_90", 1), 900);
        assert_eq!(compute_break_seconds("custom", 1), 300);
        // 第 4 个即便 immersive 也是 15min(碰巧等于常规值)
        assert_eq!(compute_break_seconds("immersive_90", 4), 900);
    }

    #[test]
    fn break_seconds_count_zero() {
        // 未完成任何番茄(理论上不会触发 break 计算,但函数得安全)
        assert_eq!(compute_break_seconds("classic_25", 0), 300);
    }
}

---
mode: plan
cwd: C:\Users\zhx\Desktop\FocusLab
task: Phase 1 Week 2a · 番茄钟核心闭环(后端 tokio tick + 事件推送)
complexity: medium-high
stack: Tauri 2 + tokio + Rust service + Vue 3 + Pinia
depends_on: Week 1b(commit a03b9f4)
total_thoughts: 14
created_at: 2026-04-18_22-59-12
---

# Plan: Phase 1 Week 2a · 番茄钟核心闭环

## 🎯 Context

Week 1a/1b 把"应用骨架 + 数据模型 + 时间基础设施"都铺好了,现在要做 Phase 1 最核心的一块:**用户能在 FocusLab 里跑一个完整的 25 分钟番茄钟**,数据完整落到 `sessions` 表,timer_state 随之演进,UI 上有圆环倒计时和开始/暂停/继续/放弃四个操作。

架构按 docs/04 §3.4 走**后端权威计时**路线:Rust 端 tokio 任务每秒 tick + emit `timer:tick` / `timer:state_changed` 事件,前端订阅更新,自己不独立 tick。Week 3 加悬浮球窗口时可直接复用这套事件总线,零额外同步成本。

**本轮范围(Week 2a · 只走番茄核心闭环)**:
1. `sessions` 表的 CRUD + 中断 `interruptions`(仅 record 接口,弹窗在 2b)
2. Rust `TimerService`(tokio tick + 内存权威状态 + 每秒 emit + 30s 落盘 timer_state)
3. 状态变迁命令:`start_pomodoro` / `pause` / `resume` / `abandon` / `skip_break`
4. 自动状态机:focus 计时到 → 自动进 break(短休 5min / 每 4 个触发长休 15min) → break 到 → 回 idle
5. `PomodoroRing.vue`(SVG 圆环 + 倒计时数字 + 模式徽章)
6. 控制按钮组(开始 / 暂停 / 继续 / 放弃)
7. TodayView:顶部当前计时卡(非 idle 才渲染) + 每个 assignment 行的 ▶ 按钮
8. 前端 `useTimerStore` 订阅 `timer:tick` + `timer:state_changed`,本地只存快照不独立 tick

**本轮不做(留 2b)**:
- 任务编辑/删除 · 四象限分组视图 · 中断原因弹窗 · 休息三选一(继续/切换/延长) · 基础音效 · 计划锁定 plan_locked_at · 完成率按 dta · 自由模式 🌀 切换 UI · 悬浮球窗口

**为什么不全量做 Week 2**: 11 项原文里,番茄钟本体 + 状态机就够占一整次迭代的工作量与验证面。把"任务编辑"等琐碎项混进来会拖慢核心闭环验证。2a 跑通后 2b 在确定架构上做细节,风险小。

---

## 📋 Execution Plan

### Step 1 · sessions 模型 + CRUD

- `src-tauri/src/models/session.rs`:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Session {
      pub id: String,
      pub task_id: String,
      pub start_time: String,              // RFC3339
      pub end_time: Option<String>,
      pub planned_duration_minutes: Option<i64>,
      pub actual_duration_minutes: Option<i64>,
      pub mode: String,                     // pomodoro | free
      pub pomodoro_preset: Option<String>,
      pub status: String,                   // in_progress | completed | abandoned
      pub is_manual_entry: bool,
      pub abandon_reason: Option<String>,
      pub created_at: String,
  }
  ```
- **不直接暴露 session CRUD 命令给前端** — session 生命周期由 TimerService 管理,前端只通过 `start_pomodoro` / `abandon` 等语义命令间接操作
- 提供内部 helper:`create_session(conn, task_id, mode, preset, planned_min)` / `complete_session(conn, id, actual_min)` / `abandon_session(conn, id, reason, actual_min)`
- 单元测试放在 `session.rs` 同模块:至少 3 例覆盖 create / complete / abandon 的 SQL 正确性(用内存 SQLite 测)

### Step 2 · TimerService 骨架(内存权威 + tokio tick)

- `src-tauri/src/services/timer_service.rs`:
  ```rust
  pub struct TimerService {
      state: Arc<tokio::sync::Mutex<Option<RunningTimer>>>,
      tick_handle: Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>>,
      app: AppHandle,
  }

  struct RunningTimer {
      task_id: String,
      session_id: String,
      mode: String,                  // pomodoro | free (2a 只走 pomodoro)
      preset: Option<String>,        // classic_25 | deep_45 | immersive_90
      status: String,                // running | paused | break
      start_time: DateTime<Utc>,
      planned_seconds: i64,
      elapsed_seconds: i64,
      pomodoro_count: i64,           // 当前会话组里已完成的番茄数(4 的倍数触发长休)
      is_break: bool,
      break_planned_seconds: i64,    // break 时长(短 5min / 长 15min)
      last_persisted_at: DateTime<Utc>,
  }
  ```
- **权威状态放内存**,DB 的 `timer_state` 行作为"崩溃恢复副本"每 30s 被覆盖一次
- `init(app: AppHandle) -> Self` 在 lib.rs 的 setup 里调用,`app.manage(timer_service)` 注入 State
- 启动时若 DB `timer_state.status != 'idle'` → 由 Week 1b 的 check_crash_recovery 路径处理,TimerService 不自动接管

### Step 3 · 核心 tick 循环

- `spawn_tick` 内部函数:
  ```rust
  let handle = tokio::spawn(async move {
      let mut interval = tokio::time::interval(Duration::from_secs(1));
      interval.tick().await; // consume first immediate tick
      loop {
          interval.tick().await;
          let mut guard = state.lock().await;
          let Some(t) = guard.as_mut() else { break; };
          if t.status != "running" && t.status != "break" { continue; }
          t.elapsed_seconds += 1;

          // 1) 每秒 emit tick
          let snap = snapshot(t);
          let _ = app.emit("timer:tick", &snap);

          // 2) 每 30s 落盘 timer_state (防崩溃)
          if (Utc::now() - t.last_persisted_at).num_seconds() >= 30 {
              persist_timer_state(&app, &snap).await;
              t.last_persisted_at = Utc::now();
          }

          // 3) 自动状态迁移
          if t.status == "running" && t.elapsed_seconds >= t.planned_seconds {
              // focus 完成 → 进 break
              transition_focus_to_break(&mut *guard, &app).await;
          } else if t.status == "break" && t.elapsed_seconds >= t.break_planned_seconds {
              // break 完成 → 回 idle
              transition_break_to_idle(&mut *guard, &app).await;
              break; // 退出 tick 循环
          }
      }
  });
  ```
- `paused` 状态时 `elapsed_seconds` 不增,但 loop 继续空转(性能可忽略)— 或更优雅的做法是 pause 时 abort handle、resume 时重启。**MVP 选空转**保持简单

### Step 4 · 状态变迁 · start / pause / resume / abandon

- `services/timer_service.rs` 的方法:
  - `start_pomodoro(task_id: String, preset: String) -> AppResult<TimerState>`
    - 检查内存态是 None(idle),否则返回错误"已有计时进行中"
    - 计算 planned_seconds:classic_25=1500, deep_45=2700, immersive_90=5400
    - `models::session::create_session(...)` 拿 session_id
    - 写入内存 RunningTimer,start_time=now,status='running'
    - 写一次 DB timer_state(立即落盘,避免 30s 内崩溃丢失)
    - `app.emit("timer:state_changed", &snap)` + `spawn_tick`
    - 返回当前 TimerState(转成 DB-shape 便于前端用已有类型)
  - `pause() -> AppResult<TimerState>`
    - 内存 status='paused',emit state_changed,**不 abort tick**(空转)
  - `resume() -> AppResult<TimerState>`
    - 内存 status='running',emit state_changed
  - `abandon() -> AppResult<()>`
    - abort tick handle,`models::session::abandon_session(...)` 带 actual_minutes=elapsed/60
    - 内存清空,`reset_timer_state`(复用 Week 1b)
    - emit state_changed(最终 idle)
  - `skip_break() -> AppResult<()>`
    - 仅在 status='break' 时有效 → 直接 transition_break_to_idle

### Step 5 · 自动迁移 · transition_focus_to_break / transition_break_to_idle

- `transition_focus_to_break`:
  - `models::session::complete_session(conn, session_id, actual_min=planned_min)`
  - 内存 `pomodoro_count += 1`
  - 决定 break 时长:classic_25/5 · deep_45/10 · immersive_90/15;若 `pomodoro_count % 4 == 0` 触发长休 15-30min(2a 固定 15min,2b 再做偏好配置)
  - 内存 `status='break'`, `elapsed_seconds=0`, `break_planned_seconds=xxx`
  - 落盘 timer_state,emit state_changed
- `transition_break_to_idle`:
  - 内存清空,reset_timer_state,abort tick
  - emit state_changed (status='idle')
  - **不自动开始下一个番茄钟** — 留给用户主动决策(2b 做三选一交互)

### Step 6 · Tauri 命令封装

- `src-tauri/src/commands/focus_commands.rs`(新文件,避免和 timer_state CRUD 同名):
  ```rust
  #[tauri::command]
  pub async fn start_pomodoro(
      task_id: String,
      preset: String,
      timer: State<'_, TimerService>,
  ) -> AppResult<TimerState> { timer.start_pomodoro(task_id, preset).await }

  // pause / resume / abandon / skip_break 同形
  ```
- lib.rs invoke_handler! 追加这 5 条
- lib.rs setup 里 `app.manage(TimerService::new(app.handle().clone()))`

### Step 7 · 事件契约 · timer:tick / timer:state_changed

定义共享 payload:
```rust
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimerSnapshot {
    pub status: String,          // running | paused | break | idle
    pub task_id: Option<String>,
    pub session_id: Option<String>,
    pub mode: Option<String>,
    pub preset: Option<String>,
    pub elapsed_seconds: i64,
    pub planned_seconds: i64,    // focus 时是 focus 时长,break 时是 break 时长
    pub pomodoro_count: i64,
    pub is_break: bool,
}
```
- `timer:tick` 每秒 emit 一份 snapshot
- `timer:state_changed` 在每次状态跳转(start / pause / resume / abandon / focus→break / break→idle)emit 一份
- 前端两个事件用同一个 handler 更新 store;state_changed 可能带和 tick 一样的字段,前端不区分

### Step 8 · 前端 useTimerStore 整合

- `src/stores/useTimerStore.ts`(新,比 Week 1b 的 useTimerStateStore 更高层):
  - 持有 `snapshot: Ref<TimerSnapshot | null>` · 由 Tauri 事件喂
  - 订阅 `timer:tick` 和 `timer:state_changed`(用 `@tauri-apps/api/event` 的 `listen`)
  - actions:`startPomodoro(taskId, preset)` / `pause()` / `resume()` / `abandon()` / `skipBreak()`
  - 初始化时调 `get_timer_state` 拿一次快照回填(防启动前有未结束的计时)
- `useTimerStateStore`(Week 1b)保留作为底层 DB 访问,高层组件只用 useTimerStore

### Step 9 · PomodoroRing 组件

- `src/components/timer/PomodoroRing.vue`
- 结构:
  - SVG 圆环 280×280(对齐 prototype/screens/pomodoro.html:91-100)
  - 灰色 track + 彩色 arc,`stroke-dasharray=circum` / `stroke-dashoffset` 随进度变化
  - 中心区:大号倒计时数字(mm:ss) + 下方 mode 徽章(🍅 经典 25 · 🍅🍅 深度 45 · 🍅🍅🍅 沉浸 90)
  - `pomodoro_count` 点阵展示(最多 4 个,超过换行或截断)
- 配色按状态:
  - running → `--color-primary` arc
  - paused → `--color-neutral` arc + "已暂停" 角标
  - break → `--color-success` arc + "休息中" + icon
  - 最后 5 分钟 → `--color-warning` arc + 微呼吸动画(可选,preset ≥ 45 时只在 final 1 分钟亮)
- 输入 props:`snapshot: TimerSnapshot | null`;null 时占位"选择任务开始"

### Step 10 · 控制按钮组 · TimerControls

- `src/components/timer/TimerControls.vue`
- 按钮随 status 变化:
  - idle 或无 snapshot → 不展示(由父组件条件渲染)
  - running → [⏸ 暂停] [✕ 放弃]
  - paused → [▶ 继续] [✕ 放弃]
  - break → [⏭ 跳过休息]
- 放弃按钮点击后弹一个简单 confirm(原生 `confirm()` 足够 2a;2b 换成 app 内 Dialog + 原因选择)

### Step 11 · TodayView 集成 · 当前计时卡 + ▶ 按钮

- TodayView 顶部新增 `<TimerCard />` 组件,仅在 `snapshot.status !== 'idle'` 时渲染
  - 显示任务名 · PomodoroRing · TimerControls
- 任务池每行追加 ▶ 按钮(仅 `status === 'pending'` 的任务 + 当前 idle 时可点),点击调 `timerStore.startPomodoro(task_id, 'classic_25')`
  - **2a 固定 preset=classic_25**,preset 切换 UI 留给 2b 的设置页
- 每 assignment 行也追加 ▶ 按钮,同样行为
- 集成后 TodayView 要做基本 layout 调整(顶部 Timer 区占高 ≈ 400px)

### Step 12 · 崩溃恢复对接(复用 Week 1b)

- Week 1b 的 `check_crash_recovery` + RecoveryDialog 已经建好,本轮做**两件衔接事**:
  1. AutoResume 分支:从 DB 的 timer_state 还原到内存 RunningTimer + spawn_tick(Week 1b 的 TODO 落地)
  2. AutoEnd 分支:Week 1b 已调 reset_timer_state,本轮补充 `abandon_session(session_id, reason='crash', actual_min=elapsed/60)`
- 即:在 `useRecovery.checkOnMount` 里,AutoResume 调新命令 `timer_resume_from_crash`;AutoEnd 调 `timer_abandon_from_crash`
- 后端 `focus_commands` 加这两个命令(内部复用 spawn_tick / abandon 逻辑,但不重建 session,而是挂到现有 session_id)

### Step 13 · 前端事件订阅生命周期

- `useTimerStore` 创建时就 listen(模块级副作用),`unlisten` 留在模块 dispose — 但 SPA 里不会 dispose,等价于"应用生命周期内常驻"
- Tauri 2 的 `@tauri-apps/api/event` 的 `listen` 返回 `UnlistenFn`;store 保存引用,但正常流程不主动 unlisten
- 若同一事件被多次订阅要防重复 — 用模块级 `ensureListeners()` 守卫一次性订阅

### Step 14 · 验证 + 提交

**单元测试**(Rust lib):
- `models::session::tests`:内存 SQLite 跑 create / complete / abandon,断言行数 + 字段
- `services::timer_service::tests`:
  - `compute_planned_seconds(preset)` 三档对应 1500/2700/5400
  - `compute_break_seconds(count)` 4 的倍数 → 15min,其他 → 对应 5/10/15
  - classify 已在 Week 1b 覆盖,不重复

**端到端验证**(用户跑 `pnpm tauri dev`):
1. 在任务池加一条任务 → 点 ▶ → 顶部出现 25:00 圆环,每秒递减
2. ⏸ 暂停 → 圆环变灰 + 数字不动;▶ 继续 → 恢复
3. 改系统时间快进(或把 preset classic_25 临时改成 3 秒 `--debug-duration` 环境变量 → 2a 不做开关,直接肉眼等)观察:
   - focus 完成 → 圆环变绿 + "休息中 5:00"
   - break 完成 → 回 idle,圆环消失,下一个 25:00 不自动开始
4. 在 focus 中点 ✕ 放弃 → 弹 confirm → 确认 → 圆环消失,sessions 表里对应行 status=abandoned
5. focus 中关闭应用 10 秒再开 → AutoResume 分支 toast + 内存态接回计时,UI 恢复;不出现 RecoveryDialog
6. focus 中关闭应用 10 分钟再开 → RecoveryDialog 弹出,点"继续计时"→ Week 2a 的 resume_from_crash 把内存态拉起

**静态检查**:
- `cargo test --lib` 全绿
- `cargo clippy --all-targets -- -D warnings` 零警告
- `pnpm run type-check` 零错误

**提交**(2 个):
1. `feat(core): sessions CRUD + TimerService tokio tick + 番茄钟状态机 + 崩溃接续` — 后端
2. `feat(ui): PomodoroRing + TimerControls + TodayView 集成 + useTimerStore 事件订阅` — 前端

---

## ⚠️ 风险与预案

1. **tokio::sync::Mutex vs std::sync::Mutex 的选择** · 本轮 TimerService 状态在 async 上下文中持有跨 `.await`,必须用 `tokio::sync::Mutex`。但 `Db.0: std::sync::Mutex<Connection>` 是同步锁 — 不要混用。落盘时 `let conn = db.0.lock().map_err(...)` 是同步调用,短暂持锁(~1ms)没问题。

2. **Tick loop 的 abort 时机** · pause 选"空转"而非 abort,是为了避免 abort + respawn 的状态恢复复杂度;但这意味 paused 下每秒仍有一次锁获取 + emit(emit 内容 status='paused')。若前端对此不希望,可在 tick loop 里加 `if status=='paused' { continue; }`(只做 skip、不 emit)。**决定**:跳过 emit,保持锁获取,最小改动。

3. **`interval.tick()` 首次立即返回** · tokio 的 `interval` 首次 tick 会立即返回。代码里先 `interval.tick().await` 吃掉那一次,再进循环,避免 start 后第一秒就 +1 累计。

4. **事件溢出** · 1 事件/秒,Tauri 事件系统能轻松承受。若后续加悬浮球 + 快捷面板各自 emit,需要广播而不是 send — Tauri 2 的 `emit` 默认广播所有窗口,符合预期。

5. **崩溃恢复的 pomodoro_count 归属** · 跨应用重启后,`pomodoro_count` 是否继续?当前设计:pomodoro_count 存在内存和 timer_state 表里。恢复时从 timer_state 读回,内存 RunningTimer 用该值初始化。

6. **Session 孤儿行** · 若用户在 focus 中硬关闭进程(任务管理器杀掉),session 状态仍是 in_progress,不会自动 abandon。下次启动 Week 1b 的崩溃恢复流程读 timer_state 发现非 idle → 走 AutoEnd/AskUser/AutoResume,其中 AutoEnd 和 AskUser→结束 都要 abandon_session。本 Step 12 已覆盖。

7. **前端圆环 SVG 性能** · 每秒重绘 stroke-dashoffset 是 GPU 友好操作,没有问题。若后续加粒子效果,才需要 offload 到 Canvas。

8. **`--debug-duration` 加速测试** · 2a 不加,验证阶段可临时改源码把 classic_25 的 1500 改成 15 秒测完整流程,验证后再改回(或者 2b 加一个 DEV-only 开关)。

---

## 📎 关键文件路径(本轮新增/修改)

**新增**:
- `src-tauri/src/models/session.rs`
- `src-tauri/src/services/timer_service.rs`(替换原空 mod.rs 的内容)
- `src-tauri/src/commands/focus_commands.rs`
- `src/stores/useTimerStore.ts`
- `src/components/timer/PomodoroRing.vue`
- `src/components/timer/TimerControls.vue`
- `src/components/timer/TimerCard.vue`

**修改**:
- `src-tauri/src/services/mod.rs`(导出 timer_service)
- `src-tauri/src/commands/mod.rs`(导出 focus_commands)
- `src-tauri/src/models/mod.rs`(导出 session)
- `src-tauri/src/lib.rs`(setup 里 manage TimerService,invoke_handler 注册 5-7 条新命令)
- `src/composables/useRecovery.ts`(AutoResume / AutoEnd 分支接真实恢复)
- `src/types.ts`(追加 TimerSnapshot、PomodoroPreset)
- `src/views/TodayView.vue`(顶部插 TimerCard · 任务池 + 计划行加 ▶)

**只读对标**:
- `docs/02 §2.1-2.2`(番茄钟模式 · 状态机 · 中断处理)
- `docs/04 §3.4`(Tauri Event System 契约)
- `docs/04 §7.2`(sessions / interruptions DDL,已在 schema 里)
- `prototype/screens/pomodoro.html:91-150`(圆环 SVG 参考)

---

## ✅ 验证(端到端)

成功判定 = "用户能跑完一个 25 分钟番茄钟,数据入 sessions,崩溃可恢复":

1. `cargo test --lib` 全绿(新增 session ≥3 例 + service ≥2 例)
2. `cargo clippy --all-targets -- -D warnings` 零警告
3. `pnpm run type-check` 零错误
4. `pnpm tauri dev` 能完成 Step 14 的 6 条手动清单
5. sqlite 里 sessions 表有完整行(in_progress → completed 或 abandoned),`planned_duration_minutes=25`、`mode='pomodoro'`、`pomodoro_preset='classic_25'`
6. `git status` 干净,两次 commit 入库

若任何一项失败:停下,对照 docs/02 §2.1 状态机图和 docs/04 §3.4 事件契约定位问题,**不降级不 mock**(CLAUDE.md §2.2)。

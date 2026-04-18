---
mode: plan
cwd: C:\Users\zhx\Desktop\FocusLab
task: Phase 1 Week 1b · timer_state 持久化 + 逻辑日 + 崩溃恢复 + DTA CRUD
complexity: medium
stack: Tauri 2 + rusqlite + chrono/chrono-tz + Vue 3 + Pinia
depends_on: Week 1a(commit ec99764)
total_thoughts: 12
created_at: 2026-04-18_22-00-41
---

# Plan: Phase 1 Week 1b · v1.1 基础设施 4 件套

## 🎯 Context

Week 1a 已完成 Tauri+Vue 骨架、SQLite 全量 schema、最小 task CRUD,主题系统跑通。下一步要补齐 docs/06 §Week 1 里剩下的 4 条 v1.1 任务,它们是 Week 2(番茄钟)、Week 3(日结算)后续一切时间统计的**前置依赖**:

1. **timer_state 持久化 CRUD** — 单行状态表的读写;Week 2 番茄钟服务每 30 秒落盘靠它
2. **getLogicalDate 工具函数** — 所有按日期统计的 SQL(日结算、streak、dta 分配)都要用 `logical_date_range` 算 UTC 范围
3. **崩溃恢复三档流程** — 启动时读 `timer_state`,按 gap 时长走三档策略;docs/04 §11.2 已有状态机
4. **daily_task_assignments CRUD** — 任务与"哪一天的计划"的显式关联;完成率 / 顺延 / 锁计划都挂这张表

**本轮不做**:
- 番茄钟实际计时 / 状态机 / 30s 定时写盘 — Week 2
- carry_over 自动顺延 / 日结算补算 / 到期自动置顶 — Week 3
- 完整的 FTUE 首启引导 — Week 4

**为什么这四条放一起**: 它们互相耦合(崩溃恢复读 timer_state + 要算 gap 需要 chrono,dta 的 plan_date 必须是逻辑日),拆开反而各自需要留桩或 mock,合在一次推更干净。

---

## 📋 Execution Plan

### Step 1 · utils/datetime.rs 逻辑日工具函数

对齐 docs/04 §10.3 的 Rust 契约。

- **依赖**: `chrono::{DateTime, Utc, NaiveDate, Datelike, Timelike, Duration}`、`chrono_tz::Tz`、`chrono::Local`(系统时区)
- **函数签名**:
  ```rust
  pub fn to_logical_date(utc: DateTime<Utc>, tz: Tz, boundary_hour: u32) -> NaiveDate
  pub fn logical_date_range(date: NaiveDate, tz: Tz, boundary_hour: u32)
      -> (DateTime<Utc>, DateTime<Utc>)
  pub fn current_logical_date(boundary_hour: u32) -> NaiveDate  // 走 chrono::Local
  ```
- **时区取值**: MVP 阶段固定用 `chrono_tz::Asia::Shanghai`(docs/README 已注默认 'Asia/Shanghai'),跨时区留给 Phase 4。把常量 `pub const DEFAULT_TZ: Tz = chrono_tz::Asia::Shanghai;` 暴露出来,其他模块引用它。
- **单元测试**(`#[cfg(test)]`,至少 6 例覆盖边界):
  - 上午 10:00 → 当天(boundary=4)
  - 凌晨 3:00 → 前一天
  - 恰好 04:00:00 → 当天(边界归属下一个逻辑日)
  - 月初 03:30 → 前一月最后一天
  - 年初 01-01 03:30 → 前一年 12-31
  - `logical_date_range` 返回的 end - start == 24h

### Step 2 · settings 读写辅助

`day_boundary_hour` 在 settings 表里是 TEXT(值 "4"),需要一个统一读取入口。

- 新建 `src-tauri/src/models/settings.rs`:
  ```rust
  pub fn get(conn: &Connection, key: &str) -> AppResult<Option<String>>
  pub fn set(conn: &Connection, key: &str, value: &str) -> AppResult<()>
  pub fn get_u32(conn: &Connection, key: &str) -> AppResult<Option<u32>>
  ```
- `models/mod.rs` 追加 `pub mod settings;`
- 便利函数 `get_boundary_hour(conn) -> u32`(缺省返回 4)在后续命令里复用

### Step 3 · timer_state 模型 + CRUD 命令

- `src-tauri/src/models/timer_state.rs`:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct TimerState {
      pub task_id: Option<String>,
      pub session_id: Option<String>,
      pub start_time: Option<String>,       // RFC3339
      pub elapsed_seconds: i64,
      pub planned_seconds: Option<i64>,
      pub mode: Option<String>,             // pomodoro | free
      pub pomodoro_preset: Option<String>,
      pub status: String,                   // running | paused | break | idle
      pub pomodoro_count: i64,
      pub is_break: bool,
      pub break_remaining: Option<i64>,
      pub updated_at: String,
  }
  ```
- `src-tauri/src/commands/timer_commands.rs` 暴露 3 个命令:
  - `get_timer_state(db) -> TimerState` — 读 id='current' 单行
  - `update_timer_state(patch: TimerStatePatch, db) -> ()` — UPSERT 补丁,空字段不动
  - `reset_timer_state(db) -> ()` — 把全部字段清空回 idle,`updated_at=now`
- `TimerStatePatch` 是 `Option<...>` 包裹的同名结构,方便前端只传变化的字段
- 注册到 `lib.rs` 的 `invoke_handler!`

### Step 4 · 崩溃恢复 · 后端判定

- `src-tauri/src/commands/recovery_commands.rs`:
  ```rust
  #[derive(Serialize)]
  pub struct RecoveryInfo {
      pub state: TimerState,
      pub task_name: Option<String>,        // 连表查 tasks
      pub gap_seconds: i64,
      pub recommendation: RecoveryAction,   // AutoResume / AskUser / AutoEnd / None
  }
  pub enum RecoveryAction { AutoResume, AskUser, AutoEnd, None }

  #[tauri::command]
  pub fn check_crash_recovery(db) -> AppResult<Option<RecoveryInfo>>
  ```
- 实现规则(docs/04 §11.2):
  - `status='idle'` → 返回 `None`,正常启动
  - `gap < 120s` → `AutoResume`
  - `120s ≤ gap < 3600s` → `AskUser`
  - `gap ≥ 3600s` → `AutoEnd`
- 前端负责具体动作(AutoEnd 需要调 `reset_timer_state` + 补写 session.status='abandoned');后端只给建议不做副作用,保持"**debug-first,不做静默处理**"(CLAUDE.md §2.2)
- 单元测试:构造 now 与 updated_at 差值,断言 recommendation 分类正确

### Step 5 · daily_task_assignments 模型 + CRUD

- `src-tauri/src/models/daily_task_assignment.rs`:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct DailyTaskAssignment {
      pub id: String,
      pub plan_date: String,          // "YYYY-MM-DD"
      pub task_id: String,
      pub is_planned: bool,
      pub source: String,
      pub day_status: String,
      pub added_at: String,
      pub completed_at: Option<String>,
      pub sort_order: i64,
  }
  ```
- `src-tauri/src/commands/assignment_commands.rs`:
  - `list_assignments(plan_date: Option<String>, db) -> Vec<DailyTaskAssignment>` —
    `plan_date=None` 时用 `current_logical_date(boundary_hour)` 作为今天
  - `create_assignment(input: CreateAssignmentInput, db) -> DailyTaskAssignment` —
    字段: task_id / plan_date(可选,缺省今天) / source(缺省 manual) / is_planned(缺省 true)
  - `update_assignment_status(id: String, day_status: String, db) -> ()` —
    `day_status=completed` 时同步 `completed_at=now`
  - `remove_assignment(id: String, db) -> ()` — 硬删(不是搁置语义)
- 唯一键冲突(同天同任务)要返回友好错误而非 rusqlite 的 `CHECK constraint failed`
- Week 3 的 `carry_over` / 自动顺延不在本轮

### Step 6 · lib.rs 注册新命令 + 启动钩子

- `invoke_handler!` 追加:
  - `timer_commands::{get_timer_state, update_timer_state, reset_timer_state}`
  - `recovery_commands::check_crash_recovery`
  - `assignment_commands::{list_assignments, create_assignment, update_assignment_status, remove_assignment}`
- `setup` 钩子不自动做恢复动作,只做数据库初始化;恢复流程由前端 `App.vue` onMounted 触发 `check_crash_recovery` 决定

### Step 7 · 前端 types.ts 扩展

增补:
```ts
export type TimerStatus = "idle" | "running" | "paused" | "break";
export type RecoveryAction = "AutoResume" | "AskUser" | "AutoEnd" | "None";

export interface TimerState { /* 对齐后端 */ }
export interface TimerStatePatch { /* Option 版本 */ }
export interface RecoveryInfo { state; taskName; gapSeconds; recommendation }
export interface DailyTaskAssignment { /* 对齐后端 */ }
```

### Step 8 · 前端 stores/composables

- `src/stores/useTimerStateStore.ts` — 包装 get/update/reset 命令,保存最近一次的 state 快照(不含 ticking 逻辑,Week 2 再加)
- `src/stores/useAssignmentStore.ts` — list/create/update/remove,当前 `plan_date` 状态
- `src/composables/useRecovery.ts` — 单例 composable:
  - `checkOnMount()` 调后端,按 recommendation 分发:
    - `AutoResume` → toast "已自动恢复计时 (gap Xs)"(Week 2 才实际恢复计时,这里先只 toast)
    - `AskUser` → 把 `recoveryInfo` 推到 `useRecoveryStore`,由 Dialog 组件显示
    - `AutoEnd` → 调 `reset_timer_state`,toast "崩溃时间过长,已自动结束上次会话"
    - `None` → 无事发生
- `src/stores/useRecoveryStore.ts` — 持有 `visible` + `info`,供 Dialog 绑定

### Step 9 · RecoveryDialog 组件

- `src/components/recovery/RecoveryDialog.vue`
- 布局: 卡片式模态,显示 `任务名` + `已经过 XX 分钟` + 三个操作:
  - **继续计时** — 保留 timer_state,关闭 Dialog(Week 2 接真实恢复)
  - **结束会话** — 调 `reset_timer_state`,关闭
  - **丢弃** — 同上,但文案区分(Week 2 时会让 session status=abandoned,当前一致即可)
- 用 tokens.css 变量,不引新库;蒙层 `--color-bg` 60% + `--shadow-modal`
- 挂在 `App.vue` 最外层,受 `useRecoveryStore.visible` 控制

### Step 10 · 启动时集成 + SettingsView 调试面板

- `App.vue` onMounted 调 `useRecovery().checkOnMount()`
- `SettingsView.vue` 追加「🧪 调试面板」区域(仅开发态显示):
  - 按钮「模拟崩溃 · running 30s 前」→ 调 `update_timer_state` 把 status=running / updated_at=30s 前 → 刷新页面应见 AutoResume toast
  - 按钮「模拟崩溃 · running 10min 前」→ 同上但 10min → 应见 AskUser Dialog
  - 按钮「模拟崩溃 · running 2h 前」→ 同上但 2h → 应见 AutoEnd toast
  - 按钮「清空 timer_state」→ reset_timer_state
- 用于 UI 侧快速验证三档分支

### Step 11 · 验证 + 提交

1. **Rust 单元测试**: `cargo test --manifest-path src-tauri/Cargo.toml` 全绿(至少 10 例:datetime 6 + recovery 4)
2. **静态检查**: `cargo clippy --all-targets -- -D warnings` 无警告;`pnpm run type-check` 绿灯
3. **端到端 UI 验证**(由用户跑 `pnpm tauri dev`):
   - 「今日」页输入任务 → 回到「设置 → 🧪 调试」点「添加到今日计划」(需要顺带做一个 mini UI,见 Step 9 附注)→ 返回「今日」能看到 assignment 列表
   - 设置页「模拟崩溃 · 10min」→ 重启应用 → 出现 RecoveryDialog → 点「结束会话」→ 弹窗关闭,timer_state 回到 idle
   - 设置页「模拟崩溃 · 30s」→ 重启 → 无 Dialog,右上角 toast「已自动恢复计时」
   - 设置页「模拟崩溃 · 2h」→ 重启 → 无 Dialog,toast「自动结束」
4. **提交**:
   - Commit 1(后端基础设施): `feat(core): timer_state CRUD + 逻辑日工具 + 崩溃恢复判定 + dta CRUD`
   - Commit 2(前端恢复流程): `feat(ui): 崩溃恢复对话框 + 调试面板 + 前端 store`

---

## ⚠️ 风险与预案

1. **逻辑日边界相等情况语义** · 04:00 整点属于"当天"还是"前一天"?docs/04 §10.1 示例 `7/10 04:00:00 ~ 7/11 03:59:59`,即 `<` 前一天,`>=` 当天。单元测试里显式覆盖 04:00:00.000 和 03:59:59.999 两侧。

2. **chrono_tz 未启用 serde 特性** · chrono-tz 0.9 的 `Tz` 默认不实现 Serialize。MVP 里 Tz 只在后端用,不往前端透传,避免问题。如果要暴露,改用 IANA 字符串 `"Asia/Shanghai"` 做序列化载体。

3. **timer_state 的 `BOOLEAN is_break`** · SQLite 里是 INTEGER 0/1,rusqlite 映射到 Rust `bool` 需要 `row.get::<_, bool>()` 隐式转换 — rusqlite 0.31 已支持 bool FromSql。若报 `InvalidColumnType`,改手动 `row.get::<_, i64>("is_break").map(|v| v != 0)`。

4. **UPSERT 的并发** · Week 2 番茄钟每 30 秒写 timer_state,Week 1b 的 update 命令和 Week 2 定时器可能并发。MVP 的 `Mutex<Connection>` 天然串行化所有写,不会有竞争。一旦切换连接池再评估。

5. **启动时检测时机** · `App.vue` onMounted 调恢复时,Pinia + Router 已挂好,但**数据库还在 Tauri setup 的错误后果范围**。如果 setup 失败整个 app 不会起,所以 onMounted 能跑 = db 可用。

6. **模拟崩溃调试面板生产环境风险** · 如果带到正式版会暴露 reset 按钮。用 `import.meta.env.DEV` 做运行期 gate,非 dev 下整块不渲染。

7. **uniqueness 冲突的错误文案** · `daily_task_assignments(plan_date, task_id)` 唯一键命中时,rusqlite 返回 `SQLITE_CONSTRAINT_UNIQUE`。在 `create_assignment` 里检测这个错误码并转成 `AppError::Custom("该任务今天已在计划里")`,不要让底层信息穿透。

---

## 📎 关键文件路径(本轮新增/修改)

**新增**:
- `src-tauri/src/utils/datetime.rs`(替换占位)
- `src-tauri/src/models/settings.rs`
- `src-tauri/src/models/timer_state.rs`
- `src-tauri/src/models/daily_task_assignment.rs`
- `src-tauri/src/commands/timer_commands.rs`
- `src-tauri/src/commands/recovery_commands.rs`
- `src-tauri/src/commands/assignment_commands.rs`
- `src/composables/useRecovery.ts`
- `src/stores/useTimerStateStore.ts`
- `src/stores/useAssignmentStore.ts`
- `src/stores/useRecoveryStore.ts`
- `src/components/recovery/RecoveryDialog.vue`

**修改**:
- `src-tauri/src/lib.rs`(注册新命令)
- `src-tauri/src/models/mod.rs`(pub mod)
- `src-tauri/src/commands/mod.rs`(pub mod)
- `src/App.vue`(挂 Dialog + onMounted 检测)
- `src/types.ts`(追加类型)
- `src/views/SettingsView.vue`(dev 调试面板)

**只读对标**:
- `docs/04-技术架构文档.md` §10.3 / §11.2 / §7.2(timer_state DDL / dta DDL)
- `docs/02-功能设计文档.md` §5.5(跨天边界规则)
- `docs/06-开发计划与风险管理.md` §Week 1 v1.1 四条
- `src-tauri/src/db/migrations/001_init.sql`(已有的表 DDL)

---

## ✅ 验证(端到端)

1. `cargo test` 全绿(预期 ≥10 用例)
2. `cargo clippy --all-targets -- -D warnings` 无 warning
3. `pnpm run type-check` 无 error
4. `pnpm tauri dev` 起窗口,三档崩溃模拟按钮触发对应分支
5. 创建 assignment → 在 TodayView 输出(TodayView 在本轮可小改接入 assignment 列表替代原来的纯任务列表)
6. `git status` 干净,两次 commit 入库

若单元测试失败 / clippy 报 warning / UI 三档有一档不触发:停下,定位根因后再推进,**不接受跳过失败 case 或增加静默 fallback**(CLAUDE.md §2.2 Debug-First)。

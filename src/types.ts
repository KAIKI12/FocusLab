/**
 * 前端类型定义 — 与后端 Rust 的 serde 序列化结果对齐。
 *
 * 字段命名约定:
 *   - 命令参数(invoke args):后端声明 `#[serde(rename_all = "camelCase")]` 的用 camelCase,
 *     否则 snake_case。需要时在下方对应类型上标注。
 *   - 返回值:Task / DailyTaskAssignment 依赖 serde 默认,故为 snake_case;
 *     AssignmentWithTask / RecoveryInfo 用了 camelCase 注解。
 */

// ---------- Task ----------

export interface Task {
  id: string;
  name: string;
  description: string | null;
  quadrant: string;
  status: string;
  estimated_minutes: number | null;
  due_date: string | null;
  shelved_at: string | null;
  created_at: string;
  updated_at: string;
  completed_at: string | null;
}

export interface CreateTaskInput {
  name: string;
  quadrant?: string;
}

export interface UpdateTaskInput {
  id: string;
  name?: string;
  description?: string;
  quadrant?: string;
  estimatedMinutes?: number;
  dueDate?: string;
}

// ---------- TimerState ----------

export type TimerStatus = "idle" | "running" | "paused" | "break" | "break_ended";

export interface TimerState {
  task_id: string | null;
  session_id: string | null;
  start_time: string | null;
  elapsed_seconds: number;
  planned_seconds: number | null;
  mode: "pomodoro" | "free" | null;
  pomodoro_preset: string | null;
  status: TimerStatus;
  pomodoro_count: number;
  is_break: boolean;
  break_remaining: number | null;
  updated_at: string;
}

/**
 * TimerState 部分更新补丁 — 只传要改的字段。
 * 后端已声明 `#[serde(rename_all = "camelCase")]`,所以这里全走 camelCase。
 */
export interface TimerStatePatch {
  taskId?: string;
  sessionId?: string;
  startTime?: string;
  elapsedSeconds?: number;
  plannedSeconds?: number;
  mode?: "pomodoro" | "free";
  pomodoroPreset?: string;
  status?: TimerStatus;
  pomodoroCount?: number;
  isBreak?: boolean;
  breakRemaining?: number;
  /** 仅调试用:模拟崩溃场景覆盖 updated_at。生产代码不要传。 */
  updatedAt?: string;
}

// ---------- TimerSnapshot (Week 2a) ----------

export type PomodoroPreset = "classic_25" | "deep_45" | "immersive_90";

export interface TimerSnapshot {
  status: TimerStatus;
  taskId: string | null;
  sessionId: string | null;
  mode: "pomodoro" | "free" | null;
  preset: PomodoroPreset | null;
  elapsedSeconds: number;
  plannedSeconds: number;
  pomodoroCount: number;
  isBreak: boolean;
}

// ---------- Interruption (Week 2b) ----------

export type InterruptionReason =
  | "phone_message"
  | "colleague"
  | "rest"
  | "distraction"
  | "errand"
  | "other";

export interface CreateInterruptionInput {
  sessionId: string;
  reason?: InterruptionReason;
  note?: string;
}

// ---------- Recovery ----------

export type RecoveryAction = "AutoResume" | "AskUser" | "AutoEnd";

export interface RecoveryInfo {
  state: TimerState;
  taskName: string | null;
  gapSeconds: number;
  recommendation: RecoveryAction;
}

// ---------- DailyTaskAssignment ----------

export type DayStatus =
  | "pending"
  | "completed"
  | "carried_forward"
  | "shelved"
  | "cancelled";

export type AssignmentSource =
  | "manual"
  | "carried_over"
  | "ai_suggested"
  | "guided"
  | "recurring"
  | "auto_due_pinned";

export interface DailyTaskAssignment {
  id: string;
  plan_date: string;
  task_id: string;
  is_planned: boolean;
  source: AssignmentSource;
  day_status: DayStatus;
  added_at: string;
  completed_at: string | null;
  sort_order: number;
}

/** list_assignments 的联表返回(tasks × dta),走 camelCase。 */
export interface AssignmentWithTask {
  id: string;
  planDate: string;
  taskId: string;
  taskName: string;
  taskQuadrant: string;
  isPlanned: boolean;
  source: AssignmentSource;
  dayStatus: DayStatus;
  addedAt: string;
  completedAt: string | null;
  sortOrder: number;
}

export interface CreateAssignmentInput {
  taskId: string;
  planDate?: string;
  source?: AssignmentSource;
  isPlanned?: boolean;
}

// ---------- CompletionStats (Week 2b) ----------

export interface CompletionStats {
  planDate: string;
  isLocked: boolean;
  plannedCount: number;
  completedCount: number;
  extraCount: number;
  extraCompleted: number;
  completionRate: number;
}

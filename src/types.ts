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
  is_background: boolean;
  is_recurring: boolean;
  recurrence_rule: string | null;
  milestone_id: string | null;
  shelved_at: string | null;
  created_at: string;
  updated_at: string;
  completed_at: string | null;
}

export interface CreateTaskInput {
  name: string;
  quadrant?: string;
  recurrenceRule?: string;
}

export interface UpdateTaskInput {
  id: string;
  name?: string;
  description?: string;
  quadrant?: string;
  estimatedMinutes?: number;
  dueDate?: string;
  isBackground?: boolean;
  milestoneId?: string;
  recurrenceRule?: string;
  /** 三态轮转:pending | in_progress | completed */
  status?: "pending" | "in_progress" | "completed";
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

export type PomodoroPreset = "classic_25" | "deep_45" | "immersive_90" | "custom";

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
  /** 任务全局生命周期:pending | in_progress | completed */
  taskStatus: "pending" | "in_progress" | "completed";
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

export type InspirationVerification =
  | "none"
  | "needs_check"
  | "possibly_wrong"
  | "verified"
  | "overturned"
  | "resolved"; // legacy: 早期版本用过,保留兼容,语义等同 "none"
export type InspirationRelation = "related" | "contradicts";

export interface InspirationRecord {
  id: string;
  content: string;
  goalId: string | null;
  summary: string | null;
  keywords: string[];
  verification: InspirationVerification;
  embeddingStatus: "pending" | "done" | "failed";
  convertedTaskId: string | null;
  convertedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface InspirationLink {
  id: string;
  sourceId: string;
  targetId: string;
  relation: InspirationRelation;
  sourceType: "manual" | "ai_accepted";
  reason: string | null;
  createdAt: string;
}

export interface InspirationRecommendation {
  candidateId: string;
  candidateContent: string;
  relation: InspirationRelation;
  reason: string;
  confidence: number;
}

// ---------- Goal + Milestone (Phase 2) ----------

export interface Goal {
  id: string;
  name: string;
  description: string | null;
  status: string;
  target_date: string | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
  completed_at: string | null;
}

export interface Milestone {
  id: string;
  goal_id: string;
  name: string;
  description: string | null;
  status: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
  completed_at: string | null;
  target_date: string | null;
}

export interface MilestoneNote {
  id: string;
  milestone_id: string;
  text: string;
  created_at: string;
}

export interface WeeklyInvestBucket {
  /** 0=周一 .. 6=周日 */
  weekday: number;
  minutes: number;
}

export interface WeeklyInvest {
  buckets: WeeklyInvestBucket[];
  totalMinutes: number;
  todayMinutes: number;
}

export interface GoalWeeklyInvest {
  goalId: string;
  goalName: string;
  totalMinutes: number;
}

// ---------- MicroReview ----------

/** 微复盘触发场景 · 静默场景(q3/q4 事务性等)直接不渲染组件 */
export type MicroReviewScenario = "deviation" | "q1" | "milestone";

export interface CreateGoalInput {
  name: string;
  description?: string;
  targetDate?: string;
}

export interface CreateMilestoneInput {
  goalId: string;
  name: string;
  description?: string;
}

// ---------- Settlement (Phase 2) ----------

export interface Settlement {
  id: string;
  settleDate: string;
  totalTasks: number;
  completedTasks: number;
  extraTasks: number;
  shelvedTasks: number;
  completionRate: number;
  totalFocusMinutes: number;
  totalPomodoros: number;
  totalInterruptions: number;
  grade: "S" | "A" | "B" | "C";
  longestFocusTaskId: string | null;
  longestFocusMinutes: number | null;
  aiSummary: string | null;
  userReflection: string | null;
  triggerType: string;
  createdAt: string;
  /** 晚间情绪 1-5,NULL = 未打卡 */
  eveningMood: number | null;
  /** 早晨意图档位 1-5,NULL = 未打卡 */
  morningIntent: number | null;
}

export interface YesterdaySummary {
  settleDate: string;
  completedTasks: number;
  totalTasks: number;
  completionRate: number;
  grade: "S" | "A" | "B" | "C";
  totalFocusMinutes: number;
  totalPomodoros: number;
  longestFocusTaskName: string | null;
  carriedOverCount: number;
}

/** 昨日未结算检测：仅当昨日有 planned 任务且未结算时返回 */
export interface UnsettledYesterday {
  settleDate: string;
  plannedTasks: number;
  completedTasks: number;
}

export interface SettleInput {
  planDate?: string;
  triggerType?: string;
  userReflection?: string;
  eveningMood?: number | null;
  morningIntent?: number | null;
}

// ---------- ManualSession (手动补录) ----------

export interface ManualSessionInput {
  taskId: string;
  startTime: string;
  durationMinutes: number;
  mode?: string;
}

// ---------- DaySummary (日历视图) ----------

export interface DaySummary {
  settleDate: string;
  completedTasks: number;
  totalTasks: number;
  grade: "S" | "A" | "B" | "C";
  totalFocusMinutes: number;
  totalPomodoros: number;
}

// ---------- Stats (数据洞察) ----------

export interface HeatmapCell {
  dayOfWeek: number;
  hour: number;
  minutes: number;
}

export interface TrendPoint {
  date: string;
  completionRate: number;
  completed: number;
  total: number;
  focusMinutes: number;
}

export interface CategoryTime {
  quadrant: string;
  minutes: number;
  sessionCount: number;
}

export interface StatsOverview {
  totalFocusMinutes: number;
  totalSessions: number;
  totalPomodoros: number;
  totalTasksCompleted: number;
  avgDailyFocus: number;
  bestGradeCount: number;
  currentStreak: number;
}

export interface BadgeExtraStats {
  hasEarlySession: boolean;
  hasNightSession: boolean;
  hasMidnightSession: boolean;
  has3amSession: boolean;
  morning3PomodoroDay: boolean;
  evening2PomodoroDay: boolean;
  maxDayFocusMinutes: number;
  has90minSession: boolean;
  freeModeCount: number;
  zeroInterruptionDays: number;
  completedMilestones: number;
}

/**
 * MicroReview 场景决策 · 从 PomodoroView 的 watch 抽出,便于测试。
 *
 * 决策规则(对齐 prototype/screens/micro-review.html):
 *   1. 日内已弹 ≥3 次 → 静默
 *   2. 关联里程碑    → milestone
 *   3. 紧急重要(Q1) → q1
 *   4. |偏差| > 30%  → deviation
 *   5. 其余          → 静默
 */

import type { MicroReviewScenario, Task } from "@/types";

/** 日内弹窗计数的 localStorage key 前缀(每天一把) */
export const MICRO_REVIEW_COUNT_PREFIX = "fl-micro-review-count-";

/** 日内最大弹窗次数;超过此值全部静默,避免复盘疲劳 */
export const MICRO_REVIEW_DAILY_CAP = 3;

export interface ResolveReviewScenarioArgs {
  task: Task | null;
  actualMinutes: number;
  /** 本日弹窗计数(调用方从 localStorage 读) */
  todayCount: number;
}

export function resolveReviewScenario(args: ResolveReviewScenarioArgs): MicroReviewScenario | null {
  const { task, actualMinutes, todayCount } = args;
  if (!task) return null;
  if (todayCount >= MICRO_REVIEW_DAILY_CAP) return null;

  if (task.milestone_id) return "milestone";
  if (task.quadrant === "important_urgent") return "q1";

  const est = task.estimated_minutes;
  if (est && est > 0) {
    const dev = Math.abs((actualMinutes - est) / est);
    if (dev > 0.3) return "deviation";
  }
  return null;
}

/** 读取今日已弹次数(供调用方或测试独立调用) */
export function readTodayCount(today: string = new Date().toISOString().slice(0, 10)): number {
  try {
    return Number(localStorage.getItem(MICRO_REVIEW_COUNT_PREFIX + today) ?? "0");
  } catch {
    return 0;
  }
}

/** +1 今日弹窗计数 */
export function incrementTodayCount(today: string = new Date().toISOString().slice(0, 10)): void {
  try {
    const n = readTodayCount(today);
    localStorage.setItem(MICRO_REVIEW_COUNT_PREFIX + today, String(n + 1));
  } catch {
    /* localStorage 不可用时静默 */
  }
}

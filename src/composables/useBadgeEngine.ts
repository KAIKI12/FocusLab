/**
 * useBadgeEngine · 徽章解锁判定引擎。
 *
 * 数据源全部来自现有后端命令,不引入新 schema:
 *   - list_day_summaries: 番茄/专注/评级/连续打卡类徽章
 *   - list_goals (active + all): 目标类徽章
 *
 * 首次解锁时刻无历史回溯(后端 settlements 表不记 badge),用 localStorage
 * 记"第一次被观察到解锁"的日期 → 展示用。
 *
 * 未接入的徽章(需要 session preset / 时段分布 / AI 拆解计数 / 节日类)
 * 在 evaluateUnlocked 里默认 return false,后续补齐。
 */

import { ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import type { DaySummary, Goal } from "@/types";

export interface BadgeStats {
  totalPomodoros: number;
  totalFocusMinutes: number;
  maxDayPomodoros: number;
  currentStreakDays: number;
  maxStreakDays: number;
  hasSGrade: boolean;
  hasAGrade: boolean;
  maxConsecutiveAOrAbove: number;
  maxMonthSCount: number;
  maxAscendingStreak: number;
  hasPerfectMonth: boolean;
  activeGoalCount: number;
  completedGoalCount: number;
  totalGoalCount: number;
  loaded: boolean;
}

const EMPTY_STATS: BadgeStats = {
  totalPomodoros: 0,
  totalFocusMinutes: 0,
  maxDayPomodoros: 0,
  currentStreakDays: 0,
  maxStreakDays: 0,
  hasSGrade: false,
  hasAGrade: false,
  maxConsecutiveAOrAbove: 0,
  maxMonthSCount: 0,
  maxAscendingStreak: 0,
  hasPerfectMonth: false,
  activeGoalCount: 0,
  completedGoalCount: 0,
  totalGoalCount: 0,
  loaded: false,
};

const GRADE_RANK: Record<string, number> = { S: 4, A: 3, B: 2, C: 1 };

const UNLOCKED_KEY = "fl-badge-unlocked-at";

function readUnlockedMap(): Record<string, string> {
  try {
    const raw = localStorage.getItem(UNLOCKED_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw);
    return typeof parsed === "object" && parsed ? parsed : {};
  } catch {
    return {};
  }
}

function writeUnlockedMap(map: Record<string, string>) {
  localStorage.setItem(UNLOCKED_KEY, JSON.stringify(map));
}

function todayStr(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

function computeStreakDays(sortedDates: string[]): { currentStreakDays: number; maxStreakDays: number } {
  if (sortedDates.length === 0) return { currentStreakDays: 0, maxStreakDays: 0 };
  const dateSet = new Set(sortedDates);

  // 最长连续 — 扫描
  let max = 0;
  let cur = 0;
  let prev: Date | null = null;
  for (const d of sortedDates) {
    const curDate = new Date(`${d}T00:00:00`);
    if (prev) {
      const diff = Math.round((curDate.getTime() - prev.getTime()) / 86_400_000);
      if (diff === 1) cur++;
      else cur = 1;
    } else {
      cur = 1;
    }
    if (cur > max) max = cur;
    prev = curDate;
  }

  // 当前连续 — 从今天或昨天往前
  let currentStreak = 0;
  const today = new Date(`${todayStr()}T00:00:00`);
  // 允许"今天还没 settle"的情况 —— 从昨天起算
  let cursor = new Date(today);
  if (!dateSet.has(cursorToStr(cursor))) cursor.setDate(cursor.getDate() - 1);
  while (dateSet.has(cursorToStr(cursor))) {
    currentStreak++;
    cursor.setDate(cursor.getDate() - 1);
  }

  return { currentStreakDays: currentStreak, maxStreakDays: max };
}

function cursorToStr(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

function computeConsecutiveAOrAbove(sortedSummaries: DaySummary[]): number {
  let max = 0;
  let cur = 0;
  let prev: Date | null = null;
  for (const s of sortedSummaries) {
    const rank = GRADE_RANK[s.grade] ?? 0;
    const d = new Date(`${s.settleDate}T00:00:00`);
    const isAdjacent = prev ? Math.round((d.getTime() - prev.getTime()) / 86_400_000) === 1 : true;
    if (rank >= 3 && isAdjacent) {
      cur++;
    } else if (rank >= 3) {
      cur = 1;
    } else {
      cur = 0;
    }
    if (cur > max) max = cur;
    prev = d;
  }
  return max;
}

function computeAscendingStreak(sortedSummaries: DaySummary[]): number {
  let max = 0;
  let cur = 1;
  let prev: DaySummary | null = null;
  for (const s of sortedSummaries) {
    if (prev) {
      const prevDate = new Date(`${prev.settleDate}T00:00:00`);
      const curDate = new Date(`${s.settleDate}T00:00:00`);
      const adj = Math.round((curDate.getTime() - prevDate.getTime()) / 86_400_000) === 1;
      const ascending = (GRADE_RANK[s.grade] ?? 0) > (GRADE_RANK[prev.grade] ?? 0);
      if (adj && ascending) cur++;
      else cur = 1;
    }
    if (cur > max) max = cur;
    prev = s;
  }
  return max;
}

function computeMaxMonthSCount(summaries: DaySummary[]): number {
  const monthly: Record<string, number> = {};
  for (const s of summaries) {
    if (s.grade !== "S") continue;
    const ym = s.settleDate.slice(0, 7);
    monthly[ym] = (monthly[ym] ?? 0) + 1;
  }
  return Object.values(monthly).reduce((m, v) => Math.max(m, v), 0);
}

function computeHasPerfectMonth(summaries: DaySummary[]): boolean {
  const byMonth: Record<string, Set<string>> = {};
  for (const s of summaries) {
    const ym = s.settleDate.slice(0, 7);
    if (!byMonth[ym]) byMonth[ym] = new Set();
    byMonth[ym].add(s.settleDate);
  }
  for (const [ym, set] of Object.entries(byMonth)) {
    const [y, m] = ym.split("-").map(Number);
    const days = new Date(y, m, 0).getDate();
    if (set.size >= days) return true;
  }
  return false;
}

const stats = ref<BadgeStats>({ ...EMPTY_STATS });
const unlockedAtMap = ref<Record<string, string>>(readUnlockedMap());

async function load(badgeIds: string[], evaluateUnlocked: (id: string, s: BadgeStats) => boolean) {
  try {
    const [summaries, goalsActive, goalsAll] = await Promise.all([
      invokeCmd<DaySummary[]>("list_day_summaries", { from: "2020-01-01", to: "2099-12-31" }),
      invokeCmd<Goal[]>("list_goals", { includeArchived: false }),
      invokeCmd<Goal[]>("list_goals", { includeArchived: true }),
    ]);

    const sorted = [...summaries].sort((a, b) => a.settleDate.localeCompare(b.settleDate));
    const { currentStreakDays, maxStreakDays } = computeStreakDays(sorted.map((s) => s.settleDate));

    stats.value = {
      totalPomodoros: sorted.reduce((sum, s) => sum + (s.totalPomodoros ?? 0), 0),
      totalFocusMinutes: sorted.reduce((sum, s) => sum + (s.totalFocusMinutes ?? 0), 0),
      maxDayPomodoros: sorted.reduce((m, s) => Math.max(m, s.totalPomodoros ?? 0), 0),
      currentStreakDays,
      maxStreakDays,
      hasSGrade: sorted.some((s) => s.grade === "S"),
      hasAGrade: sorted.some((s) => (GRADE_RANK[s.grade] ?? 0) >= 3),
      maxConsecutiveAOrAbove: computeConsecutiveAOrAbove(sorted),
      maxMonthSCount: computeMaxMonthSCount(sorted),
      maxAscendingStreak: computeAscendingStreak(sorted),
      hasPerfectMonth: computeHasPerfectMonth(sorted),
      activeGoalCount: goalsActive.filter((g) => g.status === "active").length,
      completedGoalCount: goalsAll.filter((g) => g.status === "completed").length,
      totalGoalCount: goalsAll.length,
      loaded: true,
    };
  } catch (e) {
    // 边界错误容忍:后端命令失败时保持 empty stats,不阻塞 UI
    console.error("[badge-engine] load failed", e);
    stats.value = { ...EMPTY_STATS, loaded: true };
  }

  // 首次观察到解锁 → 记今天为 unlockedAt
  const map = { ...unlockedAtMap.value };
  const today = todayStr();
  let changed = false;
  for (const id of badgeIds) {
    if (evaluateUnlocked(id, stats.value) && !map[id]) {
      map[id] = today;
      changed = true;
    }
  }
  if (changed) {
    writeUnlockedMap(map);
    unlockedAtMap.value = map;
  }
}

export function useBadgeEngine() {
  return { stats, unlockedAtMap, load };
}

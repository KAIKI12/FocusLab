/**
 * useMicroReviewScenario · 决策树测试。
 *
 * 覆盖 5 个分支:
 *   1. 无 task → null
 *   2. todayCount ≥ CAP → null
 *   3. milestone_id → milestone(优先级最高)
 *   4. quadrant=important_urgent → q1
 *   5. |偏差| > 30% → deviation;≤ 30% → null;无 estimate → null
 *
 * 另外覆盖 readTodayCount / incrementTodayCount 的 localStorage 读写。
 */

import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  MICRO_REVIEW_COUNT_PREFIX,
  MICRO_REVIEW_DAILY_CAP,
  incrementTodayCount,
  readTodayCount,
  resolveReviewScenario,
} from "@/composables/useMicroReviewScenario";
import type { Task } from "@/types";

function makeTask(overrides: Partial<Task> = {}): Task {
  return {
    id: "t1",
    name: "Task 1",
    description: null,
    quadrant: "important_not_urgent",
    status: "pending",
    estimated_minutes: null,
    due_date: null,
    is_background: false,
    milestone_id: null,
    shelved_at: null,
    created_at: "2026-04-21T00:00:00Z",
    updated_at: "2026-04-21T00:00:00Z",
    completed_at: null,
    ...overrides,
  };
}

describe("resolveReviewScenario", () => {
  it("returns null when task is null", () => {
    expect(resolveReviewScenario({ task: null, actualMinutes: 0, todayCount: 0 })).toBeNull();
  });

  it("returns null when todayCount has hit daily cap", () => {
    const task = makeTask({ milestone_id: "m1" });
    expect(
      resolveReviewScenario({ task, actualMinutes: 25, todayCount: MICRO_REVIEW_DAILY_CAP }),
    ).toBeNull();
  });

  it("milestone_id wins over Q1 and deviation", () => {
    const task = makeTask({
      milestone_id: "m1",
      quadrant: "important_urgent",
      estimated_minutes: 10,
    });
    expect(
      resolveReviewScenario({ task, actualMinutes: 60, todayCount: 0 }),
    ).toBe("milestone");
  });

  it("Q1 wins over deviation when no milestone", () => {
    const task = makeTask({
      quadrant: "important_urgent",
      estimated_minutes: 10,
    });
    expect(
      resolveReviewScenario({ task, actualMinutes: 60, todayCount: 0 }),
    ).toBe("q1");
  });

  it("deviation scenario when |dev| > 30%", () => {
    const task = makeTask({ estimated_minutes: 20 });
    // actual 30m,偏差 +50% > 30%
    expect(
      resolveReviewScenario({ task, actualMinutes: 30, todayCount: 0 }),
    ).toBe("deviation");
    // actual 10m,偏差 -50% > 30%
    expect(
      resolveReviewScenario({ task, actualMinutes: 10, todayCount: 0 }),
    ).toBe("deviation");
  });

  it("silent (null) when |dev| <= 30%", () => {
    const task = makeTask({ estimated_minutes: 20 });
    // 25m,偏差 +25% ≤ 30%
    expect(
      resolveReviewScenario({ task, actualMinutes: 25, todayCount: 0 }),
    ).toBeNull();
    // 15m,偏差 -25% ≤ 30%
    expect(
      resolveReviewScenario({ task, actualMinutes: 15, todayCount: 0 }),
    ).toBeNull();
  });

  it("silent when no estimate and no priority markers", () => {
    const task = makeTask(); // 普通 Q2 任务,无预估
    expect(
      resolveReviewScenario({ task, actualMinutes: 300, todayCount: 0 }),
    ).toBeNull();
  });

  it("estimate=0 treated as absent (silent)", () => {
    const task = makeTask({ estimated_minutes: 0 });
    expect(
      resolveReviewScenario({ task, actualMinutes: 100, todayCount: 0 }),
    ).toBeNull();
  });
});

describe("readTodayCount / incrementTodayCount", () => {
  beforeEach(() => {
    // node env 下 vitest 暂无 localStorage,用 Map 代理
    const map = new Map<string, string>();
    vi.stubGlobal("localStorage", {
      getItem: (k: string) => map.get(k) ?? null,
      setItem: (k: string, v: string) => { map.set(k, v); },
      removeItem: (k: string) => { map.delete(k); },
      clear: () => { map.clear(); },
      key: () => null,
      length: 0,
    });
  });

  it("readTodayCount defaults to 0 when key missing", () => {
    expect(readTodayCount("2026-04-21")).toBe(0);
  });

  it("increment then read roundtrip", () => {
    incrementTodayCount("2026-04-21");
    incrementTodayCount("2026-04-21");
    expect(readTodayCount("2026-04-21")).toBe(2);
    // 确认 key 带前缀
    expect(localStorage.getItem(MICRO_REVIEW_COUNT_PREFIX + "2026-04-21")).toBe("2");
  });

  it("each day keys independently", () => {
    incrementTodayCount("2026-04-20");
    incrementTodayCount("2026-04-21");
    expect(readTodayCount("2026-04-20")).toBe(1);
    expect(readTodayCount("2026-04-21")).toBe(1);
  });
});

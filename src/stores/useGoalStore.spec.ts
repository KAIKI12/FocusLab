/**
 * useGoalStore · v2 扩展动作测试。
 *
 * 覆盖本轮新增:
 *   - setMilestoneTargetDate 更新本地 milestone + 调对应 command
 *   - loadNotes / addNote / removeNote 的状态同步
 *   - loadWeeklyInvest 成功 / 失败路径
 *   - selectGoal 并发拉 milestones + weeklyInvest 且清空 notes 缓存
 */

import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useGoalStore } from "@/stores/useGoalStore";
import type { Milestone, MilestoneNote, WeeklyInvest } from "@/types";

function mkMilestone(overrides: Partial<Milestone> = {}): Milestone {
  return {
    id: "m1",
    goal_id: "g1",
    name: "MS",
    description: null,
    status: "pending",
    sort_order: 0,
    created_at: "2026-04-21T00:00:00Z",
    updated_at: "2026-04-21T00:00:00Z",
    completed_at: null,
    target_date: null,
    ...overrides,
  };
}

function mkNote(overrides: Partial<MilestoneNote> = {}): MilestoneNote {
  return {
    id: "n" + Math.random().toString(36).slice(2, 6),
    milestone_id: "m1",
    text: "note",
    created_at: "2026-04-21T10:00:00Z",
    ...overrides,
  };
}

describe("useGoalStore · setMilestoneTargetDate", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("calls command with camelCased input and patches local milestone", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    const store = useGoalStore();
    store.milestones = [mkMilestone({ id: "m1", target_date: null })];

    await store.setMilestoneTargetDate("m1", "2026-04-30");

    expect(invoke).toHaveBeenCalledWith("set_milestone_target_date", {
      input: { milestoneId: "m1", targetDate: "2026-04-30" },
    });
    expect(store.milestones[0].target_date).toBe("2026-04-30");
  });

  it("clearing with null persists null to local state", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    const store = useGoalStore();
    store.milestones = [mkMilestone({ id: "m1", target_date: "2026-01-01" })];

    await store.setMilestoneTargetDate("m1", null);

    expect(store.milestones[0].target_date).toBeNull();
  });
});

describe("useGoalStore · notes CRUD", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("loadNotes caches by milestoneId", async () => {
    const notes = [mkNote({ id: "n1" }), mkNote({ id: "n2" })];
    vi.mocked(invoke).mockResolvedValueOnce(notes);

    const store = useGoalStore();
    await store.loadNotes("m1");

    expect(invoke).toHaveBeenCalledWith("list_milestone_notes", { milestoneId: "m1" });
    expect(store.notesByMilestone["m1"]).toEqual(notes);
  });

  it("addNote prepends the created note (newest first)", async () => {
    const existing = mkNote({ id: "n-old", text: "old" });
    const store = useGoalStore();
    store.notesByMilestone["m1"] = [existing];

    const created = mkNote({ id: "n-new", text: "fresh" });
    vi.mocked(invoke).mockResolvedValueOnce(created);

    const result = await store.addNote("m1", "fresh");

    expect(invoke).toHaveBeenCalledWith("add_milestone_note", {
      input: { milestoneId: "m1", text: "fresh" },
    });
    expect(result).toEqual(created);
    expect(store.notesByMilestone["m1"]).toEqual([created, existing]);
  });

  it("removeNote filters out the deleted note without touching peers", async () => {
    const a = mkNote({ id: "a" });
    const b = mkNote({ id: "b" });
    const store = useGoalStore();
    store.notesByMilestone["m1"] = [a, b];

    vi.mocked(invoke).mockResolvedValueOnce(undefined);
    await store.removeNote("m1", "a");

    expect(invoke).toHaveBeenCalledWith("delete_milestone_note", { id: "a" });
    expect(store.notesByMilestone["m1"]).toEqual([b]);
  });
});

describe("useGoalStore · loadWeeklyInvest", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("stores successful payload", async () => {
    const payload: WeeklyInvest = {
      buckets: Array.from({ length: 7 }, (_, i) => ({ weekday: i, minutes: i * 10 })),
      totalMinutes: 210,
      todayMinutes: 30,
    };
    vi.mocked(invoke).mockResolvedValueOnce(payload);

    const store = useGoalStore();
    await store.loadWeeklyInvest("g1");

    expect(store.weeklyInvest).toEqual(payload);
  });

  it("null out on command failure + does not throw", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("backend down"));
    const consoleErr = vi.spyOn(console, "error").mockImplementation(() => {});

    const store = useGoalStore();
    // 预置一个旧值以验证失败后被清掉
    store.weeklyInvest = {
      buckets: [],
      totalMinutes: 99,
      todayMinutes: 0,
    };

    await expect(store.loadWeeklyInvest("g1")).resolves.toBeUndefined();
    expect(store.weeklyInvest).toBeNull();
    consoleErr.mockRestore();
  });
});

describe("useGoalStore · selectGoal", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("concurrently loads milestones + weekly invest and clears note cache", async () => {
    const milestones = [mkMilestone({ id: "m1" })];
    const invest: WeeklyInvest = { buckets: [], totalMinutes: 0, todayMinutes: 0 };

    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_milestones") return milestones as unknown;
      if (cmd === "get_goal_weekly_invest") return invest as unknown;
      throw new Error(`unexpected cmd: ${cmd}`);
    });

    const store = useGoalStore();
    // 预置一个 stale note 缓存
    store.notesByMilestone["old"] = [mkNote()];

    await store.selectGoal("g1");

    expect(store.selectedGoalId).toBe("g1");
    expect(store.milestones).toEqual(milestones);
    expect(store.weeklyInvest).toEqual(invest);
    expect(store.notesByMilestone).toEqual({});
  });
});

/**
 * useMilestoneSubtasks · 派生逻辑测试。
 *
 * 覆盖:
 *   - byMilestone 按 milestone_id 分桶 + 排序(in_progress > pending > completed)
 *   - subtasksOf / progressOf 的简单 get
 *   - todayActiveOf 优先当前 timer task,回落 in_progress
 */

import { beforeEach, describe, expect, it } from "vitest";

import { useMilestoneSubtasks } from "@/composables/useMilestoneSubtasks";
import { useTaskStore } from "@/stores/useTaskStore";
import { useTimerStore } from "@/stores/useTimerStore";
import type { Task, TimerSnapshot } from "@/types";

function makeTask(overrides: Partial<Task> = {}): Task {
  return {
    id: overrides.id ?? "t-" + Math.random().toString(36).slice(2, 8),
    name: "Task",
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

function makeSnapshot(overrides: Partial<TimerSnapshot> = {}): TimerSnapshot {
  return {
    status: "running",
    taskId: null,
    sessionId: null,
    mode: "pomodoro",
    preset: "classic_25",
    elapsedSeconds: 0,
    plannedSeconds: 1500,
    pomodoroCount: 0,
    isBreak: false,
    ...overrides,
  };
}

describe("useMilestoneSubtasks · byMilestone / subtasksOf / progressOf", () => {
  let tasks: ReturnType<typeof useTaskStore>;

  beforeEach(() => {
    tasks = useTaskStore();
    tasks.tasks = [
      makeTask({ id: "a", name: "A", milestone_id: "m1", status: "completed" }),
      makeTask({ id: "b", name: "B", milestone_id: "m1", status: "in_progress" }),
      makeTask({ id: "c", name: "C", milestone_id: "m1", status: "pending" }),
      makeTask({ id: "d", name: "D", milestone_id: "m2", status: "pending" }),
      makeTask({ id: "e", name: "E", milestone_id: null, status: "pending" }),
    ];
  });

  it("tasks without milestone_id are excluded", () => {
    const { byMilestone } = useMilestoneSubtasks();
    expect(byMilestone.value["m1"]).toHaveLength(3);
    expect(byMilestone.value["m2"]).toHaveLength(1);
    // 无 milestone 的 e 不出现在任何 bucket
    const allIds = Object.values(byMilestone.value).flat().map((t) => t.id);
    expect(allIds).not.toContain("e");
  });

  it("subtasksOf sorts in_progress > pending > completed", () => {
    const { subtasksOf } = useMilestoneSubtasks();
    const list = subtasksOf("m1");
    expect(list.map((t) => t.id)).toEqual(["b", "c", "a"]);
  });

  it("progressOf returns done / total", () => {
    const { progressOf } = useMilestoneSubtasks();
    expect(progressOf("m1")).toEqual({ done: 1, total: 3 });
    expect(progressOf("m2")).toEqual({ done: 0, total: 1 });
    expect(progressOf("unknown")).toEqual({ done: 0, total: 0 });
  });

  it("subtasksOf returns empty array for unknown milestone", () => {
    const { subtasksOf } = useMilestoneSubtasks();
    expect(subtasksOf("unknown")).toEqual([]);
  });
});

describe("useMilestoneSubtasks · todayActiveOf", () => {
  let tasks: ReturnType<typeof useTaskStore>;
  let timer: ReturnType<typeof useTimerStore>;

  beforeEach(() => {
    tasks = useTaskStore();
    timer = useTimerStore();
  });

  it("returns null for empty milestone", () => {
    tasks.tasks = [];
    const { todayActiveOf } = useMilestoneSubtasks();
    expect(todayActiveOf("m1")).toBeNull();
  });

  it("prefers the task currently being focused", () => {
    tasks.tasks = [
      makeTask({ id: "b", milestone_id: "m1", status: "in_progress" }),
      makeTask({ id: "c", milestone_id: "m1", status: "pending" }),
    ];
    timer.snapshot = makeSnapshot({ taskId: "c", elapsedSeconds: 900 });

    const { todayActiveOf } = useMilestoneSubtasks();
    const active = todayActiveOf("m1");

    expect(active).not.toBeNull();
    expect(active!.task.id).toBe("c");
    expect(active!.isFocusing).toBe(true);
    expect(active!.focusingSeconds).toBe(900);
  });

  it("falls back to an in_progress task when timer is not in this milestone", () => {
    tasks.tasks = [
      makeTask({ id: "b", milestone_id: "m1", status: "in_progress" }),
      makeTask({ id: "c", milestone_id: "m1", status: "pending" }),
    ];
    // timer 在跑别的里程碑下的任务
    timer.snapshot = makeSnapshot({ taskId: "other-task", elapsedSeconds: 100 });

    const { todayActiveOf } = useMilestoneSubtasks();
    const active = todayActiveOf("m1");

    expect(active).not.toBeNull();
    expect(active!.task.id).toBe("b");
    expect(active!.isFocusing).toBe(false);
    expect(active!.focusingSeconds).toBe(0);
  });

  it("returns null when no in_progress and no focused task in this milestone", () => {
    tasks.tasks = [
      makeTask({ id: "a", milestone_id: "m1", status: "completed" }),
      makeTask({ id: "c", milestone_id: "m1", status: "pending" }),
    ];
    timer.snapshot = null;

    const { todayActiveOf } = useMilestoneSubtasks();
    expect(todayActiveOf("m1")).toBeNull();
  });
});

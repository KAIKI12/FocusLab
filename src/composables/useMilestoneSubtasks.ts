/**
 * useMilestoneSubtasks · 里程碑下子任务派生数据。
 *
 * 对齐 prototype/goals/milestones.html:244 子任务列表 + :535 今日关联任务。
 * 基于 useTaskStore 派生,不发新请求。
 *
 * 设计要点:
 *   - 子任务 = 该里程碑下的 tasks(complete 排后,in_progress/pending 排前)
 *   - 今日活跃 = 当前 timer 正在跑且属于该里程碑,或该里程碑下任一 task.status === 'in_progress'
 *   - 深度时长暂不从 sessions 聚合(避免新 command),由 timer snapshot 或 elapsed 体现
 */

import { computed } from "vue";

import { useTaskStore } from "@/stores/useTaskStore";
import { useTimerStore } from "@/stores/useTimerStore";
import type { Task } from "@/types";

export interface TodayActive {
  task: Task;
  /** 是否当前 Pomodoro 正在跑这个 task */
  isFocusing: boolean;
  /** 当前 session 已专注秒数(仅当 isFocusing 时非 0) */
  focusingSeconds: number;
}

export function useMilestoneSubtasks() {
  const tasks = useTaskStore();
  const timer = useTimerStore();

  const byMilestone = computed<Record<string, Task[]>>(() => {
    const result: Record<string, Task[]> = {};
    for (const t of tasks.tasks) {
      if (!t.milestone_id) continue;
      if (!result[t.milestone_id]) result[t.milestone_id] = [];
      result[t.milestone_id].push(t);
    }
    // 排序:in_progress > pending > completed,同状态按 name
    const order: Record<string, number> = { in_progress: 0, pending: 1, completed: 2 };
    for (const id of Object.keys(result)) {
      result[id].sort((a, b) => {
        const oa = order[a.status] ?? 1;
        const ob = order[b.status] ?? 1;
        return oa !== ob ? oa - ob : a.name.localeCompare(b.name);
      });
    }
    return result;
  });

  function subtasksOf(milestoneId: string): Task[] {
    return byMilestone.value[milestoneId] ?? [];
  }

  /** 当前里程碑下"今天正在推进"的 task:优先当前 timer 正在跑的,回落到任意 in_progress */
  function todayActiveOf(milestoneId: string): TodayActive | null {
    const list = byMilestone.value[milestoneId];
    if (!list?.length) return null;

    const focusingId = timer.snapshot?.taskId ?? null;
    const elapsed = timer.snapshot?.elapsedSeconds ?? 0;

    // 优先:当前 timer 正在跑且属于该里程碑
    if (focusingId) {
      const focusingTask = list.find((t) => t.id === focusingId);
      if (focusingTask) {
        return { task: focusingTask, isFocusing: true, focusingSeconds: elapsed };
      }
    }

    // 次选:该里程碑下最近一个 in_progress 的 task
    const inProgress = list.find((t) => t.status === "in_progress");
    if (inProgress) {
      return { task: inProgress, isFocusing: false, focusingSeconds: 0 };
    }

    return null;
  }

  /** 进度统计:(done, total) */
  function progressOf(milestoneId: string): { done: number; total: number } {
    const list = byMilestone.value[milestoneId] ?? [];
    const done = list.filter((t) => t.status === "completed").length;
    return { done, total: list.length };
  }

  return { byMilestone, subtasksOf, todayActiveOf, progressOf };
}

/**
 * useAssignmentStore · 今日计划 (daily_task_assignments) store。
 *
 * 本轮提供列表/添加/状态切换/移除的基本操作。
 */

import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import type {
  AssignmentWithTask,
  CreateAssignmentInput,
  DailyTaskAssignment,
  DayStatus,
} from "@/types";

export const useAssignmentStore = defineStore("assignment", () => {
  const assignments = ref<AssignmentWithTask[]>([]);
  const loading = ref(false);
  /** null 表示"当前逻辑日",由后端解析 */
  const planDate = ref<string | null>(null);

  async function load(date?: string | null) {
    loading.value = true;
    if (date !== undefined) planDate.value = date;
    try {
      assignments.value = await invokeCmd<AssignmentWithTask[]>(
        "list_assignments",
        { planDate: planDate.value },
      );
    } finally {
      loading.value = false;
    }
  }

  async function create(input: CreateAssignmentInput) {
    const created = await invokeCmd<DailyTaskAssignment>("create_assignment", {
      input,
    });
    // 刷新联表视图(避免本地拼 task 字段出错)
    await load();
    return created;
  }

  async function setStatus(id: string, dayStatus: DayStatus) {
    await invokeCmd<void>("update_assignment_status", { id, dayStatus });
    const target = assignments.value.find((a) => a.id === id);
    if (target) {
      target.dayStatus = dayStatus;
      if (dayStatus === "completed") {
        target.completedAt = new Date().toISOString();
      }
    }
  }

  async function remove(id: string) {
    await invokeCmd<void>("remove_assignment", { id });
    assignments.value = assignments.value.filter((a) => a.id !== id);
  }

  return { assignments, loading, planDate, load, create, setStatus, remove };
});

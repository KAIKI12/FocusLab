/**
 * useTaskStore · Task store — list / create / complete / update / remove。
 */

import { defineStore } from "pinia";
import { computed, ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import type { CreateTaskInput, Task, UpdateTaskInput } from "@/types";

export const useTaskStore = defineStore("task", () => {
  const tasks = ref<Task[]>([]);
  const loading = ref(false);

  async function load() {
    loading.value = true;
    try {
      tasks.value = await invokeCmd<Task[]>("list_tasks", {
        statusFilter: null,
      });
    } finally {
      loading.value = false;
    }
  }

  async function create(input: CreateTaskInput) {
    const created = await invokeCmd<Task>("create_task", { input });
    tasks.value.unshift(created);
    return created;
  }

  async function complete(id: string) {
    await invokeCmd<void>("complete_task", { id });
    tasks.value = tasks.value.filter((t) => t.id !== id);
  }

  async function update(input: UpdateTaskInput) {
    const updated = await invokeCmd<Task>("update_task", { input });
    const idx = tasks.value.findIndex((t) => t.id === input.id);
    if (idx >= 0) {
      tasks.value[idx] = updated;
    }
    return updated;
  }

  /** 三态轮转:pending → in_progress → completed → pending */
  const STATUS_CYCLE = ["pending", "in_progress", "completed"] as const;
  type CycleStatus = (typeof STATUS_CYCLE)[number];

  async function cycleStatus(id: string, current: string): Promise<CycleStatus> {
    const idx = STATUS_CYCLE.indexOf(current as CycleStatus);
    const next = STATUS_CYCLE[(idx + 1) % STATUS_CYCLE.length];
    await update({ id, status: next });
    return next;
  }

  async function remove(id: string) {
    await invokeCmd<void>("delete_task", { id });
    tasks.value = tasks.value.filter((t) => t.id !== id);
  }

  /** 搁置区物理删除（永久移除记录） */
  async function removePermanently(id: string) {
    await invokeCmd<void>("hard_delete_task", { id });
    tasks.value = tasks.value.filter((t) => t.id !== id);
  }

  /** 按四象限分组 */
  const tasksByQuadrant = computed(() => {
    const groups: Record<string, Task[]> = {
      important_urgent: [],
      important_not_urgent: [],
      not_important_urgent: [],
      not_important_not_urgent: [],
    };
    for (const t of tasks.value) {
      const q = t.quadrant in groups ? t.quadrant : "important_not_urgent";
      groups[q].push(t);
    }
    return groups;
  });

  return { tasks, loading, load, create, complete, update, cycleStatus, remove, removePermanently, tasksByQuadrant };
});

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

  async function remove(id: string) {
    await invokeCmd<void>("delete_task", { id });
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

  return { tasks, loading, load, create, complete, update, remove, tasksByQuadrant };
});

/**
 * useTaskStore · Week 1a 最小 task store。
 * 只覆盖 list / create / complete 三个动作 — 与后端 CRUD 对齐。
 */

import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import type { CreateTaskInput, Task } from "@/types";

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

  return { tasks, loading, load, create, complete };
});

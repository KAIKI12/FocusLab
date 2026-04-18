/**
 * useTimerStateStore · 计时器状态持久化 — Week 1b 只负责读写 timer_state 表。
 *
 * Week 2 再加真正的计时状态机(每秒 tick、每 30s 写盘)。
 */

import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import type { TimerState, TimerStatePatch } from "@/types";

export const useTimerStateStore = defineStore("timerState", () => {
  const state = ref<TimerState | null>(null);
  const loading = ref(false);

  async function load() {
    loading.value = true;
    try {
      state.value = await invokeCmd<TimerState>("get_timer_state");
    } finally {
      loading.value = false;
    }
  }

  async function update(patch: TimerStatePatch) {
    state.value = await invokeCmd<TimerState>("update_timer_state", { patch });
  }

  async function reset() {
    await invokeCmd<void>("reset_timer_state");
    await load();
  }

  return { state, loading, load, update, reset };
});

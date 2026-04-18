/**
 * useTimerStore · 番茄钟快照 store(Week 2a)。
 *
 * 后端 TimerService 是权威源,前端只订阅两个事件:
 *   - `timer:tick` 每秒一次的 snapshot 更新(elapsed + 1)
 *   - `timer:state_changed` 状态迁移(start / pause / resume / break / idle)
 *
 * 前端不独立 tick;store 持有最新 snapshot 即可驱动 UI。
 */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { computed, ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import type { PomodoroPreset, TimerSnapshot } from "@/types";

/** 从后端 timer_state 行转成 TimerSnapshot — 启动时初始化用 */
function stateRowToSnapshot(row: {
  task_id: string | null;
  session_id: string | null;
  start_time: string | null;
  elapsed_seconds: number;
  planned_seconds: number | null;
  mode: "pomodoro" | "free" | null;
  pomodoro_preset: string | null;
  status: string;
  pomodoro_count: number;
  is_break: boolean;
  break_remaining: number | null;
}): TimerSnapshot {
  return {
    status: row.status as TimerSnapshot["status"],
    taskId: row.task_id,
    sessionId: row.session_id,
    mode: row.mode,
    preset: (row.pomodoro_preset as PomodoroPreset | null) ?? null,
    elapsedSeconds: row.elapsed_seconds,
    plannedSeconds: row.is_break
      ? row.break_remaining ?? 0
      : row.planned_seconds ?? 0,
    pomodoroCount: row.pomodoro_count,
    isBreak: row.is_break,
  };
}

// 模块级事件订阅 — 保证只挂一次
let listenerGuard: Promise<UnlistenFn[]> | null = null;

export const useTimerStore = defineStore("timer", () => {
  const snapshot = ref<TimerSnapshot | null>(null);

  const isIdle = computed(
    () => !snapshot.value || snapshot.value.status === "idle",
  );
  const isRunning = computed(() => snapshot.value?.status === "running");
  const isPaused = computed(() => snapshot.value?.status === "paused");
  const isBreak = computed(() => snapshot.value?.status === "break");
  const isBreakEnded = computed(() => snapshot.value?.status === "break_ended");
  const isFreeMode = computed(() => snapshot.value?.mode === "free");

  /** 剩余秒数(倒计时数字用);自由模式下返回 elapsed(正计时) */
  const remainingSeconds = computed(() => {
    if (!snapshot.value) return 0;
    if (snapshot.value.mode === "free") return snapshot.value.elapsedSeconds;
    return Math.max(
      0,
      snapshot.value.plannedSeconds - snapshot.value.elapsedSeconds,
    );
  });

  /** 进度百分比(圆环 stroke-dashoffset 用) */
  const progress = computed(() => {
    if (!snapshot.value || snapshot.value.plannedSeconds <= 0) return 0;
    return Math.min(
      1,
      snapshot.value.elapsedSeconds / snapshot.value.plannedSeconds,
    );
  });

  /** 当前选中的 preset(idle 时由 PresetSwitcher 设置) */
  const selectedPreset = ref<PomodoroPreset | "free">("classic_25");

  async function ensureListeners() {
    if (listenerGuard) return listenerGuard;
    listenerGuard = (async () => {
      const handler = (ev: { payload: TimerSnapshot }) => {
        snapshot.value = ev.payload;
      };
      return Promise.all([
        listen<TimerSnapshot>("timer:tick", handler),
        listen<TimerSnapshot>("timer:state_changed", handler),
      ]);
    })();
    return listenerGuard;
  }

  /** 初始化 — 订阅事件 + 拉一次当前 timer_state 做冷启动快照 */
  async function init() {
    await ensureListeners();
    try {
      const row = await invokeCmd<
        Parameters<typeof stateRowToSnapshot>[0]
      >("get_timer_state");
      snapshot.value = stateRowToSnapshot(row);
    } catch (e) {
      console.error("[timer] init get_timer_state failed", e);
    }
  }

  // ---------- actions ----------

  async function startPomodoro(taskId: string, preset: PomodoroPreset = "classic_25") {
    snapshot.value = await invokeCmd<TimerSnapshot>("start_pomodoro", {
      taskId,
      preset,
    });
  }

  async function pause() {
    snapshot.value = await invokeCmd<TimerSnapshot>("pause_timer");
  }

  async function resume() {
    snapshot.value = await invokeCmd<TimerSnapshot>("resume_timer");
  }

  async function abandon(reason?: string) {
    await invokeCmd<void>("abandon_timer", { reason: reason ?? null });
    // 等待 state_changed 事件自己把 snapshot 置 idle
  }

  async function skipBreak() {
    await invokeCmd<void>("skip_break");
  }

  // ---------- Week 2b: 休息三选一 ----------

  async function continueAfterBreak() {
    snapshot.value = await invokeCmd<TimerSnapshot>("continue_after_break");
  }

  async function switchTaskAfterBreak(taskId: string) {
    snapshot.value = await invokeCmd<TimerSnapshot>("switch_task_after_break", { taskId });
  }

  async function extendBreak(extraSeconds: number = 300) {
    snapshot.value = await invokeCmd<TimerSnapshot>("extend_break", { extraSeconds });
  }

  // ---------- Week 2b: 自由模式 ----------

  async function startFree(taskId: string) {
    snapshot.value = await invokeCmd<TimerSnapshot>("start_free", { taskId });
  }

  async function completeFree() {
    await invokeCmd<void>("complete_free");
  }

  return {
    snapshot,
    isIdle,
    isRunning,
    isPaused,
    isBreak,
    isBreakEnded,
    isFreeMode,
    remainingSeconds,
    progress,
    selectedPreset,
    init,
    startPomodoro,
    pause,
    resume,
    abandon,
    skipBreak,
    continueAfterBreak,
    switchTaskAfterBreak,
    extendBreak,
    startFree,
    completeFree,
  };
});

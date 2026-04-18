<script setup lang="ts">
/**
 * TimerCard · 当前进行中的番茄钟卡片。
 *
 * 组合:任务名 + PomodoroRing + TimerControls。
 * 由父层在 snapshot.status !== 'idle' 时渲染。
 *
 * 任务名从 useTaskStore 里根据 taskId 查;查不到(跨逻辑日 / 已完成场景)
 * 显示兜底文案,避免 UI 空白。
 */

import { computed } from "vue";

import PomodoroRing from "@/components/timer/PomodoroRing.vue";
import TimerControls from "@/components/timer/TimerControls.vue";
import { useTaskStore } from "@/stores/useTaskStore";
import { useTimerStore } from "@/stores/useTimerStore";

const timer = useTimerStore();
const tasks = useTaskStore();

const taskName = computed(() => {
  const id = timer.snapshot?.taskId;
  if (!id) return "(未绑定任务)";
  return tasks.tasks.find((t) => t.id === id)?.name ?? "(任务已归档)";
});
</script>

<template>
  <section v-if="timer.snapshot && !timer.isIdle" class="fl-timer-card">
    <header class="fl-timer-head">
      <span class="fl-timer-task">{{ taskName }}</span>
    </header>
    <PomodoroRing :snapshot="timer.snapshot" />
    <TimerControls />
  </section>
</template>

<style scoped>
.fl-timer-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sp-4);
  padding: var(--sp-5) var(--sp-6);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-card);
}

.fl-timer-head {
  text-align: center;
}

.fl-timer-task {
  font-size: var(--fs-16);
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
}
</style>

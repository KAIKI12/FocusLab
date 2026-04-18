<script setup lang="ts">
/**
 * TimerCard · 当前进行中的番茄钟卡片。
 *
 * 组合:任务名 + PomodoroRing + TimerControls + InterruptionDialog。
 * 由父层在 snapshot.status !== 'idle' 时渲染。
 *
 * 中断跟踪:暂停时 TimerControls 弹 InterruptionDialog,
 * dialog 记录后把 interruptionId 存下;resume 时自动调 end_interruption。
 */

import { computed, ref, watch } from "vue";
import { Minimize2 } from "lucide-vue-next";

import InterruptionDialog from "@/components/timer/InterruptionDialog.vue";
import PomodoroRing from "@/components/timer/PomodoroRing.vue";
import TimerControls from "@/components/timer/TimerControls.vue";
import { useBubble } from "@/composables/useBubble";
import { invokeCmd } from "@/composables/useTauriInvoke";
import { useTaskStore } from "@/stores/useTaskStore";
import { useTimerStore } from "@/stores/useTimerStore";

const timer = useTimerStore();
const tasks = useTaskStore();
const { open: openBubble } = useBubble();

const taskName = computed(() => {
  const id = timer.snapshot?.taskId;
  if (!id) return "(未绑定任务)";
  return tasks.tasks.find((t) => t.id === id)?.name ?? "(任务已归档)";
});

// 当前活跃中断 ID — resume 时自动 end
const activeInterruptionId = ref<string | null>(null);

function onInterruptionRecorded(id: string) {
  activeInterruptionId.value = id;
}

// 监听 status 从 paused → running: 自动结束中断
watch(
  () => timer.snapshot?.status,
  (newStatus, oldStatus) => {
    if (oldStatus === "paused" && newStatus === "running" && activeInterruptionId.value) {
      invokeCmd<void>("end_interruption", { id: activeInterruptionId.value }).catch((e) =>
        console.error("[interruption] end failed", e),
      );
      activeInterruptionId.value = null;
    }
  },
);
</script>

<template>
  <section v-if="timer.snapshot && !timer.isIdle" class="fl-timer-card">
    <header class="fl-timer-head">
      <span class="fl-timer-task">{{ taskName }}</span>
      <button class="fl-bubble-btn" type="button" title="悬浮球" @click="openBubble">
        <Minimize2 :size="14" />
      </button>
    </header>
    <PomodoroRing :snapshot="timer.snapshot" />
    <TimerControls />
    <InterruptionDialog
      :session-id="timer.snapshot?.sessionId ?? null"
      @recorded="onInterruptionRecorded"
    />
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
  display: flex;
  align-items: center;
  gap: var(--sp-2);
}

.fl-timer-task {
  flex: 1;
  font-size: var(--fs-16);
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
}

.fl-bubble-btn {
  background: none;
  border: 1px solid var(--color-border);
  border-radius: var(--r-sm);
  padding: 4px;
  color: var(--color-text-muted);
  cursor: pointer;
  display: grid;
  place-items: center;
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-bubble-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}
</style>

<script setup lang="ts">
/**
 * BreakEndDialog · 休息结束后三选一。
 *
 * 仅在 status === 'break_ended' 时渲染。
 * 三个选项:
 *   1. 继续同一任务
 *   2. 切换到其他任务(下拉选择)
 *   3. 延长休息 5 分钟
 */

import { Clock, Play, RefreshCw } from "lucide-vue-next";
import { computed, ref } from "vue";

import { useAssignmentStore } from "@/stores/useAssignmentStore";
import { useTaskStore } from "@/stores/useTaskStore";
import { useTimerStore } from "@/stores/useTimerStore";

const timer = useTimerStore();
const tasks = useTaskStore();
const assignments = useAssignmentStore();

const switchTaskId = ref("");

const currentTaskName = computed(() => {
  const id = timer.snapshot?.taskId;
  if (!id) return "(未知)";
  return tasks.tasks.find((t) => t.id === id)?.name ?? "(任务已归档)";
});

/** 可切换的候选任务(排除当前任务) */
const candidates = computed(() => {
  const currentId = timer.snapshot?.taskId;
  // 优先用今日计划里的任务,否则从任务池取
  const fromAssignments = assignments.assignments
    .filter((a) => a.taskId !== currentId && a.dayStatus !== "completed")
    .map((a) => ({ id: a.taskId, name: a.taskName }));
  if (fromAssignments.length > 0) return fromAssignments;
  return tasks.tasks
    .filter((t) => t.id !== currentId)
    .map((t) => ({ id: t.id, name: t.name }));
});

async function onContinue() {
  try {
    await timer.continueAfterBreak();
  } catch (e) {
    console.error("[break-end] continue failed", e);
  }
}

async function onSwitch() {
  if (!switchTaskId.value) return;
  try {
    await timer.switchTaskAfterBreak(switchTaskId.value);
  } catch (e) {
    console.error("[break-end] switch failed", e);
  }
}

async function onExtend() {
  try {
    await timer.extendBreak(300); // 5 分钟
  } catch (e) {
    console.error("[break-end] extend failed", e);
  }
}
</script>

<template>
  <Transition name="fl-fade">
    <div
      v-if="timer.isBreakEnded"
      class="fl-be-mask"
      role="presentation"
    >
      <div class="fl-be-card" role="dialog" aria-modal="true" aria-labelledby="fl-be-title">
        <h3 id="fl-be-title" class="fl-be-head">休息结束!</h3>

        <p class="fl-be-sub">
          上一个番茄钟: {{ currentTaskName }}
          · 已完成 {{ timer.snapshot?.pomodoroCount ?? 0 }} 个
        </p>

        <div class="fl-be-options">
          <button class="fl-be-opt fl-be-primary" type="button" @click="onContinue">
            <Play :size="16" />
            <span>继续: {{ currentTaskName }}</span>
          </button>

          <div class="fl-be-switch-row">
            <select v-model="switchTaskId" class="fl-be-select">
              <option value="" disabled>选择其他任务…</option>
              <option v-for="c in candidates" :key="c.id" :value="c.id">
                {{ c.name }}
              </option>
            </select>
            <button
              class="fl-be-opt fl-be-secondary"
              type="button"
              :disabled="!switchTaskId"
              @click="onSwitch"
            >
              <RefreshCw :size="14" /> 切换
            </button>
          </div>

          <button class="fl-be-opt fl-be-ghost" type="button" @click="onExtend">
            <Clock :size="14" /> 再休息 5 分钟
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fl-be-mask {
  position: fixed;
  inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 28%, transparent);
  display: grid;
  place-items: center;
  z-index: var(--z-modal);
  padding: var(--sp-4);
}

.fl-be-card {
  width: min(420px, 100%);
  max-height: calc(100vh - 32px);
  overflow-y: auto;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-modal);
  padding: var(--sp-5) var(--sp-6);
  display: flex;
  flex-direction: column;
  gap: var(--sp-4);
}

.fl-be-head {
  margin: 0;
  font-size: var(--fs-16);
  font-weight: var(--fw-semibold);
  color: var(--color-success);
}

.fl-be-sub {
  margin: 0;
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
}

.fl-be-options {
  display: flex;
  flex-direction: column;
  gap: var(--sp-3);
}

.fl-be-opt {
  display: inline-flex;
  align-items: center;
  gap: var(--sp-2);
  padding: var(--sp-3) var(--sp-4);
  border-radius: var(--r-md);
  font-size: var(--fs-13, var(--fs-14));
  font-weight: var(--fw-medium);
  cursor: pointer;
  border: 1px solid transparent;
  transition: all var(--dur-fast) var(--ease-smooth);
}

.fl-be-primary {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
}
.fl-be-primary:hover {
  background: var(--color-primary-dark);
}
.fl-be-primary span {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.fl-be-secondary {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
  border-color: var(--color-border);
  padding: var(--sp-2) var(--sp-3);
  flex-shrink: 0;
}
.fl-be-secondary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.fl-be-ghost {
  background: transparent;
  color: var(--color-text-secondary);
  border-color: var(--color-border);
}
.fl-be-ghost:hover {
  border-color: var(--color-success);
  color: var(--color-success);
}

.fl-be-switch-row {
  display: flex;
  gap: var(--sp-2);
}

.fl-be-select {
  flex: 1;
  padding: var(--sp-2) var(--sp-3);
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-subtle);
  color: var(--color-text-primary);
  font-size: var(--fs-12);
  outline: none;
}
.fl-be-select:focus {
  border-color: var(--color-primary);
}

.fl-fade-enter-active,
.fl-fade-leave-active {
  transition: opacity var(--dur-base) var(--ease-smooth);
}
.fl-fade-enter-from,
.fl-fade-leave-to {
  opacity: 0;
}
</style>

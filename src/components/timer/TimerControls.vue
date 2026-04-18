<script setup lang="ts">
/**
 * TimerControls · 番茄钟控制按钮组。
 *
 * 行为矩阵:
 *   running → [⏸ 暂停] [✕ 放弃]
 *   paused  → [▶ 继续] [✕ 放弃]
 *   break   → [⏭ 跳过休息]
 *   idle/无 → 父层不渲染,本组件不做兜底
 *
 * 放弃前用原生 confirm(Week 2a 足够,Week 2b 做中断原因弹窗)。
 */

import { Check, Pause, Play, SkipForward, X } from "lucide-vue-next";
import { computed } from "vue";

import { useTimerStore } from "@/stores/useTimerStore";
import { useUIStore } from "@/stores/useUIStore";

const timer = useTimerStore();
const ui = useUIStore();

const status = computed(() => timer.snapshot?.status ?? "idle");
const isFree = computed(() => timer.snapshot?.mode === "free");

async function onPause() {
  try {
    await timer.pause();
    // 暂停成功后弹出中断原因选择(非阻塞,用户可跳过)
    ui.showInterruptionDialog = true;
  } catch (e) {
    console.error("[timer] pause failed", e);
  }
}

async function onResume() {
  try {
    await timer.resume();
  } catch (e) {
    console.error("[timer] resume failed", e);
  }
}

async function onAbandon() {
  if (!window.confirm("确认放弃本次番茄钟?")) return;
  try {
    await timer.abandon();
  } catch (e) {
    console.error("[timer] abandon failed", e);
  }
}

async function onCompleteFree() {
  try {
    await timer.completeFree();
  } catch (e) {
    console.error("[timer] completeFree failed", e);
  }
}

async function onSkipBreak() {
  try {
    await timer.skipBreak();
  } catch (e) {
    console.error("[timer] skipBreak failed", e);
  }
}
</script>

<template>
  <div class="fl-controls" role="toolbar" aria-label="计时控制">
    <template v-if="status === 'running'">
      <button class="fl-btn fl-btn-main" type="button" aria-label="暂停" @click="onPause">
        <Pause :size="20" />
      </button>
      <button v-if="isFree" class="fl-btn fl-btn-complete" type="button" @click="onCompleteFree">
        <Check :size="16" /> 完成
      </button>
      <button class="fl-btn fl-btn-ghost" type="button" @click="onAbandon">
        <X :size="16" /> 放弃
      </button>
    </template>

    <template v-else-if="status === 'paused'">
      <button class="fl-btn fl-btn-main" type="button" aria-label="继续" @click="onResume">
        <Play :size="20" />
      </button>
      <button class="fl-btn fl-btn-ghost" type="button" @click="onAbandon">
        <X :size="16" /> 放弃
      </button>
    </template>

    <template v-else-if="status === 'break'">
      <button class="fl-btn fl-btn-main" type="button" @click="onSkipBreak">
        <SkipForward :size="18" /> 跳过休息
      </button>
    </template>
  </div>
</template>

<style scoped>
.fl-controls {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--sp-3);
}

.fl-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--sp-1);
  border-radius: var(--r-md);
  font-size: var(--fs-13, var(--fs-14));
  font-weight: var(--fw-medium);
  cursor: pointer;
  transition:
    background var(--dur-fast) var(--ease-smooth),
    border-color var(--dur-fast) var(--ease-smooth),
    color var(--dur-fast) var(--ease-smooth);
  border: 1px solid transparent;
}

.fl-btn-main {
  width: 56px;
  height: 56px;
  padding: 0;
  border-radius: 50%;
  background: var(--color-primary);
  color: var(--color-text-on-primary);
  justify-content: center;
  box-shadow:
    0 8px 20px color-mix(in srgb, var(--color-primary) 35%, transparent);
}
.fl-btn-main:hover {
  background: var(--color-primary-dark, var(--color-primary));
}

/* break 态主按钮改用 success 色 */
.fl-controls:has(.fl-btn-main + :nth-child(0)),
.fl-controls .fl-btn-main:only-child {
  /* 兜底:break 下是唯一按钮,走普通 width */
  width: auto;
  height: auto;
  padding: var(--sp-2) var(--sp-4);
  border-radius: var(--r-md);
  background: var(--color-success);
  box-shadow:
    0 6px 16px color-mix(in srgb, var(--color-success) 30%, transparent);
}

.fl-btn-ghost {
  background: transparent;
  padding: var(--sp-2) var(--sp-4);
  border-color: var(--color-border);
  color: var(--color-text-secondary);
}
.fl-btn-ghost:hover {
  border-color: var(--color-q1, var(--color-danger, #ef4444));
  color: var(--color-q1, var(--color-danger, #ef4444));
}

.fl-btn-complete {
  background: var(--color-success);
  color: var(--color-text-on-primary);
  padding: var(--sp-2) var(--sp-4);
  box-shadow: 0 4px 12px color-mix(in srgb, var(--color-success) 30%, transparent);
}
.fl-btn-complete:hover {
  opacity: 0.9;
}
</style>

<script setup lang="ts">
/**
 * RecoveryDialog · 崩溃恢复 AskUser 档位的对话框。
 *
 * 三个动作:
 *   继续计时 — 保留 timer_state,关闭 Dialog(Week 2 才接真实恢复;
 *              本轮相当于"我知道了,先不动")
 *   结束会话 — reset_timer_state,关闭
 *   丢弃数据 — 语义同上,文案区分(Week 2 时会让 session 落到 abandoned)
 */

import { AlarmClock, Check, Trash2, X } from "lucide-vue-next";
import { computed } from "vue";

import { useRecoveryStore } from "@/stores/useRecoveryStore";
import { useTimerStateStore } from "@/stores/useTimerStateStore";

const recoveryStore = useRecoveryStore();
const timerStore = useTimerStateStore();

const gapText = computed(() => {
  const sec = recoveryStore.info?.gapSeconds ?? 0;
  if (sec < 60) return `${sec} 秒`;
  if (sec < 3600) return `${Math.round(sec / 60)} 分钟`;
  return `${(sec / 3600).toFixed(1)} 小时`;
});

const taskName = computed(
  () => recoveryStore.info?.taskName ?? "(未绑定任务)",
);

const elapsedText = computed(() => {
  const sec = recoveryStore.info?.state.elapsed_seconds ?? 0;
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  return `${m}m ${s}s`;
});

async function onResume() {
  // Week 2 接真实恢复。本轮只关闭 Dialog。
  recoveryStore.hide();
}

async function onEnd() {
  await timerStore.reset();
  recoveryStore.hide();
}

async function onDiscard() {
  // 语义等同 onEnd(Week 2 加 session.status=abandoned)
  await timerStore.reset();
  recoveryStore.hide();
}
</script>

<template>
  <Transition name="fl-fade">
    <div
      v-if="recoveryStore.visible && recoveryStore.info"
      class="fl-recovery-mask"
      role="presentation"
    >
      <div
        class="fl-recovery-card"
        role="dialog"
        aria-modal="true"
        aria-labelledby="fl-recovery-title"
      >
        <header class="fl-head">
          <div class="fl-head-icon" aria-hidden="true">
            <AlarmClock :size="20" />
          </div>
          <h2 id="fl-recovery-title">上次的计时没有正常结束</h2>
        </header>

        <div class="fl-body">
          <div class="fl-row">
            <span class="fl-label">任务</span>
            <span class="fl-value">{{ taskName }}</span>
          </div>
          <div class="fl-row">
            <span class="fl-label">已计时</span>
            <span class="fl-value">{{ elapsedText }}</span>
          </div>
          <div class="fl-row">
            <span class="fl-label">中断时长</span>
            <span class="fl-value fl-highlight">{{ gapText }}</span>
          </div>
        </div>

        <footer class="fl-actions">
          <button class="fl-btn fl-btn-primary" type="button" @click="onResume">
            <Check :size="14" />
            继续计时
          </button>
          <button class="fl-btn fl-btn-secondary" type="button" @click="onEnd">
            <X :size="14" />
            结束会话
          </button>
          <button class="fl-btn fl-btn-ghost" type="button" @click="onDiscard">
            <Trash2 :size="14" />
            丢弃数据
          </button>
        </footer>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fl-recovery-mask {
  position: fixed;
  inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 32%, transparent);
  display: grid;
  place-items: center;
  z-index: var(--z-modal);
  padding: var(--sp-4);
}

.fl-recovery-card {
  width: min(420px, 100%);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-modal);
  padding: var(--sp-5) var(--sp-6);
  display: flex;
  flex-direction: column;
  gap: var(--sp-4);
}

.fl-head {
  display: flex;
  align-items: center;
  gap: var(--sp-3);
}

.fl-head-icon {
  width: 36px;
  height: 36px;
  border-radius: var(--r-pill);
  display: grid;
  place-items: center;
  background: var(--color-warning-soft);
  color: var(--color-warning-text);
}

.fl-head h2 {
  font-size: var(--fs-16);
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
  margin: 0;
}

.fl-body {
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
  padding: var(--sp-3) var(--sp-4);
  background: var(--color-bg-subtle);
  border-radius: var(--r-md);
}

.fl-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-4);
  font-size: var(--fs-12);
}

.fl-label {
  color: var(--color-text-muted);
}

.fl-value {
  color: var(--color-text-primary);
  font-weight: var(--fw-medium);
}

.fl-highlight {
  color: var(--color-warning-text);
}

.fl-actions {
  display: flex;
  gap: var(--sp-2);
  justify-content: flex-end;
  flex-wrap: wrap;
}

.fl-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: var(--sp-2) var(--sp-4);
  border-radius: var(--r-md);
  font-size: var(--fs-12);
  font-weight: var(--fw-medium);
  border: 1px solid transparent;
  cursor: pointer;
  transition:
    background var(--dur-fast) var(--ease-smooth),
    border-color var(--dur-fast) var(--ease-smooth);
}

.fl-btn-primary {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
}
.fl-btn-primary:hover {
  background: var(--color-primary-dark);
}

.fl-btn-secondary {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
  border-color: var(--color-border);
}
.fl-btn-secondary:hover {
  border-color: var(--color-border-strong);
}

.fl-btn-ghost {
  background: transparent;
  color: var(--color-text-secondary);
}
.fl-btn-ghost:hover {
  color: var(--color-q1-text);
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

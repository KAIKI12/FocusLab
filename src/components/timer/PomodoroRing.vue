<script setup lang="ts">
/**
 * PomodoroRing · 番茄钟 SVG 圆环 + 中心倒计时。
 *
 * 设计参考 prototype/screens/pomodoro.html:91-150:
 *   - 280x280 圆环(viewBox 0 0 300 300,r=135,周长 ≈ 848)
 *   - 灰色 track + 配色 arc(随 status 变化)
 *   - 中心大号 mm:ss + 下方阶段徽章
 *
 * props.snapshot 为 null 时显示"选择任务开始计时"占位。
 *
 * 状态配色:
 *   - running (非最后 5 分钟) → primary
 *   - running (最后 5 分钟)   → warning + 微呼吸
 *   - paused                  → neutral
 *   - break                   → success
 */

import { computed } from "vue";

import type { TimerSnapshot } from "@/types";

const props = defineProps<{
  snapshot: TimerSnapshot | null;
}>();

// viewBox 300×300 · r=135 · 周长 = 2πr ≈ 848.23
const CIRCUM = 848;

const progress = computed(() => {
  const s = props.snapshot;
  if (!s || s.plannedSeconds <= 0) return 0;
  return Math.min(1, Math.max(0, s.elapsedSeconds / s.plannedSeconds));
});

/** stroke-dashoffset: 0 = 满环,CIRCUM = 空环。elapsed 越多 offset 越小。 */
const dashOffset = computed(() => CIRCUM * (1 - progress.value));

const remainingSec = computed(() => {
  const s = props.snapshot;
  if (!s) return 0;
  return Math.max(0, s.plannedSeconds - s.elapsedSeconds);
});

const timeText = computed(() => {
  const sec = remainingSec.value;
  const m = Math.floor(sec / 60);
  const r = sec % 60;
  return `${String(m).padStart(2, "0")}:${String(r).padStart(2, "0")}`;
});

/** 最后 5 分钟(focus 态)预警 */
const isFinal = computed(() => {
  const s = props.snapshot;
  if (!s || s.isBreak) return false;
  if (s.status !== "running") return false;
  return remainingSec.value > 0 && remainingSec.value <= 5 * 60;
});

/** 为 SVG fill 方便切换,把"视觉态"归成一个枚举字符串 */
const visualState = computed(() => {
  const s = props.snapshot;
  if (!s || s.status === "idle") return "idle";
  if (s.status === "paused") return "paused";
  if (s.isBreak || s.status === "break") return "break";
  if (isFinal.value) return "final";
  return "focus";
});

const phaseLabel = computed(() => {
  switch (visualState.value) {
    case "break":
      return "休息中";
    case "paused":
      return "已暂停";
    case "final":
      return "最后冲刺";
    case "focus":
      return "专注中";
    default:
      return "选择任务开始";
  }
});

const presetBadge = computed(() => {
  switch (props.snapshot?.preset) {
    case "classic_25":
      return "🍅 经典 25";
    case "deep_45":
      return "🍅🍅 深度 45";
    case "immersive_90":
      return "🍅🍅🍅 沉浸 90";
    default:
      return "";
  }
});

/** 完成的番茄点阵(最多展示 4 个) */
const pomodoroDots = computed(() => {
  const count = props.snapshot?.pomodoroCount ?? 0;
  const shown = Math.min(4, count);
  return Array.from({ length: 4 }, (_, i) => i < shown);
});
</script>

<template>
  <div class="fl-ring" :data-state="visualState">
    <div class="fl-ring-svg">
      <svg viewBox="0 0 300 300" aria-hidden="true">
        <defs>
          <linearGradient id="flRingFocus" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="var(--color-primary)" />
            <stop offset="100%" stop-color="var(--color-primary-light, var(--color-primary))" />
          </linearGradient>
          <linearGradient id="flRingFinal" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="var(--color-warning)" />
            <stop offset="100%" stop-color="var(--color-warning)" />
          </linearGradient>
          <linearGradient id="flRingBreak" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="var(--color-success)" />
            <stop offset="100%" stop-color="var(--color-success)" />
          </linearGradient>
        </defs>
        <circle class="fl-track" cx="150" cy="150" r="135" />
        <circle
          class="fl-arc"
          cx="150"
          cy="150"
          r="135"
          :stroke-dasharray="CIRCUM"
          :stroke-dashoffset="dashOffset"
        />
      </svg>

      <div class="fl-ring-center">
        <div class="fl-time">{{ snapshot ? timeText : "--:--" }}</div>
        <div class="fl-phase">{{ phaseLabel }}</div>
      </div>
    </div>

    <div v-if="snapshot" class="fl-meta">
      <span v-if="presetBadge" class="fl-badge">{{ presetBadge }}</span>
      <div class="fl-dots" :aria-label="`已完成 ${snapshot.pomodoroCount} 个番茄`">
        <span
          v-for="(done, idx) in pomodoroDots"
          :key="idx"
          class="fl-dot"
          :class="{ 'is-done': done }"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.fl-ring {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sp-3);
}

.fl-ring-svg {
  position: relative;
  width: 280px;
  height: 280px;
}

.fl-ring-svg svg {
  width: 100%;
  height: 100%;
  transform: rotate(-90deg); /* 从 12 点方向开始 */
}

.fl-track {
  fill: none;
  stroke-width: 10;
  stroke: color-mix(in srgb, var(--color-primary) 12%, transparent);
}

.fl-arc {
  fill: none;
  stroke-width: 10;
  stroke-linecap: round;
  stroke: url(#flRingFocus);
  transition: stroke-dashoffset var(--dur-slow, 320ms) var(--ease-out, ease-out);
  filter: drop-shadow(
    0 0 12px color-mix(in srgb, var(--color-primary) 25%, transparent)
  );
}

/* 最后 5 分钟 · 呼吸 */
.fl-ring[data-state="final"] .fl-track {
  stroke: color-mix(in srgb, var(--color-warning) 14%, transparent);
}
.fl-ring[data-state="final"] .fl-arc {
  stroke: url(#flRingFinal);
  animation: flRingPulse 2s ease-in-out infinite;
}
@keyframes flRingPulse {
  0%,
  100% {
    filter: drop-shadow(
      0 0 10px color-mix(in srgb, var(--color-warning) 22%, transparent)
    );
  }
  50% {
    filter: drop-shadow(
      0 0 22px color-mix(in srgb, var(--color-warning) 55%, transparent)
    );
  }
}

/* 休息中 · 绿 */
.fl-ring[data-state="break"] .fl-track {
  stroke: color-mix(in srgb, var(--color-success) 14%, transparent);
}
.fl-ring[data-state="break"] .fl-arc {
  stroke: url(#flRingBreak);
  filter: drop-shadow(
    0 0 12px color-mix(in srgb, var(--color-success) 28%, transparent)
  );
}

/* 暂停 · 灰 */
.fl-ring[data-state="paused"] .fl-track {
  stroke: color-mix(in srgb, var(--color-text-muted) 20%, transparent);
}
.fl-ring[data-state="paused"] .fl-arc {
  stroke: var(--color-text-muted);
  filter: none;
  opacity: 0.75;
}

/* Idle · 无 arc */
.fl-ring[data-state="idle"] .fl-arc {
  stroke: var(--color-border);
  filter: none;
  opacity: 0.4;
}

.fl-ring-center {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  pointer-events: none;
}

.fl-time {
  font-family: var(--font-mono, ui-monospace, SFMono-Regular, Menlo, monospace);
  font-size: 56px;
  font-weight: var(--fw-semibold);
  letter-spacing: -1.5px;
  line-height: 1;
  color: var(--color-text-primary);
  font-variant-numeric: tabular-nums;
}

.fl-ring[data-state="final"] .fl-time {
  color: var(--color-warning);
}
.fl-ring[data-state="break"] .fl-time {
  color: var(--color-success);
}
.fl-ring[data-state="paused"] .fl-time {
  color: var(--color-text-secondary);
}

.fl-phase {
  margin-top: var(--sp-2);
  font-size: var(--fs-12);
  letter-spacing: 1.5px;
  text-transform: uppercase;
  color: var(--color-text-secondary);
}

.fl-meta {
  display: flex;
  gap: var(--sp-3);
  align-items: center;
}

.fl-badge {
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
  padding: 2px var(--sp-2);
  border-radius: var(--r-pill);
  background: var(--color-bg-subtle);
}

.fl-dots {
  display: flex;
  gap: 5px;
}
.fl-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: color-mix(in srgb, var(--color-text-muted) 35%, transparent);
}
.fl-dot.is-done {
  background: var(--color-primary);
}
</style>

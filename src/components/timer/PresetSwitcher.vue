<script setup lang="ts">
/**
 * PresetSwitcher · 番茄钟模式/时长选择器。
 *
 * 仅在 timer idle 时展示。4 个 pill:25 经典 / 45 深度 / 90 沉浸 / 自由。
 */

import { useTimerStore } from "@/stores/useTimerStore";
import type { PomodoroPreset } from "@/types";

const timer = useTimerStore();

type PresetOption = PomodoroPreset | "free";

const options: { value: PresetOption; label: string }[] = [
  { value: "classic_25", label: "25 经典" },
  { value: "deep_45", label: "45 深度" },
  { value: "immersive_90", label: "90 沉浸" },
  { value: "free", label: "🌀 自由" },
];
</script>

<template>
  <div v-if="timer.isIdle" class="fl-preset" role="radiogroup" aria-label="计时模式">
    <button
      v-for="o in options"
      :key="o.value"
      class="fl-preset-pill"
      :class="{ 'is-active': timer.selectedPreset === o.value }"
      type="button"
      role="radio"
      :aria-checked="timer.selectedPreset === o.value"
      @click="timer.selectedPreset = o.value"
    >
      {{ o.label }}
    </button>
  </div>
</template>

<style scoped>
.fl-preset {
  display: flex;
  gap: var(--sp-1);
  justify-content: center;
}

.fl-preset-pill {
  padding: var(--sp-1) var(--sp-3);
  border-radius: var(--r-pill);
  border: 1px solid var(--color-border);
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 11px;
  font-weight: var(--fw-medium);
  cursor: pointer;
  transition: all var(--dur-fast) var(--ease-smooth);
}

.fl-preset-pill:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.fl-preset-pill.is-active {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
  border-color: var(--color-primary);
}
</style>

<script setup lang="ts">
/**
 * PresetSwitcher · 番茄钟模式/时长选择器。
 *
 * 仅在 timer idle 时展示。支持固定预设、单次自定义和自由计时。
 */

import { useTimerStore } from "@/stores/useTimerStore";
import type { PomodoroPreset } from "@/types";

const timer = useTimerStore();

type PresetOption = PomodoroPreset | "free";

const options: { value: PresetOption; label: string }[] = [
  { value: "classic_25", label: "25 经典" },
  { value: "deep_45", label: "45 深度" },
  { value: "immersive_90", label: "90 沉浸" },
  { value: "custom", label: "自定义" },
  { value: "free", label: "🌀 自由" },
];
</script>

<template>
  <div v-if="timer.isIdle" class="fl-preset-wrap">
    <div class="fl-preset" role="radiogroup" aria-label="计时模式">
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
    <label v-if="timer.selectedPreset === 'custom'" class="fl-custom-duration">
      <span>分钟</span>
      <input
        v-model.number="timer.selectedCustomMinutes"
        type="number"
        min="1"
        max="180"
        step="1"
        inputmode="numeric"
        aria-label="自定义番茄钟分钟数"
      />
    </label>
  </div>
</template>

<style scoped>
.fl-preset-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sp-2);
}

.fl-preset {
  display: flex;
  flex-wrap: wrap;
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

.fl-custom-duration {
  display: inline-flex;
  align-items: center;
  gap: var(--sp-1);
  padding: 3px 8px;
  border: 1px solid var(--color-border);
  border-radius: var(--r-pill);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-size: 11px;
  font-weight: var(--fw-medium);
}

.fl-custom-duration input {
  width: 52px;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--color-text-primary);
  font: inherit;
  text-align: center;
}

.fl-custom-duration input:focus {
  box-shadow: 0 1px 0 var(--color-primary);
}
</style>

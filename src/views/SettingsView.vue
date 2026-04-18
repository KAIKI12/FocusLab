<script setup lang="ts">
/**
 * SettingsView · Week 1a 只提供主题快速切换,用于验证 useTheme 在 Vue 下的联动。
 * Phase 1 Week 4 会接入完整的设置页(53 项)。
 */

import { useTheme, type ThemeMode } from "@/composables/useTheme";

const { mode, accent, setMode, setAccent } = useTheme();

const modes: ThemeMode[] = ["light", "dark", "auto"];
const accents = [
  "default",
  "claude",
  "green",
  "lavender",
  "blue-classic",
  "graphite",
  "sakura",
  "candy",
  "milktea",
  "amber",
  "teal",
  "slate",
];
</script>

<template>
  <section class="fl-settings">
    <header class="fl-page-head">
      <h1>设置</h1>
      <p class="fl-page-sub">
        Week 1a 占位 · 主题切换用于验证 CSS 变量 + localStorage 的 Vue 侧联动
      </p>
    </header>

    <div class="fl-setting-block">
      <div class="fl-setting-label">明暗模式</div>
      <div class="fl-segmented">
        <button
          v-for="m in modes"
          :key="m"
          type="button"
          class="fl-seg-btn"
          :class="{ 'is-active': mode === m }"
          @click="setMode(m)"
        >
          {{ m === "light" ? "浅色" : m === "dark" ? "深色" : "跟随系统" }}
        </button>
      </div>
    </div>

    <div class="fl-setting-block">
      <div class="fl-setting-label">主题色调</div>
      <div class="fl-accent-grid">
        <button
          v-for="a in accents"
          :key="a"
          type="button"
          class="fl-accent-btn"
          :class="{ 'is-active': accent === a }"
          @click="setAccent(a)"
        >
          {{ a }}
        </button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.fl-settings {
  max-width: 720px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--sp-6);
}

.fl-page-head h1 {
  font-size: var(--fs-24);
  font-weight: var(--fw-semibold);
  margin: 0;
  color: var(--color-text-primary);
}

.fl-page-sub {
  margin: var(--sp-1) 0 0;
  color: var(--color-text-secondary);
  font-size: var(--fs-12);
}

.fl-setting-block {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  padding: var(--sp-4) var(--sp-5);
}

.fl-setting-label {
  font-size: var(--fs-12);
  font-weight: var(--fw-medium);
  color: var(--color-text-secondary);
  margin-bottom: var(--sp-3);
  letter-spacing: 0.3px;
  text-transform: uppercase;
}

.fl-segmented {
  display: inline-flex;
  background: var(--color-bg-subtle);
  padding: 4px;
  border-radius: var(--r-md);
  gap: 2px;
}

.fl-seg-btn {
  padding: var(--sp-2) var(--sp-4);
  border: none;
  background: transparent;
  border-radius: var(--r-sm);
  color: var(--color-text-secondary);
  font-size: var(--fs-12);
  cursor: pointer;
  transition: all var(--dur-fast) var(--ease-smooth);
}

.fl-seg-btn.is-active {
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
  box-shadow: var(--shadow-card);
}

.fl-accent-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: var(--sp-2);
}

.fl-accent-btn {
  padding: var(--sp-2) var(--sp-3);
  border: 1px solid var(--color-border);
  background: var(--color-bg-subtle);
  color: var(--color-text-secondary);
  border-radius: var(--r-sm);
  font-size: var(--fs-12);
  cursor: pointer;
  transition: all var(--dur-fast) var(--ease-smooth);
}

.fl-accent-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-text-primary);
}

.fl-accent-btn.is-active {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
  border-color: var(--color-primary);
}
</style>

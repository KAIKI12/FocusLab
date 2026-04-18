<script setup lang="ts">
/**
 * SettingsView · Week 1a 主题切换 + Week 1b 崩溃恢复调试面板(仅 DEV)。
 *
 * Phase 1 Week 4 会接入完整的设置页(53 项)。
 */

import { ref } from "vue";

import { useTheme, type ThemeMode } from "@/composables/useTheme";
import { useTimerStateStore } from "@/stores/useTimerStateStore";

const { mode, accent, setMode, setAccent } = useTheme();

const modes: ThemeMode[] = ["light", "dark", "auto"];

/** 12 套色调主题 — id 与 tokens.css 的 data-accent-theme 严格对齐,
 *  中文名取自 docs/05 §1.1.1 */
const accents: { id: string; label: string }[] = [
  { id: "default", label: "🌊 默认蓝" },
  { id: "claude", label: "☁️ 奶油陶土" },
  { id: "green", label: "🌿 护眼绿" },
  { id: "lavender", label: "🪻 薰衣草紫" },
  { id: "blue-classic", label: "🌊 静谧蓝" },
  { id: "graphite", label: "🧊 极简石墨" },
  { id: "sakura", label: "🌸 樱花粉" },
  { id: "candy", label: "🎀 糖果粉紫" },
  { id: "milktea", label: "🧋 奶茶棕粉" },
  { id: "amber", label: "🍊 琥珀橙" },
  { id: "teal", label: "🦆 水鸭青" },
  { id: "slate", label: "🪨 石板蓝灰" },
];

function onAccentChange(e: Event) {
  setAccent((e.target as HTMLSelectElement).value);
}

// ---------- Week 1b · DEV 调试面板 ----------

const isDev = import.meta.env.DEV;
const timerStore = useTimerStateStore();
const log = ref<string>("");

function pastIso(secondsAgo: number): string {
  return new Date(Date.now() - secondsAgo * 1000).toISOString();
}

/** 模拟崩溃:写一条 "running" 状态 + updated_at 为 secondsAgo 前 */
async function simulateCrash(secondsAgo: number, tag: string) {
  try {
    await timerStore.update({
      status: "running",
      mode: "pomodoro",
      pomodoroPreset: "classic_25",
      plannedSeconds: 1500,
      elapsedSeconds: Math.min(secondsAgo, 1500),
      startTime: pastIso(secondsAgo + 5),
      updatedAt: pastIso(secondsAgo),
    });
    log.value = `✅ 已注入 ${tag} · 重启应用(Ctrl+R)查看分支`;
  } catch (e) {
    log.value = `❌ 失败:${e}`;
  }
}

async function clearTimer() {
  try {
    await timerStore.reset();
    log.value = "✅ timer_state 已回到 idle";
  } catch (e) {
    log.value = `❌ 失败:${e}`;
  }
}

async function showCurrent() {
  try {
    await timerStore.load();
    log.value = JSON.stringify(timerStore.state, null, 2);
  } catch (e) {
    log.value = `❌ 失败:${e}`;
  }
}
</script>

<template>
  <section class="fl-settings">
    <header class="fl-page-head">
      <h1>设置</h1>
      <p class="fl-page-sub">
        Week 1a 占位 · 主题切换 + Week 1b 崩溃恢复调试
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
      <select
        class="fl-accent-select"
        :value="accent"
        aria-label="主题色调选择"
        @change="onAccentChange"
      >
        <option v-for="a in accents" :key="a.id" :value="a.id">
          {{ a.label }}
        </option>
      </select>
      <div class="fl-accent-swatch" aria-hidden="true">
        <span class="fl-swatch-dot fl-swatch-primary"></span>
        <span class="fl-swatch-dot fl-swatch-success"></span>
        <span class="fl-swatch-dot fl-swatch-gold"></span>
        <span class="fl-swatch-dot fl-swatch-q1"></span>
        <span class="fl-swatch-label">当前色板预览</span>
      </div>
    </div>

    <div v-if="isDev" class="fl-setting-block fl-dev">
      <div class="fl-setting-label">🧪 崩溃恢复调试(仅开发构建)</div>
      <p class="fl-dev-desc">
        按钮直接往 timer_state 里注入"假崩溃"记录,随后按
        <kbd>Ctrl</kbd>+<kbd>R</kbd> 刷新应用,观察 RecoveryDialog 的三档分支。
      </p>
      <div class="fl-dev-grid">
        <button class="fl-btn" type="button" @click="simulateCrash(30, '30s 前')">
          30s 前(预期 AutoResume)
        </button>
        <button class="fl-btn" type="button" @click="simulateCrash(600, '10min 前')">
          10min 前(预期 AskUser)
        </button>
        <button class="fl-btn" type="button" @click="simulateCrash(7200, '2h 前')">
          2h 前(预期 AutoEnd)
        </button>
        <button class="fl-btn fl-btn-ghost" type="button" @click="clearTimer">
          清空 timer_state
        </button>
        <button class="fl-btn fl-btn-ghost" type="button" @click="showCurrent">
          读取当前
        </button>
      </div>
      <pre v-if="log" class="fl-dev-log">{{ log }}</pre>
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

.fl-accent-select {
  width: 100%;
  padding: var(--sp-3) var(--sp-4);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  background: var(--color-bg-subtle);
  color: var(--color-text-primary);
  font-size: var(--fs-14);
  font-family: var(--font-sans);
  cursor: pointer;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 24 24' fill='none' stroke='%238C8C8C' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right var(--sp-3) center;
  padding-right: var(--sp-8);
  transition:
    border-color var(--dur-fast) var(--ease-smooth),
    box-shadow var(--dur-fast) var(--ease-smooth);
}

.fl-accent-select:hover {
  border-color: var(--color-primary);
}

.fl-accent-select:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.fl-accent-swatch {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
  margin-top: var(--sp-3);
  padding: var(--sp-2) var(--sp-3);
  background: var(--color-bg-subtle);
  border-radius: var(--r-sm);
}

.fl-swatch-dot {
  width: 14px;
  height: 14px;
  border-radius: var(--r-pill);
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-text-primary) 8%, transparent);
}

.fl-swatch-primary { background: var(--color-primary); }
.fl-swatch-success { background: var(--color-success); }
.fl-swatch-gold    { background: var(--color-gold); }
.fl-swatch-q1      { background: var(--color-q1); }

.fl-swatch-label {
  margin-left: auto;
  font-size: var(--fs-12);
  color: var(--color-text-muted);
}

/* ---------- Dev 面板 ---------- */
.fl-dev {
  border: 1px dashed var(--color-warning);
  background: var(--color-warning-soft);
}

.fl-dev-desc {
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
  margin: 0 0 var(--sp-3);
}

.fl-dev-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: var(--sp-2);
}

.fl-btn {
  padding: var(--sp-2) var(--sp-3);
  border: 1px solid var(--color-border);
  border-radius: var(--r-sm);
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
  font-size: var(--fs-12);
  cursor: pointer;
  transition: all var(--dur-fast) var(--ease-smooth);
}

.fl-btn:hover {
  border-color: var(--color-warning);
  color: var(--color-warning-text);
}

.fl-btn-ghost {
  background: transparent;
  color: var(--color-text-secondary);
}

.fl-dev-log {
  margin-top: var(--sp-3);
  padding: var(--sp-3);
  background: var(--color-bg-subtle);
  border-radius: var(--r-sm);
  font-family: var(--font-mono);
  font-size: var(--fs-12);
  color: var(--color-text-primary);
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 200px;
  overflow: auto;
}

kbd {
  padding: 1px 5px;
  border: 1px solid var(--color-border);
  border-radius: 3px;
  background: var(--color-bg-elevated);
  font-family: var(--font-mono);
  font-size: 10px;
}
</style>

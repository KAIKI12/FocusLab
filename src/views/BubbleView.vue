<script setup lang="ts">
/**
 * BubbleView · 悬浮球 — 拖拽零延迟 + Mini Panel 对齐原型浅色设计。
 */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";

import type { TimerSnapshot, TimerStatus } from "@/types";

const snapshot = ref<TimerSnapshot | null>(null);
const expanded = ref(false);
const taskName = ref("FocusLab");
const showComplete = ref(false);
const otherTasks = ref<Array<{ id: string; name: string; quadrant: string }>>([]);
let unlisteners: UnlistenFn[] = [];

const appWindow = getCurrentWindow();

const isIdle = computed(() => !snapshot.value || snapshot.value.status === "idle");
const isRunning = computed(() => snapshot.value?.status === "running");
const isPaused = computed(() => snapshot.value?.status === "paused");
const isBreak = computed(() => snapshot.value?.status === "break" || snapshot.value?.status === "break_ended");
const isFree = computed(() => snapshot.value?.mode === "free");

const visualState = computed(() => {
  if (showComplete.value) return "complete";
  const s = snapshot.value;
  if (!s || s.status === "idle") return "breathing";
  if (s.mode === "free" && (s.status === "running" || s.status === "paused")) return "free";
  if (s.status === "running" || s.status === "paused") return "focusing";
  if (s.status === "break" || s.status === "break_ended") return "break";
  return "breathing";
});

const timeText = computed(() => {
  const s = snapshot.value;
  if (!s || s.status === "idle") return "";
  if (s.mode === "free") {
    const m = Math.floor(s.elapsedSeconds / 60);
    const sec = s.elapsedSeconds % 60;
    return `${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
  }
  const remaining = Math.max(0, s.plannedSeconds - s.elapsedSeconds);
  const m = Math.floor(remaining / 60);
  const sec = remaining % 60;
  return `${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
});

const progress = computed(() => {
  const s = snapshot.value;
  if (!s || s.plannedSeconds <= 0) return 0;
  return Math.min(1, Math.max(0, s.elapsedSeconds / s.plannedSeconds));
});

const phaseLabel = computed(() => {
  const s = snapshot.value;
  if (!s || s.status === "idle") return "FocusLab";
  if (s.mode === "free") return `🌀 自由计时`;
  if (s.status === "running") return `专注中 · 第 ${s.pomodoroCount} 番茄`;
  if (s.status === "paused") return `⏸ 已暂停 · 第 ${s.pomodoroCount} 番茄`;
  if (s.status === "break") return "☕ 休息中";
  if (s.status === "break_ended") return "☕ 休息结束";
  return "";
});

const focusedTime = computed(() => {
  const s = snapshot.value;
  if (!s) return "";
  const m = s.elapsedSeconds;
  if (m < 60) return `已专注 ${m}s`;
  const min = Math.floor(m / 60);
  if (min < 60) return `已专注 ${min}m`;
  const h = Math.floor(min / 60);
  return `已专注 ${h}h ${min % 60}m`;
});

// =============================================
// 拖拽 — 缓存位置，零 await 延迟
// =============================================

let cachedWinX = 0;
let cachedWinY = 0;
let dragStartScreenX = 0;
let dragStartScreenY = 0;
let isDragging = false;
let hasMoved = false;

async function onMouseDown(e: MouseEvent) {
  dragStartScreenX = e.screenX;
  dragStartScreenY = e.screenY;
  isDragging = true;
  hasMoved = false;
}

async function onMouseMove(e: MouseEvent) {
  if (!isDragging) return;
  const dx = e.screenX - dragStartScreenX;
  const dy = e.screenY - dragStartScreenY;
  if (!hasMoved && Math.abs(dx) < 3 && Math.abs(dy) < 3) return;

  if (!hasMoved) {
    hasMoved = true;
    // 用 Tauri 原生拖拽 — OS 接管窗口移动，不扩大窗口
    try {
      await appWindow.startDragging();
    } catch { /* 某些平台不支持则 fallback */ }
    isDragging = false; // startDragging 会接管，不再需要自己处理
    return;
  }
}

async function onMouseUp() {
  if (isDragging && hasMoved) {
    // startDragging 已接管，这里只更新缓存
    try {
      const pos = await appWindow.outerPosition();
      cachedWinX = pos.x;
      cachedWinY = pos.y;
      localStorage.setItem("fl-bubble-pos", JSON.stringify({ x: pos.x, y: pos.y }));
    } catch {}
  }
  isDragging = false;
}

onMounted(() => {
  document.addEventListener("mousemove", onMouseMove);
  document.addEventListener("mouseup", onMouseUp);
});
onUnmounted(() => {
  document.removeEventListener("mousemove", onMouseMove);
  document.removeEventListener("mouseup", onMouseUp);
});

// =============================================
// 手势(对齐原型)：单击展开面板，双击暂停/继续
// =============================================

let clickTimer: ReturnType<typeof setTimeout> | null = null;
let clickCount = 0;

function onOrbClick() {
  if (hasMoved) return;
  clickCount++;
  if (clickCount === 1) {
    clickTimer = setTimeout(() => {
      // 单击 → 展开/收缩 Mini Panel
      expanded.value = !expanded.value;
      clickCount = 0;
    }, 250);
  } else if (clickCount === 2) {
    if (clickTimer) clearTimeout(clickTimer);
    clickCount = 0;
    // 双击 → 暂停/继续(专注中) 或 打开主窗口(idle)
    onDoubleClick();
  }
}

async function onDoubleClick() {
  if (isRunning.value || isPaused.value) await togglePause();
  else if (isIdle.value) await focusMainWindow();
}

async function togglePause() {
  if (!snapshot.value) return;
  try {
    if (snapshot.value.status === "running") await invoke("pause_timer");
    else if (snapshot.value.status === "paused") await invoke("resume_timer");
  } catch (e) { console.error(e); }
}

async function onAbandon() { try { await invoke("abandon_timer", { reason: null }); } catch {} }
async function onSkipBreak() { try { await invoke("skip_break"); } catch {} }

async function focusMainWindow() {
  try {
    await invoke("show_main_window");
  } catch (e) { console.error("[bubble] focusMainWindow failed", e); }
}

async function closeBubble() { await appWindow.close(); }

// =============================================
// 窗口大小
// =============================================

watch(expanded, async (exp) => {
  if (exp) {
    const sW = window.screen.width;
    const sH = window.screen.height;
    let x = Math.max(0, Math.min(cachedWinX, sW - 340));
    let y = Math.max(0, Math.min(cachedWinY - 400, sH - 480));
    if (y < 0) y = cachedWinY + 70;
    await appWindow.setPosition(new LogicalPosition(x, y));
    await appWindow.setSize(new LogicalSize(340, 460));
  } else {
    await appWindow.setSize(new LogicalSize(64, 64));
    // 恢复原位
    await appWindow.setPosition(new LogicalPosition(cachedWinX, cachedWinY));
  }
});

// =============================================
// 任务名 + 完成闪光 + 生命周期
// =============================================

async function refreshTaskName() {
  const id = snapshot.value?.taskId;
  if (!id) { taskName.value = "FocusLab"; return; }
  try {
    const name = await invoke<string | null>("get_task_name", { id });
    taskName.value = name ?? "(任务)";
  } catch { taskName.value = "(任务)"; }
  // 加载其他任务(快速切换用)
  try {
    const tasks = await invoke<Array<{ id: string; name: string; quadrant: string }>>(
      "list_tasks", { statusFilter: null }
    );
    otherTasks.value = tasks.filter(t => t.id !== id).slice(0, 3);
  } catch { otherTasks.value = []; }
}

watch(() => snapshot.value?.taskId, () => refreshTaskName());
watch(() => snapshot.value?.status, (n, o) => {
  if (o === "running" && (n === "break" || n === "break_ended")) {
    showComplete.value = true;
    setTimeout(() => { showComplete.value = false; }, 1200);
  }
});

onMounted(async () => {
  await appWindow.setSize(new LogicalSize(64, 64));
  // 初始化缓存位置
  try {
    const pos = await appWindow.outerPosition();
    cachedWinX = pos.x;
    cachedWinY = pos.y;
  } catch {}

  try {
    const row = await invoke<{
      status: string; task_id: string | null; session_id: string | null;
      mode: string | null; pomodoro_preset: string | null;
      elapsed_seconds: number; planned_seconds: number | null;
      pomodoro_count: number; is_break: boolean; break_remaining: number | null;
    }>("get_timer_state");
    snapshot.value = {
      status: row.status as TimerStatus, taskId: row.task_id,
      sessionId: row.session_id, mode: row.mode as "pomodoro" | "free" | null,
      preset: row.pomodoro_preset as TimerSnapshot["preset"],
      elapsedSeconds: row.elapsed_seconds,
      plannedSeconds: row.is_break ? (row.break_remaining ?? 0) : (row.planned_seconds ?? 0),
      pomodoroCount: row.pomodoro_count, isBreak: row.is_break,
    };
  } catch {}
  await refreshTaskName();

  const handler = (ev: { payload: TimerSnapshot }) => { snapshot.value = ev.payload; };
  const u1 = await listen<TimerSnapshot>("timer:tick", handler);
  const u2 = await listen<TimerSnapshot>("timer:state_changed", handler);
  unlisteners = [u1, u2];
});
onUnmounted(() => { unlisteners.forEach((fn) => fn()); });
</script>

<template>
  <div class="fl-root" :data-state="visualState" :class="{ 'is-expanded': expanded }">
    <!-- ===== 收缩态 ===== -->
    <div
      v-if="!expanded"
      class="fl-orb"
      @mousedown="onMouseDown"
      @click="onOrbClick"
      @dblclick.prevent
    >
      <svg v-if="visualState === 'focusing'" class="fl-orb-ring" viewBox="0 0 64 64">
        <circle cx="32" cy="32" r="28" fill="none" stroke="rgba(255,255,255,0.2)" stroke-width="3" />
        <circle cx="32" cy="32" r="28" fill="none" stroke="rgba(255,255,255,0.85)" stroke-width="3"
          stroke-linecap="round" :stroke-dasharray="176" :stroke-dashoffset="176 * progress"
          style="transform:rotate(-90deg);transform-origin:50% 50%;transition:stroke-dashoffset .3s" />
      </svg>
      <div v-if="visualState === 'free'" class="fl-free-ring" />
      <div class="fl-orb-inner">
        <span v-if="visualState === 'complete'">✓</span>
        <span v-else-if="visualState === 'breathing'" class="fl-logo">F</span>
        <span v-else class="fl-time">{{ timeText }}</span>
      </div>
    </div>

    <!-- ===== Mini Panel(对齐原型 op-panel) ===== -->
    <div v-else class="fl-mp">
      <div class="fl-mp-head" @mousedown="onMouseDown">
        <div class="fl-mp-phase">
          <span class="fl-mp-dot" />
          {{ phaseLabel }}
        </div>
        <button class="fl-mp-close" @click="expanded = false">✕</button>
      </div>
      <div class="fl-mp-body">
        <div class="fl-mp-task">{{ taskName }}</div>
        <div class="fl-mp-meta">{{ focusedTime }}</div>
        <div class="fl-mp-ring">
          <svg viewBox="0 0 120 120">
            <circle cx="60" cy="60" r="50" fill="none" stroke="var(--color-border,#e5e7eb)" stroke-width="6" />
            <circle v-if="!isFree" cx="60" cy="60" r="50" fill="none"
              stroke="var(--color-primary,#4f8cff)" stroke-width="6" stroke-linecap="round"
              :stroke-dasharray="314" :stroke-dashoffset="314 * (1 - progress)"
              style="transform:rotate(-90deg);transform-origin:50% 50%;transition:stroke-dashoffset .3s" />
          </svg>
          <div class="fl-mp-ring-center">{{ timeText || "--:--" }}</div>
        </div>
        <div class="fl-mp-ctrl">
          <button v-if="!isIdle" class="fl-mp-cbtn" title="结束" @click="onAbandon">◼</button>
          <button v-if="isRunning||isPaused" class="fl-mp-cbtn fl-mp-main" @click="togglePause">{{ isPaused ? '▶' : '⏸' }}</button>
          <button v-if="isBreak" class="fl-mp-cbtn" title="跳过" @click="onSkipBreak">⏭</button>
        </div>
        <div v-if="otherTasks.length && !isIdle" class="fl-mp-switch">
          <div class="fl-mp-sw-label">快速切换</div>
          <div v-for="t in otherTasks" :key="t.id" class="fl-mp-sw-item">
            <span class="fl-mp-sw-dot" />
            <span class="fl-mp-sw-name">{{ t.name }}</span>
          </div>
        </div>
      </div>
      <div class="fl-mp-foot">
        <button @click="focusMainWindow">🏠 打开主窗口</button>
        <button @click="closeBubble">✕ 关闭悬浮球</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fl-root {
  width: 100%; height: 100%;
  display: flex; align-items: center; justify-content: center;
}
.fl-root:not(.is-expanded) {
  border-radius: 50%;
  overflow: hidden;
}

/* ===== 圆球 ===== */
.fl-orb {
  width: 100%; height: 100%; border-radius: 50%;
  position: relative; cursor: pointer; user-select: none;
  display: flex; align-items: center; justify-content: center;
}
.fl-orb:hover { transform: scale(1.05); }

[data-state="breathing"] .fl-orb {
  background: linear-gradient(135deg, #4f8cff, #7aabff);
  box-shadow: 0 4px 14px rgba(79,140,255,0.35);
  animation: breathe 4s ease-in-out infinite;
}
@keyframes breathe {
  0%,100% { opacity: 0.8; } 50% { opacity: 1; }
}
[data-state="focusing"] .fl-orb { background: #4f8cff; box-shadow: 0 4px 16px rgba(79,140,255,0.4); }
[data-state="free"] .fl-orb { background: linear-gradient(135deg, #8b5cf6, #a78bfa); box-shadow: 0 4px 16px rgba(139,92,246,0.45); }
[data-state="break"] .fl-orb { background: linear-gradient(135deg, #52c41a, #73d13d); box-shadow: 0 4px 16px rgba(82,196,26,0.4); }
[data-state="complete"] .fl-orb { background: linear-gradient(135deg, #52c41a, #95de64); animation: flash 0.6s ease-out 2; }
@keyframes flash {
  50% { box-shadow: 0 4px 28px rgba(82,196,26,0.8), 0 0 0 6px rgba(82,196,26,0.2); }
}

.fl-orb-ring { position: absolute; inset: 0; width: 100%; height: 100%; }
.fl-free-ring {
  position: absolute; inset: -3px; border-radius: 50%;
  border: 2px dashed rgba(255,255,255,0.6);
  animation: spin 12s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

.fl-orb-inner { position: relative; z-index: 1; text-align: center; pointer-events: none; color: #fff; }
.fl-logo { font-size: 22px; font-weight: 700; }
.fl-time { font-family: ui-monospace, monospace; font-size: 13px; font-weight: 600; letter-spacing: -0.5px; }

/* ===== Mini Panel · 浅色(对齐原型) ===== */
.fl-mp {
  width: 100%;
  height: 100%;
  background: var(--color-bg-elevated, #fff);
  border-radius: 12px;
  border: 1px solid var(--color-border, #e5e7eb);
  display: flex; flex-direction: column;
  overflow: hidden;
  color: var(--color-text-primary, #1a1a2e);
}

.fl-mp-head {
  display: flex; align-items: center; justify-content: space-between;
  padding: 10px 14px;
  border-bottom: 1px solid var(--color-border, #e5e7eb);
  cursor: grab;
}
.fl-mp-phase {
  display: flex; align-items: center; gap: 6px;
  font-size: 12px; font-weight: 500;
  color: var(--color-primary, #4f8cff);
}
.fl-mp-dot {
  width: 6px; height: 6px; border-radius: 50%;
  background: var(--color-primary, #4f8cff);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-primary, #4f8cff) 25%, transparent);
}
.fl-mp-close {
  background: none; border: none;
  color: var(--color-text-muted, #999);
  font-size: 14px; cursor: pointer; padding: 2px 4px;
}
.fl-mp-close:hover { color: var(--color-text-primary, #1a1a2e); }

.fl-mp-body {
  padding: 14px;
  display: flex; flex-direction: column; align-items: center; gap: 8px;
  flex: 1; justify-content: center;
}
.fl-mp-task {
  font-size: 14px; font-weight: 600;
  text-align: center; max-width: 100%;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.fl-mp-meta {
  font-size: 12px; color: var(--color-text-muted, #999);
}

/* 环 */
.fl-mp-ring {
  width: 120px; height: 120px; position: relative;
  margin: 4px 0;
}
.fl-mp-ring svg { width: 100%; height: 100%; }
.fl-mp-ring-center {
  position: absolute; inset: 0;
  display: flex; align-items: center; justify-content: center;
  font-family: ui-monospace, monospace;
  font-size: 24px; font-weight: 600;
  color: var(--color-text-primary, #1a1a2e);
}

/* 控制(圆形按钮) */
.fl-mp-ctrl { display: flex; gap: 8px; align-items: center; justify-content: center; }
.fl-mp-cbtn {
  width: 36px; height: 36px; border-radius: 50%;
  background: var(--color-bg-hover, #f5f5f5); border: none;
  cursor: pointer; font-size: 14px; color: var(--color-text-secondary, #666);
  display: grid; place-items: center; transition: all 0.15s;
}
.fl-mp-cbtn:hover { background: color-mix(in srgb, var(--color-primary,#4f8cff) 15%, transparent); color: var(--color-primary,#4f8cff); }
.fl-mp-main {
  width: 44px; height: 44px;
  background: var(--color-primary,#4f8cff); color: #fff;
  box-shadow: 0 4px 12px color-mix(in srgb, var(--color-primary,#4f8cff) 30%, transparent);
  font-size: 18px;
}
.fl-mp-main:hover { transform: scale(1.05); background: var(--color-primary,#4f8cff); color: #fff; }

/* 快速切换 */
.fl-mp-switch { width: 100%; margin-top: 4px; padding-top: 8px; border-top: 1px solid var(--color-border,#e5e7eb); }
.fl-mp-sw-label { font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--color-text-muted,#999); margin-bottom: 6px; }
.fl-mp-sw-item {
  display: flex; align-items: center; gap: 8px;
  padding: 5px 8px; border-radius: 4px; cursor: pointer; font-size: 13px; transition: background 0.15s;
}
.fl-mp-sw-item:hover { background: var(--color-bg-hover,#f5f5f5); }
.fl-mp-sw-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--color-primary,#4f8cff); flex-shrink: 0; }
.fl-mp-sw-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

/* 底部 */
.fl-mp-foot {
  display: flex;
  border-top: 1px solid var(--color-border, #e5e7eb);
  background: var(--color-bg-subtle, #fafafa);
}
.fl-mp-foot button {
  flex: 1; padding: 10px;
  background: none; border: none;
  font-size: 11px; cursor: pointer;
  color: var(--color-text-muted, #999);
}
.fl-mp-foot button:hover { color: var(--color-primary, #4f8cff); }
.fl-mp-foot-close:hover { color: #ef4444 !important; }
.fl-mp-foot button + button { border-left: 1px solid var(--color-border, #e5e7eb); }
</style>

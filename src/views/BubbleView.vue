<script setup lang="ts">
/**
 * BubbleView · 悬浮球 — 拖拽零延迟 + Mini Panel 对齐原型浅色设计。
 */

import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";
import { Menu } from "@tauri-apps/api/menu";
import {
  Activity,
  Check,
  Coffee,
  Home,
  Pause,
  Play,
  SkipForward,
  Square,
  Timer,
  X,
} from "lucide-vue-next";

import type { TimerSnapshot, TimerStatus } from "@/types";

const COLLAPSED_WINDOW_SIZE = 64;
const PANEL_WIDTH = 344;
const PANEL_HEIGHT = 432;
const DRAG_THRESHOLD_PX = 3;
const DOUBLE_CLICK_DELAY_MS = 250;
const COMPLETE_FLASH_MS = 1200;
const PANEL_RING_CIRCUMFERENCE = 314;
const OPACITY_SETTING_KEY = "float_opacity";
const DEFAULT_ORB_OPACITY = 1;
const MIN_ORB_OPACITY = 0.4;
const MAX_ORB_OPACITY = 1;
const OPACITY_OPTIONS = [
  { label: "40%", value: 0.4 },
  { label: "60%", value: 0.6 },
  { label: "80%", value: 0.8 },
  { label: "100%", value: 1 },
] as const;

const snapshot = ref<TimerSnapshot | null>(null);
const expanded = ref(false);
const taskName = ref("FocusLab");
const showComplete = ref(false);
const orbOpacity = ref(DEFAULT_ORB_OPACITY);
const otherTasks = ref<Array<{ id: string; name: string; quadrant: string }>>([]);
let unlisteners: UnlistenFn[] = [];
let contextMenu: Menu | null = null;

const appWindow = getCurrentWindow();

const isIdle = computed(() => !snapshot.value || snapshot.value.status === "idle");
const isRunning = computed(() => snapshot.value?.status === "running");
const isPaused = computed(() => snapshot.value?.status === "paused");
const isBreak = computed(() => snapshot.value?.status === "break" || snapshot.value?.status === "break_ended");
const isFree = computed(() => snapshot.value?.mode === "free");
const rootStyle = computed(() => ({ "--fl-bubble-opacity": String(orbOpacity.value) }));

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
  if (s.mode === "free") return "自由计时";
  if (s.status === "running") return `专注中 · 第 ${s.pomodoroCount} 番茄`;
  if (s.status === "paused") return `已暂停 · 第 ${s.pomodoroCount} 番茄`;
  if (s.status === "break") return "休息中";
  if (s.status === "break_ended") return "休息结束";
  return "";
});

const phaseIcon = computed(() => {
  if (showComplete.value) return Check;
  if (isBreak.value) return Coffee;
  if (isFree.value) return Activity;
  return Timer;
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

function normalizeOpacity(value: unknown): number {
  const n = Number(value);
  if (!Number.isFinite(n)) return DEFAULT_ORB_OPACITY;
  return Math.min(MAX_ORB_OPACITY, Math.max(MIN_ORB_OPACITY, n));
}

async function loadOrbOpacity() {
  try {
    const raw = await invoke<string | null>("get_setting", { key: OPACITY_SETTING_KEY });
    orbOpacity.value = raw == null ? DEFAULT_ORB_OPACITY : normalizeOpacity(raw);
  } catch (e) {
    console.error("[bubble] load opacity failed", e);
  }
}

async function setOrbOpacity(value: number) {
  const previous = orbOpacity.value;
  const next = normalizeOpacity(value);
  orbOpacity.value = next;
  try {
    await invoke("set_setting", { key: OPACITY_SETTING_KEY, value: String(next) });
  } catch (e) {
    orbOpacity.value = previous;
    console.error("[bubble] save opacity failed", e);
  }
}

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
  if (e.button !== 0) return;
  dragStartScreenX = e.screenX;
  dragStartScreenY = e.screenY;
  isDragging = true;
  hasMoved = false;
}

async function onMouseMove(e: MouseEvent) {
  if (!isDragging) return;
  const dx = e.screenX - dragStartScreenX;
  const dy = e.screenY - dragStartScreenY;
  if (!hasMoved && Math.abs(dx) < DRAG_THRESHOLD_PX && Math.abs(dy) < DRAG_THRESHOLD_PX) return;

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
    }, DOUBLE_CLICK_DELAY_MS);
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

async function openQuickAddWindow() {
  try {
    await invoke("show_quick_add_window");
  } catch (e) { console.error("[bubble] openQuickAddWindow failed", e); }
}

async function openQuickNoteWindow() {
  try {
    await invoke("show_quick_note_window");
  } catch (e) { console.error("[bubble] openQuickNoteWindow failed", e); }
}

async function emitTrayAction(type: "switch-task" | "quick-add" | "quick-note" | "settle-today") {
  await focusMainWindow();
  await emit("focuslab:tray:action", { type });
}

async function openSettings() {
  await focusMainWindow();
  await emit("focuslab:tray:navigate", "/settings");
}

async function openContextMenu(e: MouseEvent) {
  e.preventDefault();
  e.stopPropagation();
  isDragging = false;
  clickCount = 0;
  if (clickTimer) {
    clearTimeout(clickTimer);
    clickTimer = null;
  }

  const canTogglePause = isRunning.value || isPaused.value;
  const opacityItems = OPACITY_OPTIONS.map((item) => ({
    id: `opacity-${Math.round(item.value * 100)}`,
    text: `${orbOpacity.value === item.value ? "✓ " : ""}${item.label}`,
    action: () => { void setOrbOpacity(item.value); },
  }));

  try {
    contextMenu = await Menu.new({
      items: [
        { id: "open-main", text: "打开主窗口", accelerator: "Ctrl+Shift+F", action: () => { void focusMainWindow(); } },
        { id: "toggle-pause", text: "暂停 / 继续", enabled: canTogglePause, accelerator: "Space", action: () => { void togglePause(); } },
        { id: "switch-task", text: "切换任务", enabled: !isIdle.value, action: () => { void emitTrayAction("switch-task"); } },
        { item: "Separator" },
        {
          id: "opacity",
          text: "透明度",
          items: opacityItems,
        },
        { item: "Separator" },
        { id: "quick-add", text: "快速添加任务", accelerator: "Ctrl+N", action: () => { void openQuickAddWindow(); } },
        { id: "quick-note", text: "速记便签", accelerator: "Ctrl+Shift+N", action: () => { void openQuickNoteWindow(); } },
        { id: "settle-today", text: "结束今天", action: () => { void emitTrayAction("settle-today"); } },
        { item: "Separator" },
        { id: "settings", text: "设置", accelerator: "Ctrl+,", action: () => { void openSettings(); } },
        { id: "close-bubble", text: "关闭悬浮球", action: () => { void closeBubble(); } },
      ],
    });
    await contextMenu.popup(new LogicalPosition(e.clientX, e.clientY), appWindow);
  } catch (err) {
    console.error("[bubble] context menu failed", err);
  }
}

// =============================================
// 窗口大小
// =============================================

watch(expanded, async (exp) => {
  if (exp) {
    const sW = window.screen.width;
    const sH = window.screen.height;
    let x = Math.max(0, Math.min(cachedWinX, sW - PANEL_WIDTH));
    const preferredY = cachedWinY - (PANEL_HEIGHT - COLLAPSED_WINDOW_SIZE);
    let y = preferredY < 0 ? cachedWinY + COLLAPSED_WINDOW_SIZE : preferredY;
    y = Math.max(0, Math.min(y, sH - PANEL_HEIGHT));
    await appWindow.setPosition(new LogicalPosition(x, y));
    await appWindow.setSize(new LogicalSize(PANEL_WIDTH, PANEL_HEIGHT));
  } else {
    await appWindow.setSize(new LogicalSize(COLLAPSED_WINDOW_SIZE, COLLAPSED_WINDOW_SIZE));
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
    setTimeout(() => { showComplete.value = false; }, COMPLETE_FLASH_MS);
  }
});

onMounted(async () => {
  await loadOrbOpacity();
  await appWindow.setSize(new LogicalSize(COLLAPSED_WINDOW_SIZE, COLLAPSED_WINDOW_SIZE));
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
  <div class="fl-root" :data-state="visualState" :class="{ 'is-expanded': expanded }" :style="rootStyle">
    <!-- ===== 收缩态 ===== -->
    <div
      v-if="!expanded"
      class="fl-orb"
      role="button"
      tabindex="0"
      :aria-label="isRunning || isPaused ? '展开专注控制面板，双击暂停或继续' : '展开 FocusLab 悬浮面板'"
      @mousedown="onMouseDown"
      @click="onOrbClick"
      @contextmenu="openContextMenu"
      @keydown.enter.prevent="onOrbClick"
      @keydown.space.prevent="onOrbClick"
      @dblclick.prevent
    >
      <div class="fl-orb-inner">
        <Check v-if="visualState === 'complete'" :size="22" :stroke-width="2.4" />
        <span v-else-if="visualState === 'breathing'" class="fl-logo">F</span>
        <span v-else class="fl-time">{{ timeText }}</span>
      </div>
    </div>

    <!-- ===== Mini Panel(对齐原型 op-panel) ===== -->
    <div v-else class="fl-mp">
      <div class="fl-mp-head" @mousedown="onMouseDown">
        <div class="fl-mp-phase">
          <span class="fl-mp-dot">
            <component :is="phaseIcon" :size="13" :stroke-width="2.2" />
          </span>
          <span>{{ phaseLabel }}</span>
        </div>
        <button class="fl-mp-icon-btn" type="button" aria-label="收起悬浮面板" @click="expanded = false">
          <X :size="15" />
        </button>
      </div>
      <div class="fl-mp-body">
        <div class="fl-mp-kicker">当前专注</div>
        <div class="fl-mp-task" :title="taskName">{{ taskName }}</div>
        <div class="fl-mp-meta">{{ focusedTime }}</div>
        <div class="fl-mp-ring">
          <svg viewBox="0 0 120 120">
            <circle cx="60" cy="60" r="50" fill="none" stroke="var(--color-border,#e5e7eb)" stroke-width="6" />
            <circle v-if="!isFree" cx="60" cy="60" r="50" fill="none"
              stroke="var(--color-primary,#4f8cff)" stroke-width="6" stroke-linecap="round"
              :stroke-dasharray="PANEL_RING_CIRCUMFERENCE" :stroke-dashoffset="PANEL_RING_CIRCUMFERENCE * (1 - progress)"
              style="transform:rotate(-90deg);transform-origin:50% 50%;transition:stroke-dashoffset .3s" />
          </svg>
          <div class="fl-mp-ring-center">{{ timeText || "--:--" }}</div>
        </div>
        <div class="fl-mp-ctrl">
          <button v-if="!isIdle" class="fl-mp-cbtn" type="button" title="结束" aria-label="结束当前计时" @click="onAbandon">
            <Square :size="15" />
          </button>
          <button
            v-if="isRunning||isPaused"
            class="fl-mp-cbtn fl-mp-main"
            type="button"
            :aria-label="isPaused ? '继续计时' : '暂停计时'"
            @click="togglePause"
          >
            <component :is="isPaused ? Play : Pause" :size="18" :fill="isPaused ? 'currentColor' : 'none'" />
          </button>
          <button v-if="isBreak" class="fl-mp-cbtn" type="button" title="跳过" aria-label="跳过休息" @click="onSkipBreak">
            <SkipForward :size="16" />
          </button>
        </div>
        <div v-if="otherTasks.length && !isIdle" class="fl-mp-switch">
          <div class="fl-mp-sw-label">候选任务</div>
          <div v-for="t in otherTasks" :key="t.id" class="fl-mp-sw-item">
            <span class="fl-mp-sw-dot" />
            <span class="fl-mp-sw-name">{{ t.name }}</span>
          </div>
        </div>
      </div>
      <div class="fl-mp-foot">
        <button type="button" @click="focusMainWindow">
          <Home :size="14" />
          <span>主窗口</span>
        </button>
        <button type="button" class="fl-mp-foot-close" @click="closeBubble">
          <X :size="14" />
          <span>关闭</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fl-root {
  width: 100%;
  height: 100%;
  box-sizing: border-box;
  display: flex;
  align-items: center;
  justify-content: center;
}
.fl-root:not(.is-expanded) {
  border-radius: 50%;
  overflow: hidden;
  opacity: var(--fl-bubble-opacity, 1);
  transition: opacity var(--dur-fast, 150ms) var(--ease-smooth, ease);
}

/* ===== 圆球 ===== */
.fl-orb {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  position: relative;
  cursor: pointer;
  user-select: none;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 0;
  outline: none;
  overflow: hidden;
  transition:
    transform var(--dur-fast, 150ms) var(--ease-smooth, ease),
    filter var(--dur-fast, 150ms) var(--ease-smooth, ease);
}
.fl-orb:hover {
  filter: brightness(1.04);
}
.fl-orb:focus-visible,
.fl-mp-icon-btn:focus-visible,
.fl-mp-cbtn:focus-visible,
.fl-mp-foot button:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--color-primary, #4f8cff) 72%, white);
  outline-offset: 2px;
}

[data-state="breathing"] .fl-orb {
  background: linear-gradient(145deg, var(--color-primary, #4f8cff), var(--color-primary-light, #7aabff));
  animation: breathe 4s ease-in-out infinite;
}
@keyframes breathe {
  0%, 100% { opacity: 0.82; }
  50% { opacity: 1; }
}
[data-state="focusing"] .fl-orb {
  background: linear-gradient(145deg, var(--color-primary-dark, #3366cc), var(--color-primary, #4f8cff));
}
[data-state="free"] .fl-orb {
  background: linear-gradient(145deg, #7b68ae, #a090cc);
}
[data-state="break"] .fl-orb {
  background: linear-gradient(145deg, #5d8a6a, #82ad8e);
}
[data-state="complete"] .fl-orb {
  background: linear-gradient(145deg, #52c41a, #95de64);
  animation: flash 0.6s ease-out 2;
}
@keyframes flash {
  50% {
    filter: brightness(1.12);
  }
}

.fl-orb-inner { position: relative; z-index: 1; text-align: center; pointer-events: none; color: #fff; }
.fl-logo { font-size: 22px; font-weight: 700; }
.fl-time { font-family: ui-monospace, monospace; font-size: 13px; font-weight: 700; letter-spacing: 0; }

/* ===== Mini Panel · 浅色(对齐原型) ===== */
.fl-mp {
  width: 100%;
  height: 100%;
  background: var(--color-bg-elevated, #fff);
  border-radius: 16px;
  border: 1px solid color-mix(in srgb, var(--color-border, #e5e7eb) 86%, white);
  box-shadow:
    0 18px 42px rgba(20, 28, 48, 0.18),
    0 2px 8px rgba(20, 28, 48, 0.08);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  color: var(--color-text-primary, #1a1a2e);
}

.fl-mp-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 12px 10px 14px;
  border-bottom: 1px solid color-mix(in srgb, var(--color-border, #e5e7eb) 72%, transparent);
  cursor: grab;
}
.fl-mp-phase {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  font-size: 12px;
  font-weight: 650;
  color: var(--color-primary, #4f8cff);
}
.fl-mp-dot {
  width: 24px;
  height: 24px;
  border-radius: 999px;
  display: grid;
  place-items: center;
  background: color-mix(in srgb, var(--color-primary, #4f8cff) 12%, transparent);
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-primary, #4f8cff) 18%, transparent);
}
.fl-mp-icon-btn {
  width: 28px;
  height: 28px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--color-text-muted, #999);
  cursor: pointer;
  display: grid;
  place-items: center;
  transition:
    background var(--dur-fast, 150ms) var(--ease-smooth, ease),
    color var(--dur-fast, 150ms) var(--ease-smooth, ease);
}
.fl-mp-icon-btn:hover { background: var(--color-bg-hover, #f5f5f5); color: var(--color-text-primary, #1a1a2e); }

.fl-mp-body {
  padding: 18px 20px 14px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  flex: 1;
  justify-content: center;
}
.fl-mp-kicker {
  font-size: 11px;
  font-weight: 650;
  color: var(--color-text-muted, #999);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}
.fl-mp-task {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 16px;
  font-weight: 700;
  text-align: center;
  color: var(--color-text-primary, #1a1a2e);
}
.fl-mp-meta {
  min-height: 18px;
  font-size: 12px;
  color: var(--color-text-muted, #999);
}

/* 环 */
.fl-mp-ring {
  width: 128px;
  height: 128px;
  position: relative;
  margin: 6px 0 4px;
}
.fl-mp-ring svg { width: 100%; height: 100%; }
.fl-mp-ring-center {
  position: absolute; inset: 0;
  display: flex; align-items: center; justify-content: center;
  font-family: ui-monospace, monospace;
  font-size: 24px; font-weight: 700;
  letter-spacing: 0;
  color: var(--color-text-primary, #1a1a2e);
}

/* 控制(圆形按钮) */
.fl-mp-ctrl { display: flex; gap: 8px; align-items: center; justify-content: center; }
.fl-mp-cbtn {
  width: 38px;
  height: 38px;
  border-radius: 50%;
  background: var(--color-bg-hover, #f5f5f5);
  border: 1px solid color-mix(in srgb, var(--color-border, #e5e7eb) 68%, transparent);
  cursor: pointer;
  color: var(--color-text-secondary, #666);
  display: grid;
  place-items: center;
  transition:
    transform var(--dur-fast, 150ms) var(--ease-smooth, ease),
    background var(--dur-fast, 150ms) var(--ease-smooth, ease),
    color var(--dur-fast, 150ms) var(--ease-smooth, ease),
    border-color var(--dur-fast, 150ms) var(--ease-smooth, ease);
}
.fl-mp-cbtn:hover {
  background: color-mix(in srgb, var(--color-primary,#4f8cff) 12%, transparent);
  border-color: color-mix(in srgb, var(--color-primary,#4f8cff) 32%, transparent);
  color: var(--color-primary,#4f8cff);
  transform: translateY(-1px);
}
.fl-mp-main {
  width: 46px;
  height: 46px;
  background: var(--color-primary,#4f8cff);
  border-color: transparent;
  color: #fff;
  box-shadow: 0 8px 18px color-mix(in srgb, var(--color-primary,#4f8cff) 28%, transparent);
}
.fl-mp-main:hover { transform: translateY(-1px); background: var(--color-primary,#4f8cff); color: #fff; }

/* 快速切换 */
.fl-mp-switch {
  width: 100%;
  margin-top: 6px;
  padding-top: 10px;
  border-top: 1px solid color-mix(in srgb, var(--color-border,#e5e7eb) 72%, transparent);
}
.fl-mp-sw-label {
  margin-bottom: 6px;
  font-size: 11px;
  font-weight: 650;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--color-text-muted,#999);
}
.fl-mp-sw-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 0;
  border-radius: 6px;
  font-size: 13px;
  color: var(--color-text-secondary, #666);
}
.fl-mp-sw-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--color-primary,#4f8cff); flex-shrink: 0; }
.fl-mp-sw-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

/* 底部 */
.fl-mp-foot {
  display: flex;
  gap: 8px;
  padding: 10px;
  border-top: 1px solid color-mix(in srgb, var(--color-border, #e5e7eb) 72%, transparent);
  background: color-mix(in srgb, var(--color-bg-subtle, #fafafa) 78%, transparent);
}
.fl-mp-foot button {
  flex: 1;
  min-height: 34px;
  border: 0;
  border-radius: 9px;
  background: transparent;
  cursor: pointer;
  color: var(--color-text-muted, #999);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 650;
  transition:
    background var(--dur-fast, 150ms) var(--ease-smooth, ease),
    color var(--dur-fast, 150ms) var(--ease-smooth, ease);
}
.fl-mp-foot button:hover {
  background: color-mix(in srgb, var(--color-primary, #4f8cff) 10%, transparent);
  color: var(--color-primary, #4f8cff);
}
.fl-mp-foot-close:hover {
  background: color-mix(in srgb, #ef4444 10%, transparent) !important;
  color: #ef4444 !important;
}

@media (prefers-reduced-motion: reduce) {
  .fl-orb,
  [data-state="breathing"] .fl-orb,
  [data-state="complete"] .fl-orb {
    animation: none;
    transition: none;
  }
}
</style>

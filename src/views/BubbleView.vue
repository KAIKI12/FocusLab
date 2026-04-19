<script setup lang="ts">
/**
 * BubbleView · 悬浮球 — 自定义拖拽 + 真圆形 + 手势修正。
 *
 * 拖拽: mousedown 记录起点 → mousemove 算 delta → setPosition (不用 startDragging)
 * 手势: 单击暂停/继续, 双击展开面板
 * 圆形: 窗口 64px, 内容 CSS 圆形, body 透明
 */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";

import type { TimerSnapshot, TimerStatus } from "@/types";

const snapshot = ref<TimerSnapshot | null>(null);
const expanded = ref(false);
const taskName = ref("FocusLab");
const showComplete = ref(false);
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
  if (showComplete.value) return "完成!";
  const s = snapshot.value;
  if (!s || s.status === "idle") return "FocusLab";
  if (s.mode === "free") return "🌀 自由计时";
  if (s.status === "running") return "🍅 专注中";
  if (s.status === "paused") return "⏸ 已暂停";
  if (s.status === "break") return "☕ 休息中";
  if (s.status === "break_ended") return "☕ 休息结束";
  return "";
});

// =============================================
// 自定义拖拽 — 不用 startDragging / drag-region
// =============================================

let dragStartScreenX = 0;
let dragStartScreenY = 0;
let dragStartWinX = 0;
let dragStartWinY = 0;
let isDragging = false;
let hasMoved = false;

async function onMouseDown(e: MouseEvent) {
  // 记录鼠标屏幕坐标和窗口位置
  dragStartScreenX = e.screenX;
  dragStartScreenY = e.screenY;
  isDragging = true;
  hasMoved = false;

  try {
    const pos = await appWindow.outerPosition();
    dragStartWinX = pos.x;
    dragStartWinY = pos.y;
  } catch { /* */ }
}

async function onMouseMove(e: MouseEvent) {
  if (!isDragging) return;

  const dx = e.screenX - dragStartScreenX;
  const dy = e.screenY - dragStartScreenY;

  // 超过 3px 才算拖拽（区分点击和拖拽）
  if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
    hasMoved = true;
  }

  if (hasMoved) {
    const newX = dragStartWinX + dx;
    const newY = dragStartWinY + dy;
    try {
      await appWindow.setPosition(new LogicalPosition(newX, newY));
    } catch { /* */ }
  }
}

function onMouseUp() {
  isDragging = false;
  // 拖拽结束保存位置
  if (hasMoved) {
    savePosition();
  }
}

async function savePosition() {
  try {
    const pos = await appWindow.outerPosition();
    localStorage.setItem("fl-bubble-pos", JSON.stringify({ x: pos.x, y: pos.y }));
  } catch { /* */ }
}

// 全局 mousemove/mouseup (挂在 document 上，防止鼠标移出窗口丢失)
onMounted(() => {
  document.addEventListener("mousemove", onMouseMove);
  document.addEventListener("mouseup", onMouseUp);
});
onUnmounted(() => {
  document.removeEventListener("mousemove", onMouseMove);
  document.removeEventListener("mouseup", onMouseUp);
});

// =============================================
// 手势：单击暂停/继续，双击展开面板
// =============================================

let clickTimer: ReturnType<typeof setTimeout> | null = null;
let clickCount = 0;

function onOrbClick() {
  if (hasMoved) return; // 拖拽后不触发点击
  clickCount++;
  if (clickCount === 1) {
    clickTimer = setTimeout(() => {
      onSingleClick();
      clickCount = 0;
    }, 250);
  } else if (clickCount === 2) {
    if (clickTimer) clearTimeout(clickTimer);
    clickCount = 0;
    expanded.value = !expanded.value;
  }
}

async function onSingleClick() {
  if (isRunning.value || isPaused.value) {
    await togglePause();
  } else if (isIdle.value) {
    await focusMainWindow();
  }
}

// =============================================
// 操作
// =============================================

async function togglePause() {
  if (!snapshot.value) return;
  try {
    if (snapshot.value.status === "running") await invoke("pause_timer");
    else if (snapshot.value.status === "paused") await invoke("resume_timer");
  } catch (e) { console.error(e); }
}

async function onAbandon() {
  try { await invoke("abandon_timer", { reason: null }); } catch (e) { console.error(e); }
}

async function onSkipBreak() {
  try { await invoke("skip_break"); } catch (e) { console.error(e); }
}

async function focusMainWindow() {
  try {
    const main = await WebviewWindow.getByLabel("main");
    if (main) { await main.unminimize(); await main.setFocus(); }
  } catch (e) { console.error(e); }
}

async function closeBubble() { await appWindow.close(); }

// =============================================
// 窗口大小
// =============================================

const WIN_ORB = 64;
const PANEL_W = 320;
const PANEL_H = 380;

watch(expanded, async (exp) => {
  if (exp) {
    try {
      const pos = await appWindow.outerPosition();
      const sW = window.screen.width;
      const sH = window.screen.height;
      let x = Math.max(0, Math.min(pos.x, sW - PANEL_W));
      let y = Math.max(0, Math.min(pos.y, sH - PANEL_H));
      await appWindow.setPosition(new LogicalPosition(x, y));
    } catch { /* */ }
    await appWindow.setSize(new LogicalSize(PANEL_W, PANEL_H));
  } else {
    await appWindow.setSize(new LogicalSize(WIN_ORB, WIN_ORB));
    savePosition();
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
}

watch(() => snapshot.value?.taskId, () => refreshTaskName());

watch(
  () => snapshot.value?.status,
  (newSt, oldSt) => {
    if (oldSt === "running" && (newSt === "break" || newSt === "break_ended")) {
      showComplete.value = true;
      setTimeout(() => { showComplete.value = false; }, 1200);
    }
  },
);

onMounted(async () => {
  await appWindow.setSize(new LogicalSize(WIN_ORB, WIN_ORB));

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
  } catch { /* idle */ }

  await refreshTaskName();

  const handler = (ev: { payload: TimerSnapshot }) => { snapshot.value = ev.payload; };
  const u1 = await listen<TimerSnapshot>("timer:tick", handler);
  const u2 = await listen<TimerSnapshot>("timer:state_changed", handler);
  unlisteners = [u1, u2];
});

onUnmounted(() => { unlisteners.forEach((fn) => fn()); });
</script>

<template>
  <div class="fl-root" :data-state="visualState">
    <!-- 收缩态 -->
    <div
      v-if="!expanded"
      class="fl-orb"
      @mousedown="onMouseDown"
      @click="onOrbClick"
      @dblclick.prevent
    >
      <svg v-if="visualState === 'focusing'" class="fl-orb-ring" viewBox="0 0 64 64">
        <circle class="fl-ring-track" cx="32" cy="32" r="28" fill="none" stroke-width="3" />
        <circle class="fl-ring-arc" cx="32" cy="32" r="28" fill="none" stroke-width="3"
          stroke-linecap="round" :stroke-dasharray="176" :stroke-dashoffset="176 * progress" />
      </svg>
      <div v-if="visualState === 'free'" class="fl-orb-free-ring" />
      <div class="fl-orb-center">
        <span v-if="visualState === 'complete'" class="fl-orb-check">✓</span>
        <span v-else-if="visualState === 'breathing'" class="fl-orb-logo">F</span>
        <span v-else class="fl-orb-time">{{ timeText }}</span>
      </div>
    </div>

    <!-- 展开态 -->
    <div v-else class="fl-panel">
      <div class="fl-panel-head" @mousedown="onMouseDown">
        <span class="fl-panel-phase">{{ phaseLabel }}</span>
        <button class="fl-panel-close" @click="expanded = false">×</button>
      </div>
      <div class="fl-panel-body">
        <div class="fl-panel-task">{{ taskName }}</div>
        <div v-if="snapshot && !isIdle" class="fl-panel-meta">
          <span v-if="snapshot.mode === 'pomodoro'">🍅 第 {{ snapshot.pomodoroCount }} 个</span>
          <span v-else>🌀 自由模式</span>
          <span v-if="isPaused" class="fl-panel-paused">已暂停</span>
        </div>
        <div class="fl-panel-ring-wrap">
          <svg class="fl-panel-ring" viewBox="0 0 130 130">
            <circle class="fl-pring-track" cx="65" cy="65" r="56" fill="none" stroke-width="6" />
            <circle v-if="!isFree" class="fl-pring-arc" cx="65" cy="65" r="56" fill="none"
              stroke-width="6" stroke-linecap="round" :stroke-dasharray="351.9"
              :stroke-dashoffset="351.9 * progress" />
          </svg>
          <div class="fl-panel-ring-center">
            <span class="fl-pring-time">{{ timeText || "--:--" }}</span>
          </div>
        </div>
        <div class="fl-panel-ctrl">
          <button v-if="isRunning" class="fl-ctrl-btn" @click="togglePause">⏸ 暂停</button>
          <button v-else-if="isPaused" class="fl-ctrl-btn fl-ctrl-primary" @click="togglePause">▶ 继续</button>
          <button v-if="isBreak" class="fl-ctrl-btn" @click="onSkipBreak">⏭ 跳过</button>
          <button v-if="!isIdle" class="fl-ctrl-btn fl-ctrl-danger" @click="onAbandon">✕ 放弃</button>
        </div>
      </div>
      <div class="fl-panel-foot">
        <button class="fl-foot-btn" @click="focusMainWindow">🏠 主窗口</button>
        <button class="fl-foot-btn fl-foot-close" @click="closeBubble">✕ 关闭</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 关键：根容器裁剪为圆形，收缩态时隐藏方角 */
.fl-root {
  width: 100%; height: 100%;
  display: flex; align-items: center; justify-content: center;
}
/* 收缩态时整个根元素裁剪为圆 */
.fl-root:not(:has(.fl-panel)) {
  border-radius: 50%;
  overflow: hidden;
}

.fl-orb {
  width: 64px; height: 64px;
  border-radius: 50%;
  position: relative;
  cursor: pointer;
  user-select: none;
  display: flex; align-items: center; justify-content: center;
}

/* 呼吸态 */
[data-state="breathing"] .fl-orb {
  background: linear-gradient(135deg, #4f8cff, #7aabff);
  box-shadow: 0 4px 14px rgba(79,140,255,0.35);
  animation: breathe 4s ease-in-out infinite;
}
@keyframes breathe {
  0%,100% { opacity: 0.8; transform: scale(1); }
  50% { opacity: 1; transform: scale(1.03); }
}
[data-state="focusing"] .fl-orb {
  background: linear-gradient(135deg, #4f8cff, #3b7dff);
  box-shadow: 0 4px 16px rgba(79,140,255,0.4);
}
[data-state="free"] .fl-orb {
  background: linear-gradient(135deg, #8b5cf6, #a78bfa);
  box-shadow: 0 4px 16px rgba(139,92,246,0.45);
}
[data-state="break"] .fl-orb {
  background: linear-gradient(135deg, #52c41a, #73d13d);
  box-shadow: 0 4px 16px rgba(82,196,26,0.4);
}
[data-state="complete"] .fl-orb {
  background: linear-gradient(135deg, #52c41a, #95de64);
  animation: flashGreen 0.6s ease-out 2;
}
@keyframes flashGreen {
  0%,100% { box-shadow: 0 4px 16px rgba(82,196,26,0.4); }
  50% { box-shadow: 0 4px 28px rgba(82,196,26,0.8), 0 0 0 6px rgba(82,196,26,0.2); }
}
.fl-orb:hover { transform: scale(1.06); }

.fl-orb-ring {
  position: absolute; inset: 0; width: 64px; height: 64px;
  transform: rotate(-90deg);
}
.fl-ring-track { stroke: rgba(255,255,255,0.2); }
.fl-ring-arc { stroke: rgba(255,255,255,0.85); transition: stroke-dashoffset 0.3s ease; }

.fl-orb-free-ring {
  position: absolute; inset: -2px;
  width: calc(100% + 4px); height: calc(100% + 4px);
  border-radius: 50%; border: 2px dashed rgba(255,255,255,0.6);
  animation: spin12 12s linear infinite;
}
@keyframes spin12 { to { transform: rotate(360deg); } }

.fl-orb-center { position: relative; z-index: 1; text-align: center; pointer-events: none; }
.fl-orb-time {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 13px; font-weight: 600; color: #fff;
  letter-spacing: -0.5px; font-variant-numeric: tabular-nums;
}
.fl-orb-logo { font-size: 22px; font-weight: 700; color: #fff; }
.fl-orb-check { font-size: 24px; color: #fff; }

/* ===== Mini Panel ===== */
.fl-panel {
  width: 310px; height: 370px;
  background: rgba(24,24,28,0.94); backdrop-filter: blur(20px);
  border-radius: 16px; border: 1px solid rgba(255,255,255,0.08);
  box-shadow: 0 12px 40px rgba(0,0,0,0.5);
  display: flex; flex-direction: column; color: #fff; overflow: hidden;
}
.fl-panel-head {
  display: flex; align-items: center; justify-content: space-between;
  padding: 10px 14px; border-bottom: 1px solid rgba(255,255,255,0.06);
  cursor: grab;
}
.fl-panel-phase { font-size: 13px; font-weight: 500; opacity: 0.9; }
.fl-panel-close {
  background: none; border: none; color: rgba(255,255,255,0.4);
  font-size: 20px; cursor: pointer; padding: 0 4px; line-height: 1;
}
.fl-panel-close:hover { color: #fff; }

.fl-panel-body {
  flex: 1; display: flex; flex-direction: column;
  align-items: center; padding: 10px 14px; gap: 6px;
}
.fl-panel-task {
  font-size: 14px; font-weight: 600; text-align: center;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 100%;
}
.fl-panel-meta { font-size: 11px; color: rgba(255,255,255,0.5); display: flex; gap: 8px; }
.fl-panel-paused { color: #fbbf24; }

.fl-panel-ring-wrap { position: relative; width: 120px; height: 120px; }
.fl-panel-ring { width: 100%; height: 100%; transform: rotate(-90deg); }
.fl-pring-track { stroke: rgba(255,255,255,0.1); }
.fl-pring-arc { stroke: rgba(255,255,255,0.75); transition: stroke-dashoffset 0.3s ease; }
[data-state="focusing"] .fl-pring-arc { stroke: #4f8cff; }
[data-state="free"] .fl-pring-arc { stroke: #8b5cf6; }
[data-state="break"] .fl-pring-arc { stroke: #52c41a; }
.fl-panel-ring-center {
  position: absolute; inset: 0; display: flex; align-items: center; justify-content: center;
}
.fl-pring-time {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 24px; font-weight: 600; font-variant-numeric: tabular-nums;
}

.fl-panel-ctrl { display: flex; gap: 6px; width: 100%; }
.fl-ctrl-btn {
  flex: 1; padding: 7px 8px; border-radius: 8px;
  border: 1px solid rgba(255,255,255,0.15); background: rgba(255,255,255,0.08);
  color: #fff; font-size: 12px; cursor: pointer; transition: background 0.15s;
}
.fl-ctrl-btn:hover { background: rgba(255,255,255,0.16); }
.fl-ctrl-primary { background: rgba(79,140,255,0.3); border-color: rgba(79,140,255,0.4); }
.fl-ctrl-danger { border-color: rgba(239,68,68,0.3); color: #fca5a5; }
.fl-ctrl-danger:hover { background: rgba(239,68,68,0.15); }

.fl-panel-foot { display: flex; border-top: 1px solid rgba(255,255,255,0.06); }
.fl-foot-btn {
  flex: 1; padding: 10px; background: none; border: none;
  color: rgba(255,255,255,0.55); font-size: 11px; cursor: pointer;
}
.fl-foot-btn:hover { color: #fff; }
.fl-foot-close:hover { color: #fca5a5; }
.fl-foot-btn + .fl-foot-btn { border-left: 1px solid rgba(255,255,255,0.06); }
</style>

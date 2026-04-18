<script setup lang="ts">
/**
 * BubbleView · 悬浮球窗口 — 四态视觉 + 双击手势 + Mini Panel。
 *
 * 对齐 prototype/screens/floating-ball.html 完整设计:
 *
 * 四态:
 *   breathing(idle): 64px 圆,呼吸动画(opacity 0.75↔1.0, 4s)
 *   focusing(pomodoro): 实心蓝,SVG conic ring 倒计时弧,中心 MM:SS
 *   free-focusing: 紫色 #8B5CF6,虚线环 12s 旋转,正计时
 *   complete: 绿色闪光 0.6s×2(状态迁移时短暂触发)
 *
 * 手势:
 *   单击: 展开/收缩 Mini Panel
 *   双击: running/paused → 暂停/继续; idle → 打开主窗口
 *   拖拽: data-tauri-drag-region
 *
 * Mini Panel:
 *   任务名 + 进度环 + 控制按钮 + [打开主窗口][关闭悬浮球]
 */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { LogicalSize } from "@tauri-apps/api/dpi";

import type { TimerSnapshot, TimerStatus } from "@/types";

// ---------- 状态 ----------

const snapshot = ref<TimerSnapshot | null>(null);
const expanded = ref(false);
const taskName = ref("FocusLab");
const showComplete = ref(false);
let unlisteners: UnlistenFn[] = [];

const appWindow = getCurrentWindow();

const isIdle = computed(
  () => !snapshot.value || snapshot.value.status === "idle",
);
const isRunning = computed(() => snapshot.value?.status === "running");
const isPaused = computed(() => snapshot.value?.status === "paused");
const isBreak = computed(
  () =>
    snapshot.value?.status === "break" ||
    snapshot.value?.status === "break_ended",
);
const isFree = computed(() => snapshot.value?.mode === "free");

/** 视觉态枚举(驱动 CSS) */
const visualState = computed(() => {
  if (showComplete.value) return "complete";
  const s = snapshot.value;
  if (!s || s.status === "idle") return "breathing";
  if (s.mode === "free" && (s.status === "running" || s.status === "paused"))
    return "free";
  if (s.status === "running" || s.status === "paused") return "focusing";
  if (s.status === "break" || s.status === "break_ended") return "break";
  return "breathing";
});

// ---------- 时间文本 ----------

const timeText = computed(() => {
  const s = snapshot.value;
  if (!s || s.status === "idle") return "--:--";
  if (s.mode === "free") {
    const total = s.elapsedSeconds;
    const m = Math.floor(total / 60);
    const sec = total % 60;
    return `${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
  }
  const remaining = Math.max(0, s.plannedSeconds - s.elapsedSeconds);
  const m = Math.floor(remaining / 60);
  const sec = remaining % 60;
  return `${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
});

/** 进度比例 0→1(compact ring 用) */
const progress = computed(() => {
  const s = snapshot.value;
  if (!s || s.plannedSeconds <= 0) return 0;
  return Math.min(1, Math.max(0, s.elapsedSeconds / s.plannedSeconds));
});

// SVG ring 常量:viewBox 0 0 72 72, r=30, circumference=188.5
const RING_CIRCUM = 188.5;
/** 倒计时:开始(progress=0)满环→结束(progress=1)空环 */
const ringOffset = computed(() => RING_CIRCUM * progress.value);

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

// ---------- 双击检测 ----------

let clickTimer: ReturnType<typeof setTimeout> | null = null;
let clickCount = 0;

function onOrbClick() {
  clickCount++;
  if (clickCount === 1) {
    clickTimer = setTimeout(() => {
      // 单击 → 展开/收缩
      expanded.value = !expanded.value;
      clickCount = 0;
    }, 280);
  } else if (clickCount === 2) {
    if (clickTimer) clearTimeout(clickTimer);
    clickCount = 0;
    onDoubleClick();
  }
}

async function onDoubleClick() {
  if (isRunning.value || isPaused.value) {
    await togglePause();
  } else if (isIdle.value) {
    await focusMainWindow();
  }
}

// ---------- 操作 ----------

async function togglePause() {
  if (!snapshot.value) return;
  try {
    if (snapshot.value.status === "running") {
      await invoke("pause_timer");
    } else if (snapshot.value.status === "paused") {
      await invoke("resume_timer");
    }
  } catch (e) {
    console.error("[bubble] toggle pause failed", e);
  }
}

async function onAbandon() {
  try {
    await invoke("abandon_timer", { reason: null });
  } catch (e) {
    console.error("[bubble] abandon failed", e);
  }
}

async function onSkipBreak() {
  try {
    await invoke("skip_break");
  } catch (e) {
    console.error("[bubble] skip_break failed", e);
  }
}

async function focusMainWindow() {
  try {
    const main = await WebviewWindow.getByLabel("main");
    if (main) {
      await main.unminimize();
      await main.setFocus();
    }
  } catch (e) {
    console.error("[bubble] focus main failed", e);
  }
}

async function closeBubble() {
  await appWindow.close();
}

// ---------- 窗口大小 ----------

watch(expanded, async (exp) => {
  if (exp) {
    await appWindow.setSize(new LogicalSize(320, 360));
  } else {
    await appWindow.setSize(new LogicalSize(72, 72));
  }
});

// ---------- 任务名获取 ----------

async function refreshTaskName() {
  const id = snapshot.value?.taskId;
  if (!id) {
    taskName.value = "FocusLab";
    return;
  }
  try {
    const name = await invoke<string | null>("get_task_name", { id });
    taskName.value = name ?? "(任务)";
  } catch {
    taskName.value = "(任务)";
  }
}

// 当 taskId 变化时重新获取名字
watch(
  () => snapshot.value?.taskId,
  () => refreshTaskName(),
);

// ---------- 完成闪光检测 ----------

watch(
  () => snapshot.value?.status,
  (newSt, oldSt) => {
    if (
      oldSt === "running" &&
      (newSt === "break" || newSt === "break_ended")
    ) {
      showComplete.value = true;
      setTimeout(() => {
        showComplete.value = false;
      }, 1200);
    }
  },
);

// ---------- 生命周期 ----------

onMounted(async () => {
  // 设初始窗口大小
  await appWindow.setSize(new LogicalSize(72, 72));

  // 拉初始快照
  try {
    const row = await invoke<{
      status: string;
      task_id: string | null;
      session_id: string | null;
      mode: string | null;
      pomodoro_preset: string | null;
      elapsed_seconds: number;
      planned_seconds: number | null;
      pomodoro_count: number;
      is_break: boolean;
      break_remaining: number | null;
    }>("get_timer_state");
    snapshot.value = {
      status: row.status as TimerStatus,
      taskId: row.task_id,
      sessionId: row.session_id,
      mode: row.mode as "pomodoro" | "free" | null,
      preset: row.pomodoro_preset as TimerSnapshot["preset"],
      elapsedSeconds: row.elapsed_seconds,
      plannedSeconds: row.is_break
        ? (row.break_remaining ?? 0)
        : (row.planned_seconds ?? 0),
      pomodoroCount: row.pomodoro_count,
      isBreak: row.is_break,
    };
  } catch {
    /* idle */
  }

  await refreshTaskName();

  // 事件订阅
  const handler = (ev: { payload: TimerSnapshot }) => {
    snapshot.value = ev.payload;
  };
  const u1 = await listen<TimerSnapshot>("timer:tick", handler);
  const u2 = await listen<TimerSnapshot>("timer:state_changed", handler);
  unlisteners = [u1, u2];
});

onUnmounted(() => {
  unlisteners.forEach((fn) => fn());
});
</script>

<template>
  <div class="fl-root" :data-state="visualState">
    <!-- ===== 收缩态:圆形悬浮球 ===== -->
    <div
      v-if="!expanded"
      class="fl-orb"
      @click="onOrbClick"
      @dblclick.prevent
    >
      <!-- 透明拖拽层(不阻止点击穿透) -->
      <div class="fl-orb-drag" data-tauri-drag-region />
      <!-- SVG 进度环(仅 focusing 态) -->
      <svg
        v-if="visualState === 'focusing'"
        class="fl-orb-ring"
        viewBox="0 0 72 72"
      >
        <circle
          class="fl-ring-track"
          cx="36"
          cy="36"
          r="30"
          fill="none"
          stroke-width="4"
        />
        <circle
          class="fl-ring-arc"
          cx="36"
          cy="36"
          r="30"
          fill="none"
          stroke-width="4"
          stroke-linecap="round"
          :stroke-dasharray="RING_CIRCUM"
          :stroke-dashoffset="ringOffset"
        />
      </svg>

      <!-- 虚线旋转环(free 态) -->
      <div v-if="visualState === 'free'" class="fl-orb-free-ring" />

      <!-- 中心文字 -->
      <div class="fl-orb-center">
        <span v-if="visualState === 'complete'" class="fl-orb-check">✓</span>
        <span v-else class="fl-orb-time">{{ timeText }}</span>
      </div>
    </div>

    <!-- ===== 展开态:Mini Panel ===== -->
    <div v-else class="fl-panel" @click.stop>
      <div class="fl-panel-head" data-tauri-drag-region>
        <span class="fl-panel-phase">{{ phaseLabel }}</span>
        <button
          class="fl-panel-close"
          type="button"
          title="收起"
          @click="expanded = false"
        >
          ×
        </button>
      </div>

      <div class="fl-panel-body">
        <div class="fl-panel-task">{{ taskName }}</div>

        <!-- 进度环(120px) -->
        <div class="fl-panel-ring-wrap">
          <svg class="fl-panel-ring" viewBox="0 0 130 130">
            <circle
              class="fl-pring-track"
              cx="65"
              cy="65"
              r="56"
              fill="none"
              stroke-width="6"
            />
            <circle
              v-if="!isFree"
              class="fl-pring-arc"
              cx="65"
              cy="65"
              r="56"
              fill="none"
              stroke-width="6"
              stroke-linecap="round"
              :stroke-dasharray="351.9"
              :stroke-dashoffset="351.9 * progress"
            />
          </svg>
          <div class="fl-panel-ring-text">
            <span class="fl-pring-time">{{ timeText }}</span>
          </div>
        </div>

        <!-- 控制按钮 -->
        <div class="fl-panel-ctrl">
          <button
            v-if="isRunning"
            class="fl-ctrl-btn"
            type="button"
            @click="togglePause"
          >
            ⏸ 暂停
          </button>
          <button
            v-else-if="isPaused"
            class="fl-ctrl-btn fl-ctrl-primary"
            type="button"
            @click="togglePause"
          >
            ▶ 继续
          </button>
          <button
            v-if="isBreak"
            class="fl-ctrl-btn"
            type="button"
            @click="onSkipBreak"
          >
            ⏭ 跳过休息
          </button>
          <button
            v-if="!isIdle"
            class="fl-ctrl-btn fl-ctrl-danger"
            type="button"
            @click="onAbandon"
          >
            ✕ 放弃
          </button>
        </div>
      </div>

      <!-- 底部操作 -->
      <div class="fl-panel-foot">
        <button class="fl-foot-btn" type="button" @click="focusMainWindow">
          🏠 主窗口
        </button>
        <button
          class="fl-foot-btn fl-foot-close"
          type="button"
          @click="closeBubble"
        >
          ✕ 关闭悬浮球
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ===== 根容器(透明窗口) ===== */
.fl-root {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* ===== 圆形悬浮球 ===== */
.fl-orb {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  position: relative;
  cursor: grab;
  user-select: none;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

/* 透明拖拽层,覆盖整个 orb */
.fl-orb-drag {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  z-index: 10;
  -webkit-app-region: drag;
}

/* -- 呼吸态(idle) -- */
[data-state="breathing"] .fl-orb {
  background: linear-gradient(135deg, #4f8cff, #7aabff);
  box-shadow: 0 6px 14px rgba(79, 140, 255, 0.3);
  animation: breathe 4s ease-in-out infinite;
}
@keyframes breathe {
  0%,
  100% {
    opacity: 0.75;
    box-shadow: 0 6px 14px rgba(79, 140, 255, 0.3);
  }
  50% {
    opacity: 1;
    box-shadow: 0 10px 24px rgba(79, 140, 255, 0.5);
  }
}

/* -- 专注态(pomodoro) -- */
[data-state="focusing"] .fl-orb {
  background: linear-gradient(135deg, #4f8cff, #3b7dff);
  box-shadow: 0 8px 20px rgba(79, 140, 255, 0.35);
}
[data-state="focusing"] .fl-orb:hover {
  transform: scale(1.05);
}

/* -- 自由态 -- */
[data-state="free"] .fl-orb {
  background: linear-gradient(135deg, #8b5cf6, #a78bfa);
  box-shadow: 0 8px 20px rgba(139, 92, 246, 0.4);
}

/* -- 休息态 -- */
[data-state="break"] .fl-orb {
  background: linear-gradient(135deg, #52c41a, #73d13d);
  box-shadow: 0 8px 20px rgba(82, 196, 26, 0.35);
}

/* -- 完成闪光 -- */
[data-state="complete"] .fl-orb {
  background: linear-gradient(135deg, #52c41a, #95de64);
  animation: flashGreen 0.6s ease-out 2;
}
@keyframes flashGreen {
  0%,
  100% {
    box-shadow: 0 8px 24px rgba(82, 196, 26, 0.45);
  }
  50% {
    box-shadow:
      0 8px 32px rgba(82, 196, 26, 0.8),
      0 0 0 6px rgba(82, 196, 26, 0.25);
  }
}

/* -- SVG 进度环(focusing) -- */
.fl-orb-ring {
  position: absolute;
  inset: -4px;
  width: calc(100% + 8px);
  height: calc(100% + 8px);
  transform: rotate(-90deg);
}
.fl-ring-track {
  stroke: rgba(255, 255, 255, 0.2);
}
.fl-ring-arc {
  stroke: rgba(255, 255, 255, 0.85);
  transition: stroke-dashoffset 0.3s ease;
}

/* -- 虚线旋转环(free) -- */
.fl-orb-free-ring {
  position: absolute;
  inset: -4px;
  width: calc(100% + 8px);
  height: calc(100% + 8px);
  border-radius: 50%;
  border: 2px dashed rgba(255, 255, 255, 0.7);
  animation: freeOrbSpin 12s linear infinite;
}
@keyframes freeOrbSpin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

/* -- 中心内容 -- */
.fl-orb-center {
  position: relative;
  z-index: 1;
  text-align: center;
}
.fl-orb-time {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 13px;
  font-weight: 600;
  color: #fff;
  letter-spacing: -0.5px;
  font-variant-numeric: tabular-nums;
}
.fl-orb-check {
  font-size: 24px;
  color: #fff;
}

/* ===== Mini Panel(展开态) ===== */
.fl-panel {
  width: 310px;
  height: 350px;
  background: rgba(24, 24, 28, 0.92);
  backdrop-filter: blur(20px);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
  display: flex;
  flex-direction: column;
  color: #fff;
  overflow: hidden;
}

.fl-panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  cursor: grab;
}
.fl-panel-phase {
  font-size: 13px;
  font-weight: 500;
  opacity: 0.9;
}
.fl-panel-close {
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.4);
  font-size: 20px;
  cursor: pointer;
  padding: 0 4px;
  line-height: 1;
}
.fl-panel-close:hover {
  color: #fff;
}

.fl-panel-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 16px;
  gap: 8px;
}

.fl-panel-task {
  font-size: 14px;
  font-weight: 600;
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}

/* -- Panel 进度环 -- */
.fl-panel-ring-wrap {
  position: relative;
  width: 120px;
  height: 120px;
}
.fl-panel-ring {
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
}
.fl-pring-track {
  stroke: rgba(255, 255, 255, 0.1);
}
.fl-pring-arc {
  stroke: rgba(255, 255, 255, 0.75);
  transition: stroke-dashoffset 0.3s ease;
}

/* 按 state 上色 */
[data-state="focusing"] .fl-pring-arc {
  stroke: #4f8cff;
}
[data-state="free"] .fl-pring-arc {
  stroke: #8b5cf6;
}
[data-state="break"] .fl-pring-arc {
  stroke: #52c41a;
}

.fl-panel-ring-text {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}
.fl-pring-time {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 24px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

/* -- 控制按钮 -- */
.fl-panel-ctrl {
  display: flex;
  gap: 6px;
  width: 100%;
}
.fl-ctrl-btn {
  flex: 1;
  padding: 7px 8px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  background: rgba(255, 255, 255, 0.08);
  color: #fff;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s;
}
.fl-ctrl-btn:hover {
  background: rgba(255, 255, 255, 0.16);
}
.fl-ctrl-primary {
  background: rgba(79, 140, 255, 0.3);
  border-color: rgba(79, 140, 255, 0.4);
}
.fl-ctrl-danger {
  border-color: rgba(239, 68, 68, 0.3);
  color: #fca5a5;
}
.fl-ctrl-danger:hover {
  background: rgba(239, 68, 68, 0.15);
}

/* -- 底部 -- */
.fl-panel-foot {
  display: flex;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}
.fl-foot-btn {
  flex: 1;
  padding: 10px;
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.55);
  font-size: 11px;
  cursor: pointer;
  transition: color 0.15s;
}
.fl-foot-btn:hover {
  color: #fff;
}
.fl-foot-close:hover {
  color: #fca5a5;
}
.fl-foot-btn + .fl-foot-btn {
  border-left: 1px solid rgba(255, 255, 255, 0.06);
}
</style>

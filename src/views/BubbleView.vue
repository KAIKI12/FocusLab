<script setup lang="ts">
/**
 * BubbleView · 悬浮球窗口的唯一视图。
 *
 * 两种形态:
 *   - 收缩: 48px 圆形 · 任务名(截断) + mm:ss
 *   - 展开: ~200×140 mini-panel · 完整计时信息 + [暂停/继续][放弃] 按钮
 *
 * 订阅 timer:tick / timer:state_changed 事件更新。
 * 窗口本身透明无边框,组件自绘圆角背景。
 * data-tauri-drag-region 让整个可见区域可拖动。
 */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { onMounted, onUnmounted, ref, computed, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

import type { TimerSnapshot, TimerStatus } from "@/types";

const snapshot = ref<TimerSnapshot | null>(null);
const expanded = ref(false);
let unlisteners: UnlistenFn[] = [];

const isIdle = computed(() => !snapshot.value || snapshot.value.status === "idle");

const timeText = computed(() => {
  const s = snapshot.value;
  if (!s || s.status === "idle") return "Ready";
  const isFree = s.mode === "free";
  if (isFree) {
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

const statusLabel = computed((): string => {
  const st = snapshot.value?.status as TimerStatus | undefined;
  switch (st) {
    case "running": return snapshot.value?.mode === "free" ? "🌀" : "🍅";
    case "paused": return "⏸";
    case "break": return "☕";
    case "break_ended": return "☕✓";
    default: return "☕";
  }
});

// 自动调整窗口大小
const appWindow = getCurrentWindow();
watch(expanded, async (exp) => {
  if (exp) {
    await appWindow.setSize(new (await import("@tauri-apps/api/dpi")).LogicalSize(220, 160));
  } else {
    await appWindow.setSize(new (await import("@tauri-apps/api/dpi")).LogicalSize(200, 52));
  }
});

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

onMounted(async () => {
  // 初始快照
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
      plannedSeconds: row.is_break ? (row.break_remaining ?? 0) : (row.planned_seconds ?? 0),
      pomodoroCount: row.pomodoro_count,
      isBreak: row.is_break,
    };
  } catch { /* idle */ }

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
  <div
    class="fl-bubble"
    :class="{ 'is-expanded': expanded, 'is-idle': isIdle }"
    data-tauri-drag-region
    @click="expanded = !expanded"
  >
    <!-- 收缩态 -->
    <div v-if="!expanded" class="fl-bubble-compact" data-tauri-drag-region>
      <span class="fl-bubble-icon">{{ statusLabel }}</span>
      <span class="fl-bubble-time">{{ timeText }}</span>
    </div>

    <!-- 展开态 -->
    <div v-else class="fl-bubble-panel" @click.stop>
      <div class="fl-bp-head" data-tauri-drag-region>
        <span class="fl-bp-icon">{{ statusLabel }}</span>
        <span class="fl-bp-time">{{ timeText }}</span>
        <button class="fl-bp-close" type="button" @click="expanded = false">×</button>
      </div>
      <div class="fl-bp-controls">
        <button
          v-if="snapshot?.status === 'running' || snapshot?.status === 'paused'"
          class="fl-bp-btn"
          type="button"
          @click="togglePause"
        >
          {{ snapshot?.status === 'running' ? '⏸ 暂停' : '▶ 继续' }}
        </button>
        <button
          v-if="snapshot && !isIdle"
          class="fl-bp-btn fl-bp-btn-danger"
          type="button"
          @click="onAbandon"
        >
          ✕ 放弃
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fl-bubble {
  width: 200px;
  height: 48px;
  border-radius: 24px;
  background: rgba(30, 30, 30, 0.82);
  backdrop-filter: blur(16px);
  color: #fff;
  display: flex;
  align-items: center;
  cursor: grab;
  user-select: none;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
  transition: all 0.2s ease;
  overflow: hidden;
}

.fl-bubble.is-expanded {
  width: 216px;
  height: 150px;
  border-radius: 16px;
  flex-direction: column;
  cursor: default;
}

.fl-bubble.is-idle {
  opacity: 0.6;
}

.fl-bubble-compact {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 16px;
  width: 100%;
}

.fl-bubble-icon {
  font-size: 18px;
}

.fl-bubble-time {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 18px;
  font-weight: 600;
  letter-spacing: -0.5px;
  font-variant-numeric: tabular-nums;
}

.fl-bubble-panel {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
}

.fl-bp-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  cursor: grab;
}

.fl-bp-icon {
  font-size: 16px;
}

.fl-bp-time {
  flex: 1;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 20px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.fl-bp-close {
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.5);
  font-size: 18px;
  cursor: pointer;
  padding: 0 4px;
}
.fl-bp-close:hover {
  color: #fff;
}

.fl-bp-controls {
  display: flex;
  gap: 6px;
  padding: 0 14px 12px;
  flex-wrap: wrap;
}

.fl-bp-btn {
  flex: 1;
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s;
}
.fl-bp-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.fl-bp-btn-danger {
  border-color: rgba(239, 68, 68, 0.4);
  color: #fca5a5;
}
.fl-bp-btn-danger:hover {
  background: rgba(239, 68, 68, 0.2);
}
</style>

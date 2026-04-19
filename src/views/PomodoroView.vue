<script setup lang="ts">
/**
 * PomodoroView · 番茄钟沉浸视图 — 对齐 prototype/screens/pomodoro.html。
 * 5 态全屏：专注中 / 最后5分钟 / 休息中 / 完成三选一 / 自由计时。
 * hideLayout: true → 无侧栏，全屏沉浸。
 */

import { Maximize2, Pause, Play, SkipForward, Square } from "lucide-vue-next";
import { computed } from "vue";
import { useRouter } from "vue-router";

import { useTaskStore } from "@/stores/useTaskStore";
import { useTimerStore } from "@/stores/useTimerStore";

const timer = useTimerStore();
const tasks = useTaskStore();
const router = useRouter();

/** 当前任务名 */
const taskName = computed(() => {
  if (!timer.snapshot?.taskId) return "";
  return tasks.tasks.find((t) => t.id === timer.snapshot!.taskId)?.name ?? "任务";
});

/** 判断当前状态 */
const state = computed<"focus" | "sprint" | "break" | "done" | "free" | "idle">(() => {
  if (!timer.snapshot || timer.isIdle) return "idle";
  if (timer.isFreeMode) return "free";
  if (timer.isBreak) return "break";
  if (timer.isBreakEnded) return "done";
  // 最后 5 分钟
  if (timer.isRunning && timer.remainingSeconds <= 300 && timer.remainingSeconds > 0) return "sprint";
  return "focus";
});

/** 时间显示 mm:ss */
const timeDisplay = computed(() => {
  const secs = timer.remainingSeconds;
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
});

const phaseLabel = computed(() => {
  switch (state.value) {
    case "focus": return "专注中";
    case "sprint": return "马上就好";
    case "break": return "放松一下吧";
    case "done": return "休息得怎么样?";
    case "free": return "已专注 · 手动结束";
    default: return "";
  }
});

/** SVG 环参数 */
const R = 135;
const C = 2 * Math.PI * R; // ≈848
const dashOffset = computed(() => {
  if (state.value === "free") return 0;
  return C * (1 - timer.progress);
});

/** 背景样式 */
const bgStyle = computed(() => {
  switch (state.value) {
    case "focus": return { background: "linear-gradient(180deg, #F8FBFF 0%, #E8F1FF 100%)" };
    case "sprint": return { background: "linear-gradient(180deg, #FFF8F4 0%, #FFEDE4 100%)" };
    case "break": return { background: "linear-gradient(180deg, #F5FCEF 0%, #E4F6D4 100%)" };
    case "done": return { background: "linear-gradient(180deg, #FFFBEF 0%, #FFF2C2 100%)" };
    case "free": return { background: "linear-gradient(180deg, #F7F3FF 0%, #EDE5FF 100%)" };
    default: return { background: "var(--color-bg)" };
  }
});

/** 环颜色 */
const ringColor = computed(() => {
  switch (state.value) {
    case "focus": return "var(--color-primary)";
    case "sprint": return "var(--color-q3, #FF8C00)";
    case "break": return "var(--color-success)";
    case "done": return "var(--color-gold, #FAAD14)";
    case "free": return "#7B52D6";
    default: return "var(--color-primary)";
  }
});

function goBack() {
  router.push("/today");
}
</script>

<template>
  <section class="fl-pomo" :style="bgStyle">
    <!-- 顶部栏 -->
    <div class="fl-pomo-top">
      <button class="fl-pomo-back" @click="goBack">
        <Maximize2 :size="14" /> 退出沉浸
      </button>
      <div v-if="state !== 'idle'" class="fl-pomo-tag">
        {{ state === 'free' ? '🌀 自由模式' : '🍅 番茄钟' }}
      </div>
    </div>

    <!-- idle 态 -->
    <div v-if="state === 'idle'" class="fl-pomo-idle">
      <div class="fl-pomo-idle-text">没有进行中的计时</div>
      <button class="fl-pomo-idle-btn" @click="goBack">返回今日页面</button>
    </div>

    <!-- 主体 -->
    <div v-else class="fl-pomo-body">
      <div class="fl-pomo-task">{{ taskName }}</div>

      <!-- 巨型圆环 -->
      <div class="fl-pomo-ring-wrap" :class="[`is-${state}`]">
        <svg class="fl-pomo-ring" viewBox="0 0 280 280">
          <circle
            class="fl-pomo-track"
            cx="140" cy="140" :r="R"
          />
          <circle
            class="fl-pomo-arc"
            cx="140" cy="140" :r="R"
            :style="{
              stroke: ringColor,
              strokeDasharray: C,
              strokeDashoffset: dashOffset,
            }"
          />
        </svg>
        <div class="fl-pomo-center">
          <div class="fl-pomo-time">{{ timeDisplay }}</div>
          <div class="fl-pomo-phase">{{ phaseLabel }}</div>
        </div>
      </div>

      <!-- 控制按钮 -->
      <div v-if="state !== 'done'" class="fl-pomo-controls">
        <button class="fl-pomo-ctrl" @click="timer.abandon()">
          <Square :size="18" />
        </button>
        <button class="fl-pomo-main" @click="timer.isPaused ? timer.resume() : timer.pause()">
          <Play v-if="timer.isPaused" :size="24" />
          <Pause v-else :size="24" />
        </button>
        <button v-if="state === 'break'" class="fl-pomo-ctrl" @click="timer.skipBreak()">
          <SkipForward :size="18" />
        </button>
      </div>

      <!-- 完成三选一 -->
      <div v-else class="fl-pomo-choices">
        <button class="fl-pomo-choice fl-pomo-choice-primary" @click="timer.continueAfterBreak()">
          继续专注
        </button>
        <button class="fl-pomo-choice" @click="timer.extendBreak(300)">
          延长休息 5 分钟
        </button>
        <button class="fl-pomo-choice" @click="timer.abandon()">
          结束任务
        </button>
      </div>

      <!-- 番茄点数 -->
      <div v-if="timer.snapshot && !timer.isFreeMode" class="fl-pomo-dots">
        <span
          v-for="i in 8" :key="i"
          class="fl-pomo-dot"
          :class="{
            'is-done': i <= (timer.snapshot.pomodoroCount - 1),
            'is-current': i === timer.snapshot.pomodoroCount,
          }"
        />
      </div>
    </div>

    <!-- 底部统计 -->
    <div v-if="state !== 'idle'" class="fl-pomo-bottom">
      <span>今日第 {{ timer.snapshot?.pomodoroCount ?? 0 }} 个番茄钟</span>
    </div>
  </section>
</template>

<style scoped>
.fl-pomo {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  transition: background 0.5s ease;
}

.fl-pomo-top {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sp-4) var(--sp-5);
}
.fl-pomo-back {
  display: inline-flex; align-items: center; gap: 6px;
  background: rgba(255,255,255,0.7); backdrop-filter: blur(8px);
  border: 1px solid rgba(0,0,0,0.08); border-radius: var(--r-pill);
  padding: var(--sp-2) var(--sp-3); font-size: var(--fs-12);
  color: var(--color-text-secondary); cursor: pointer;
}
.fl-pomo-back:hover { background: rgba(255,255,255,0.9); }
.fl-pomo-tag {
  font-size: var(--fs-12); color: var(--color-text-secondary);
  padding: var(--sp-1) var(--sp-3); background: rgba(255,255,255,0.6);
  border-radius: var(--r-pill);
}

/* Idle */
.fl-pomo-idle {
  flex: 1; display: flex; flex-direction: column; align-items: center;
  justify-content: center; gap: var(--sp-4);
}
.fl-pomo-idle-text { font-size: var(--fs-16); color: var(--color-text-secondary); }
.fl-pomo-idle-btn {
  padding: var(--sp-3) var(--sp-6); border-radius: var(--r-md);
  background: var(--color-primary); color: #fff; border: none;
  font-size: var(--fs-14); cursor: pointer;
}

/* Body */
.fl-pomo-body {
  flex: 1; display: flex; flex-direction: column; align-items: center;
  justify-content: center; gap: var(--sp-5); padding: var(--sp-4);
}
.fl-pomo-task {
  font-size: var(--fs-20, 20px); font-weight: var(--fw-semibold);
  color: var(--color-text-primary); text-align: center; max-width: 400px;
}

/* Ring */
.fl-pomo-ring-wrap {
  position: relative; width: 280px; height: 280px;
  display: grid; place-items: center;
}
.fl-pomo-ring { position: absolute; inset: 0; width: 280px; height: 280px; }
.fl-pomo-track { fill: none; stroke: rgba(0,0,0,0.06); stroke-width: 10; }
.fl-pomo-arc {
  fill: none; stroke-width: 10; stroke-linecap: round;
  transform: rotate(-90deg); transform-origin: 50% 50%;
  transition: stroke-dashoffset 0.3s ease, stroke 0.5s ease;
}

/* 冲刺脉冲 */
.fl-pomo-ring-wrap.is-sprint .fl-pomo-arc {
  animation: ringPulse 2s ease-in-out infinite;
}
@keyframes ringPulse {
  0%, 100% { filter: drop-shadow(0 0 8px rgba(255,140,0,0.3)); }
  50% { filter: drop-shadow(0 0 20px rgba(255,140,0,0.6)); }
}

/* 自由旋转 */
.fl-pomo-ring-wrap.is-free .fl-pomo-track { stroke-dasharray: 4 8; stroke: rgba(123,82,214,0.18); }
.fl-pomo-ring-wrap.is-free .fl-pomo-arc { animation: freeSweep 12s linear infinite; }
@keyframes freeSweep { 0% { stroke-dashoffset: 848; } 100% { stroke-dashoffset: 0; } }

.fl-pomo-center { position: relative; text-align: center; }
.fl-pomo-time {
  font-family: var(--font-mono); font-size: 56px; font-weight: var(--fw-semibold);
  color: var(--color-text-primary); letter-spacing: -1.5px; line-height: 1;
}
.fl-pomo-phase {
  font-size: 12px; color: var(--color-text-secondary); margin-top: 6px;
  text-transform: uppercase; letter-spacing: 0.5px;
}

/* Controls */
.fl-pomo-controls { display: flex; gap: var(--sp-3); align-items: center; }
.fl-pomo-ctrl {
  width: 44px; height: 44px; border-radius: 50%;
  background: rgba(255,255,255,0.85); backdrop-filter: blur(8px);
  border: 1px solid rgba(0,0,0,0.08); color: var(--color-text-secondary);
  cursor: pointer; display: grid; place-items: center;
  transition: all var(--dur-base);
}
.fl-pomo-ctrl:hover { color: var(--color-primary); transform: translateY(-1px); }
.fl-pomo-main {
  width: 56px; height: 56px; border-radius: 50%;
  background: var(--color-primary); color: #fff; border: none;
  cursor: pointer; display: grid; place-items: center;
  box-shadow: 0 8px 20px color-mix(in srgb, var(--color-primary) 32%, transparent);
  transition: all var(--dur-base);
}
.fl-pomo-main:hover { background: var(--color-primary-dark); transform: translateY(-1px); }

/* Choices (done state) */
.fl-pomo-choices { display: flex; flex-direction: column; gap: var(--sp-2); width: 280px; }
.fl-pomo-choice {
  padding: var(--sp-3); border-radius: var(--r-md);
  border: 1px solid rgba(0,0,0,0.08); background: rgba(255,255,255,0.7);
  backdrop-filter: blur(8px); font-size: var(--fs-14); cursor: pointer;
  transition: all var(--dur-fast);
}
.fl-pomo-choice:hover { background: rgba(255,255,255,0.95); transform: translateY(-1px); }
.fl-pomo-choice-primary {
  background: var(--color-primary); color: #fff; border-color: transparent;
  font-weight: var(--fw-medium);
}
.fl-pomo-choice-primary:hover { background: var(--color-primary-dark); }

/* Dots */
.fl-pomo-dots { display: flex; gap: 8px; }
.fl-pomo-dot {
  width: 12px; height: 12px; border-radius: 50%;
  background: rgba(255,255,255,0.6); border: 1.5px solid rgba(0,0,0,0.1);
}
.fl-pomo-dot.is-done { background: #FF7A5C; border-color: #FF7A5C; }
.fl-pomo-dot.is-current { background: var(--color-primary); border-color: var(--color-primary); animation: pulse-dot 2s ease-in-out infinite; }
@keyframes pulse-dot { 0%,100% { box-shadow: 0 0 0 0 rgba(79,140,255,0.4); } 50% { box-shadow: 0 0 0 4px transparent; } }

/* Bottom */
.fl-pomo-bottom {
  width: 100%; padding: var(--sp-3) var(--sp-5);
  text-align: center; font-size: var(--fs-12); color: var(--color-text-muted);
  border-top: 1px solid rgba(0,0,0,0.06);
}
</style>

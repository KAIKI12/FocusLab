<script setup lang="ts">
/**
 * PomodoroView · 番茄钟沉浸视图 — 对齐 prototype/screens/pomodoro.html。
 * 5 态全屏：专注中 / 最后5分钟 / 休息中 / 完成三选一 / 自由计时。
 * hideLayout: true → 无侧栏，全屏沉浸。
 */

import { Maximize2, Pause, Play, SkipForward, Square } from "lucide-vue-next";
import { computed, ref, watch } from "vue";
import { useRouter } from "vue-router";

import MicroReview from "@/components/task/MicroReview.vue";
import { invokeCmd } from "@/composables/useTauriInvoke";
import { useGoalStore } from "@/stores/useGoalStore";
import { useTaskStore } from "@/stores/useTaskStore";
import { useTimerStore } from "@/stores/useTimerStore";

const timer = useTimerStore();
const tasks = useTaskStore();
const goals = useGoalStore();
const router = useRouter();

const showReview = ref(false);
const reviewDismissed = ref(false);
const reviewScenario = ref<"deviation" | "q1" | "milestone">("deviation");
const reviewMilestoneName = ref<string | null>(null);

/** 当前任务对象 */
const currentTask = computed(() => {
  if (!timer.snapshot?.taskId) return null;
  return tasks.tasks.find((t) => t.id === timer.snapshot!.taskId) ?? null;
});

/** 当前任务名 */
const taskName = computed(() => currentTask.value?.name ?? "任务");

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

// 进入 done 态时按规则决定是否弹 MicroReview,以及用什么场景
// 规则(对齐 prototype/screens/micro-review.html:520-569):
//   1. 关联里程碑 → milestone 场景(必弹)
//   2. 紧急重要 Q1 → q1 场景(必弹)
//   3. 时间偏差 > 30% → deviation 场景(必弹)
//   4. 否则静默(Q3/Q4 事务性 / 偏差 <30%)
//   另: 同一天已弹 ≥3 次 → 静默
function resolveReviewScenario(): "deviation" | "q1" | "milestone" | null {
  const task = currentTask.value;
  if (!task) return null;

  const today = new Date().toISOString().slice(0, 10);
  const countKey = `fl-micro-review-count-${today}`;
  const count = Number(localStorage.getItem(countKey) ?? "0");
  if (count >= 3) return null;

  if (task.milestone_id) return "milestone";
  if (task.quadrant === "important_urgent") return "q1";

  const est = task.estimated_minutes;
  if (est && est > 0) {
    const actualMin = Math.floor((timer.snapshot?.elapsedSeconds ?? 0) / 60);
    const dev = Math.abs((actualMin - est) / est);
    if (dev > 0.3) return "deviation";
  }
  return null;
}

watch(state, (s, old) => {
  if (s === "done" && old !== "done") {
    const scenario = resolveReviewScenario();
    if (!scenario) {
      reviewDismissed.value = true;
      return;
    }
    reviewScenario.value = scenario;
    reviewMilestoneName.value = scenario === "milestone"
      ? (goals.milestones.find((m) => m.id === currentTask.value?.milestone_id)?.name ?? null)
      : null;
    reviewDismissed.value = false;
    showReview.value = true;

    // 计入当天弹出次数
    const today = new Date().toISOString().slice(0, 10);
    const countKey = `fl-micro-review-count-${today}`;
    const count = Number(localStorage.getItem(countKey) ?? "0");
    localStorage.setItem(countKey, String(count + 1));
  }
});

const currentTaskEstimate = computed(() => {
  if (!timer.snapshot?.taskId) return null;
  const t = tasks.tasks.find((x) => x.id === timer.snapshot!.taskId);
  return t?.estimated_minutes ?? null;
});

const actualMinutes = computed(() =>
  timer.snapshot ? Math.floor(timer.snapshot.elapsedSeconds / 60) : 0,
);

async function onReviewSubmit(data: { reason: string; note: string }) {
  showReview.value = false;
  reviewDismissed.value = true;
  if (!timer.snapshot?.taskId) return;
  try {
    await invokeCmd("create_task_reflection", {
      taskId: timer.snapshot.taskId,
      plannedMinutes: currentTaskEstimate.value,
      actualMinutes: actualMinutes.value,
      overtimeReason: data.reason || null,
      note: data.note || null,
    });
  } catch (e) { console.error("[review] save failed", e); }
}

function onReviewSkip() {
  showReview.value = false;
  reviewDismissed.value = true;
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

      <!-- 🌀 自由模式 vs 🍅 番茄模式 对比(对齐 prototype/pomodoro.html:508) -->
      <div class="fl-pomo-compare">
        <h3>🌀 自由模式 vs 🍅 番茄模式</h3>
        <div class="fl-pc-grid">
          <div class="fl-pc-col fl-pc-free">
            <strong>🌀 自由模式</strong>
            <ul>
              <li>时长不固定 · 递增显示</li>
              <li>手动结束 · 不自动进入休息</li>
              <li>不计入"番茄钟数" · 只计入专注时长</li>
              <li>中断仍然记录(暂停/切换任务会打断)</li>
              <li>适合:写作、读论文、深度思考</li>
            </ul>
          </div>
          <div class="fl-pc-col fl-pc-pomo">
            <strong>🍅 番茄模式</strong>
            <ul>
              <li>固定时长(25 / 45 / 90 可选) · 递减显示</li>
              <li>自动结束 · 进入休息</li>
              <li>计入"番茄钟数"</li>
              <li>休息结束三选一(继续 / 延长 / 结束)</li>
              <li>适合:coding、做题、事务性任务</li>
            </ul>
          </div>
        </div>
        <div class="fl-pc-hint">
          💡 切换方式:在今日页面的预设切换器里选 · 计时中不可切换(先结束当前会话)
        </div>
      </div>
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
        <div class="fl-pomo-ai-hint">
          ✨ 休息后继续同一任务效率最高，建议趁热打铁
        </div>
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
      <span v-if="state === 'free'">🌀 不计入番茄钟数，计入专注时长 · 手动结束</span>
      <span v-else>今日第 {{ timer.snapshot?.pomodoroCount ?? 0 }} 个番茄钟</span>
    </div>

    <!-- 微复盘弹窗 -->
    <MicroReview
      :visible="showReview"
      :task-name="taskName"
      :estimated-minutes="currentTaskEstimate"
      :actual-minutes="actualMinutes"
      :scenario="reviewScenario"
      :milestone-name="reviewMilestoneName"
      @submit="onReviewSubmit"
      @close="onReviewSkip"
    />
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

/* 模式对比面板(idle 态下方) */
.fl-pomo-compare {
  width: min(560px, 90%);
  margin-top: var(--sp-6);
  padding: var(--sp-4);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  text-align: left;
}
.fl-pomo-compare h3 {
  font-size: var(--fs-14);
  font-weight: var(--fw-semibold);
  margin: 0 0 var(--sp-3);
  color: var(--color-text-secondary);
}
.fl-pc-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--sp-4);
  font-size: var(--fs-13, 13px);
  line-height: 1.6;
}
.fl-pc-col strong { font-size: var(--fs-13, 13px); font-weight: var(--fw-semibold); }
.fl-pc-col.fl-pc-free strong { color: #7B52D6; }
.fl-pc-col.fl-pc-pomo strong { color: var(--color-primary); }
.fl-pc-col ul {
  margin: 6px 0 0;
  padding-left: 18px;
  color: var(--color-text-secondary);
}
.fl-pc-col li { margin-bottom: 2px; }
.fl-pc-hint {
  margin-top: var(--sp-3);
  padding-top: var(--sp-3);
  border-top: 1px dashed color-mix(in srgb, var(--color-border) 70%, transparent);
  font-size: 11px;
  color: var(--color-text-muted);
  line-height: 1.6;
}
@media (max-width: 560px) {
  .fl-pc-grid { grid-template-columns: 1fr; }
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

.fl-pomo-ai-hint {
  font-size: 12px; color: var(--color-text-muted);
  text-align: center; margin-top: var(--sp-2);
  padding: var(--sp-2) var(--sp-3);
  background: rgba(255,255,255,0.5); border-radius: var(--r-sm);
}

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

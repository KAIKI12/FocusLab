<script setup lang="ts">
/**
 * MinimalView · 极简模式 — 对齐 prototype/minimal/minimal.html。
 * 420×640 精简界面：仅任务列表 + 番茄环 + 基础统计。
 */

import { Check, Moon, Pause, Play, Plus, Square } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";

import { useAssignmentStore } from "@/stores/useAssignmentStore";
import { useSettlementStore } from "@/stores/useSettlementStore";
import { useTaskStore } from "@/stores/useTaskStore";
import { useTimerStore } from "@/stores/useTimerStore";

const tasks = useTaskStore();
const timer = useTimerStore();
const assignments = useAssignmentStore();
const settlement = useSettlementStore();

const name = ref("");

onMounted(() => {
  tasks.load();
  assignments.load();
});

const timerDisplay = computed(() => {
  const secs = timer.remainingSeconds;
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
});

const completedCount = computed(() =>
  assignments.assignments.filter((a) => a.dayStatus === "completed").length,
);
const totalCount = computed(() => assignments.assignments.length);

const progress = computed(() => timer.progress);

// SVG 环参数
const RADIUS = 42;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;
const dashOffset = computed(() => CIRCUMFERENCE * (1 - progress.value));

async function onAdd() {
  const trimmed = name.value.trim();
  if (!trimmed) return;
  await tasks.create({ name: trimmed });
  name.value = "";
}

async function startTask(taskId: string) {
  if (!timer.isIdle) return;
  if (timer.selectedPreset === "free") await timer.startFree(taskId);
  else await timer.startPomodoro(taskId, timer.selectedPreset);
}
</script>

<template>
  <section class="fl-minimal">
    <!-- 头部 -->
    <header class="fl-min-head">
      <div class="fl-min-date">
        {{ new Date().toLocaleDateString("zh-CN", { month: "long", day: "numeric" }) }}
      </div>
      <div class="fl-min-time">
        {{ new Date().toLocaleDateString("zh-CN", { weekday: "short" }) }}
      </div>
    </header>

    <!-- 任务列表 -->
    <div class="fl-min-section">
      <div class="fl-min-section-title">今日任务</div>
      <ul class="fl-min-tasks">
        <li
          v-for="t in tasks.tasks.slice(0, 6)"
          :key="t.id"
          class="fl-min-task"
          :class="{ 'is-done': t.status === 'completed' }"
        >
          <button class="fl-min-check" @click="tasks.complete(t.id)">
            <Check :size="10" />
          </button>
          <span class="fl-min-name">{{ t.name }}</span>
          <button
            v-if="timer.isIdle && !t.is_background"
            class="fl-min-play"
            @click="startTask(t.id)"
          >
            <Play :size="10" />
          </button>
        </li>
      </ul>
      <form class="fl-min-add" @submit.prevent="onAdd">
        <Plus :size="12" />
        <input v-model="name" placeholder="添加任务…" maxlength="60" />
      </form>
    </div>

    <!-- 番茄环 -->
    <div v-if="!timer.isIdle" class="fl-min-timer">
      <svg class="fl-min-ring" viewBox="0 0 100 100">
        <circle class="fl-min-track" cx="50" cy="50" :r="RADIUS" />
        <circle
          class="fl-min-progress"
          cx="50" cy="50" :r="RADIUS"
          :style="{
            strokeDasharray: CIRCUMFERENCE,
            strokeDashoffset: timer.isFreeMode ? 0 : dashOffset,
          }"
        />
      </svg>
      <div class="fl-min-timer-text">
        <div class="fl-min-timer-time">{{ timerDisplay }}</div>
        <div class="fl-min-timer-phase">
          {{ timer.isBreak ? '休息' : timer.isPaused ? '暂停' : '专注中' }}
        </div>
      </div>
      <div class="fl-min-controls">
        <button class="fl-min-ctrl" @click="timer.abandon()"><Square :size="14" /></button>
        <button class="fl-min-ctrl-main" @click="timer.isPaused ? timer.resume() : timer.pause()">
          <Play v-if="timer.isPaused" :size="18" />
          <Pause v-else :size="18" />
        </button>
      </div>
    </div>

    <!-- 统计 -->
    <div class="fl-min-stats">
      <div class="fl-min-stat">
        <span class="fl-min-stat-val">{{ completedCount }}/{{ totalCount }}</span>
        <span class="fl-min-stat-label">完成</span>
      </div>
      <div class="fl-min-stat">
        <span class="fl-min-stat-val">{{ timer.snapshot?.pomodoroCount ?? 0 }}</span>
        <span class="fl-min-stat-label">番茄</span>
      </div>
    </div>

    <!-- 结束今天 -->
    <button class="fl-min-settle" :disabled="settlement.settling" @click="settlement.settle()">
      <Moon :size="14" /> 结束今天
    </button>
  </section>
</template>

<style scoped>
.fl-minimal {
  max-width: 420px;
  margin: 0 auto;
  padding: var(--sp-5);
  display: flex;
  flex-direction: column;
  gap: var(--sp-4);
  min-height: 100vh;
}

.fl-min-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
}
.fl-min-date { font-size: var(--fs-20, 20px); font-weight: var(--fw-semibold); }
.fl-min-time { font-size: var(--fs-12); color: var(--color-text-muted); }

.fl-min-section-title {
  font-size: var(--fs-12); color: var(--color-text-muted);
  font-weight: var(--fw-medium); text-transform: uppercase;
  letter-spacing: 0.5px; margin-bottom: var(--sp-2);
}

.fl-min-tasks { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 4px; }
.fl-min-task {
  display: flex; align-items: center; gap: var(--sp-2);
  padding: var(--sp-2) var(--sp-3); border-radius: var(--r-sm);
  transition: background var(--dur-fast);
}
.fl-min-task:hover { background: var(--color-bg-hover); }
.fl-min-task.is-done .fl-min-name { text-decoration: line-through; color: var(--color-text-muted); }

.fl-min-check {
  width: 18px; height: 18px; border-radius: 50%;
  border: 1.5px solid var(--color-border-strong); background: transparent;
  color: transparent; cursor: pointer; display: grid; place-items: center;
  transition: all var(--dur-fast);
}
.fl-min-check:hover { border-color: var(--color-success); background: var(--color-success); color: #fff; }

.fl-min-name { flex: 1; font-size: var(--fs-14); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.fl-min-play {
  opacity: 0; width: 22px; height: 22px; border-radius: 50%;
  border: 1px solid var(--color-border); background: transparent;
  color: var(--color-success); cursor: pointer; display: grid; place-items: center;
  transition: all var(--dur-fast);
}
.fl-min-task:hover .fl-min-play { opacity: 1; }
.fl-min-play:hover { background: var(--color-success); color: #fff; border-color: var(--color-success); }

.fl-min-add {
  display: flex; align-items: center; gap: var(--sp-2);
  padding: var(--sp-2) var(--sp-3); margin-top: var(--sp-2);
  border: 1px dashed var(--color-border); border-radius: var(--r-sm);
  color: var(--color-text-muted); font-size: var(--fs-12);
}
.fl-min-add input {
  flex: 1; border: none; background: none; outline: none;
  color: var(--color-text-primary); font-size: var(--fs-13, 13px);
}

/* Timer */
.fl-min-timer {
  display: flex; flex-direction: column; align-items: center; gap: var(--sp-3);
  padding: var(--sp-4); background: var(--color-bg-elevated);
  border: 1px solid var(--color-border); border-radius: var(--r-lg);
}
.fl-min-ring { width: 100px; height: 100px; }
.fl-min-track { fill: none; stroke: var(--color-bg-hover); stroke-width: 6; }
.fl-min-progress {
  fill: none; stroke: var(--color-primary); stroke-width: 6;
  stroke-linecap: round; transform: rotate(-90deg); transform-origin: 50% 50%;
  transition: stroke-dashoffset 0.3s ease;
}
.fl-min-timer-text { text-align: center; }
.fl-min-timer-time { font-family: var(--font-mono); font-size: 28px; font-weight: var(--fw-semibold); letter-spacing: -1px; }
.fl-min-timer-phase { font-size: 11px; color: var(--color-text-muted); text-transform: uppercase; }

.fl-min-controls { display: flex; gap: var(--sp-2); }
.fl-min-ctrl {
  width: 32px; height: 32px; border-radius: 50%;
  border: 1px solid var(--color-border); background: transparent;
  color: var(--color-text-secondary); cursor: pointer; display: grid; place-items: center;
}
.fl-min-ctrl:hover { border-color: var(--color-primary); color: var(--color-primary); }
.fl-min-ctrl-main {
  width: 40px; height: 40px; border-radius: 50%;
  background: var(--color-primary); color: #fff; border: none;
  cursor: pointer; display: grid; place-items: center;
}

/* Stats */
.fl-min-stats { display: flex; gap: var(--sp-4); justify-content: center; }
.fl-min-stat { display: flex; flex-direction: column; align-items: center; gap: 2px; }
.fl-min-stat-val { font-family: var(--font-mono); font-size: var(--fs-16); font-weight: var(--fw-semibold); }
.fl-min-stat-label { font-size: 11px; color: var(--color-text-muted); }

.fl-min-settle {
  width: 100%; padding: var(--sp-3); border: none;
  border-radius: var(--r-md); background: var(--color-bg-elevated);
  border: 1px solid var(--color-border); color: var(--color-text-secondary);
  font-size: var(--fs-14); cursor: pointer; display: flex; align-items: center;
  justify-content: center; gap: var(--sp-2); margin-top: auto;
}
.fl-min-settle:hover { border-color: var(--color-primary); color: var(--color-primary); }
.fl-min-settle:disabled { opacity: 0.4; cursor: not-allowed; }
</style>

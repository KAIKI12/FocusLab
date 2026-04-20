<script setup lang="ts">
/**
 * TodayView · 对齐 prototype/screens/main-today.html 双栏布局。
 *
 * 左栏 — 焦点卡(内嵌计时环) + 任务面板(象限分组)
 * 右栏 — AI 建议卡 + 目标卡 + 今日进度卡
 */

import { Calendar, Check, ChevronRight, Clock, Grid2X2, List, Maximize2, Minimize2, Moon, Pause, Pencil, Play, Plus, SkipForward, Square, Trash2, X } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";

import ManualSessionModal from "@/components/timer/ManualSessionModal.vue";
import MorningGuide from "@/components/common/MorningGuide.vue";
import { invokeCmd } from "@/composables/useTauriInvoke";
import PresetSwitcher from "@/components/timer/PresetSwitcher.vue";
import QuadrantGrid from "@/components/task/QuadrantGrid.vue";
import QuickAddModal from "@/components/task/QuickAddModal.vue";
import TaskEditModal from "@/components/task/TaskEditModal.vue";
import { useBubble } from "@/composables/useBubble";
import { useAssignmentStore } from "@/stores/useAssignmentStore";
import { useGoalStore } from "@/stores/useGoalStore";
import { useSettlementStore } from "@/stores/useSettlementStore";
import { useTaskStore } from "@/stores/useTaskStore";
import { useTimerStore } from "@/stores/useTimerStore";
import { useUIStore } from "@/stores/useUIStore";
import { useAIStore } from "@/stores/useAIStore";
import type { AssignmentWithTask, Task } from "@/types";

const tasks = useTaskStore();
const assignments = useAssignmentStore();
const timer = useTimerStore();
const settlement = useSettlementStore();
const goals = useGoalStore();
const ui = useUIStore();
const ai = useAIStore();
const { open: openBubble } = useBubble();
const router = useRouter();

const name = ref("");
const isBackground = ref(false);
const viewMode = ref<"list" | "quadrant">("list");
const editingTask = ref<Task | null>(null);
const showManualSession = ref(false);
const showMorningGuide = ref(false);
const goalLabels = ref<Record<string, string>>({});
const aiSuggestion = ref("优先处理紧急重要象限的任务，上午注意力最佳时段可一鼓作气拿下核心任务。");
const aiLoading = ref(false);

onMounted(async () => {
  // 到期任务自动置顶 + 重复任务生成
  await Promise.all([
    invokeCmd("pin_due_tasks").catch(() => {}),
    invokeCmd("generate_recurring_tasks").catch(() => {}),
  ]);
  await Promise.all([tasks.load(), assignments.load(), goals.loadGoals()]);
  // 构建 milestone_id → 目标名 映射
  for (const g of goals.goals) {
    try {
      const ms = await invokeCmd<Array<{ id: string }>>( "list_milestones", { goalId: g.id });
      for (const m of ms) { goalLabels.value[m.id] = `🎯 ${g.name}`; }
    } catch { /* */ }
  }
  // 每日自动弹出晨起引导(当日未完成过)
  const today = new Date().toISOString().slice(0, 10);
  if (!localStorage.getItem(`fl-morning-${today}`)) {
    showMorningGuide.value = true;
  }
  // 获取 AI 建议(异步，不阻塞)
  loadAISuggestion();
});

async function loadAISuggestion() {
  aiLoading.value = true;
  try {
    const text = await ai.dailySuggestions();
    if (text) aiSuggestion.value = text;
  } catch { /* fallback 保持默认文案 */ }
  finally { aiLoading.value = false; }
}

const assignedTaskIds = computed(
  () => new Set(assignments.assignments.map((a) => a.taskId)),
);

/** 后台任务排在主任务之后 */
const sortedTasks = computed(() =>
  [...tasks.tasks].sort((a, b) => Number(a.is_background) - Number(b.is_background)),
);

/** 按象限分组(列表视图用) */
const tasksByQuadrantList = computed(() => {
  const groups: Record<string, Task[]> = {
    important_urgent: [],
    important_not_urgent: [],
    not_important_urgent: [],
    not_important_not_urgent: [],
  };
  for (const t of sortedTasks.value) {
    const q = t.quadrant in groups ? t.quadrant : "important_not_urgent";
    groups[q].push(t);
  }
  return groups;
});

const QUADRANT_META: Record<string, { label: string; cls: string }> = {
  important_urgent: { label: "紧急重要", cls: "q1" },
  important_not_urgent: { label: "计划执行", cls: "q2" },
  not_important_urgent: { label: "快速处理", cls: "q3" },
  not_important_not_urgent: { label: "减少搁置", cls: "q4" },
};

/** 计时显示 mm:ss */
const timerDisplay = computed(() => {
  const secs = timer.remainingSeconds;
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
});

/** 今日完成统计 */
const completedCount = computed(() =>
  assignments.assignments.filter((a) => a.dayStatus === "completed").length,
);
const totalCount = computed(() => assignments.assignments.length);
const completionPct = computed(() =>
  totalCount.value > 0 ? Math.round((completedCount.value / totalCount.value) * 100) : 0,
);

/** 今日到期(due_date == today) */
const dueTodayCount = computed(() => {
  const today = new Date().toISOString().slice(0, 10);
  return tasks.tasks.filter(
    (t) => t.due_date === today && t.status !== "completed",
  ).length;
});

/** 逾期未完成(due_date < today) — 后端 pin_due_tasks 已自动置顶 */
const overdueCount = computed(() => {
  const today = new Date().toISOString().slice(0, 10);
  return tasks.tasks.filter(
    (t) => !!t.due_date && t.due_date < today && t.status !== "completed",
  ).length;
});

/** 合并后的 DDL 条是否展示 */
const showDdlBar = computed(() => dueTodayCount.value + overdueCount.value > 0);

/** 第一个目标(右侧栏) */
const primaryGoal = computed(() => goals.goals[0] ?? null);

async function onAdd() {
  const trimmed = name.value.trim();
  if (!trimmed) return;
  const created = await tasks.create({ name: trimmed });
  if (isBackground.value) {
    await tasks.update({ id: created.id, isBackground: true });
  }
  name.value = "";
  isBackground.value = false;
}

async function addToPlan(taskId: string) {
  try { await assignments.create({ taskId }); } catch (e) { console.error(e); }
}

async function onStartPomodoro(taskId: string) {
  if (!timer.isIdle) return;
  try {
    if (timer.selectedPreset === "free") await timer.startFree(taskId);
    else await timer.startPomodoro(taskId, timer.selectedPreset);
  } catch (e) { console.error("[timer] start failed", e); }
}

async function onChangeQuadrant(taskId: string, quadrant: string) {
  try { await tasks.update({ id: taskId, quadrant }); } catch (e) { console.error(e); }
}

/** 三态轮转的提示文字 */
const CYCLE_TITLE: Record<string, string> = {
  pending: "点击进入进行中",
  in_progress: "点击标记完成",
  completed: "点击恢复待办",
};
function cycleTitle(status: string): string {
  return CYCLE_TITLE[status] ?? "切换状态";
}

async function onCycleTask(t: Task) {
  try {
    const next = await tasks.cycleStatus(t.id, t.status);
    // 列表 A/B 存在任务实体共享的情况:同步今日计划里的 taskStatus 快照
    const a = assignments.assignments.find((x: AssignmentWithTask) => x.taskId === t.id);
    if (a) await syncAssignmentAfterCycle(a, next);
  } catch (e) { console.error(e); }
}

async function onCycleAssignment(a: AssignmentWithTask) {
  try {
    const next = await tasks.cycleStatus(a.taskId, a.taskStatus);
    await syncAssignmentAfterCycle(a, next);
  } catch (e) { console.error(e); }
}

/** 双写:taskStatus → completed 时连带 dayStatus=completed;回退则 dayStatus=pending */
async function syncAssignmentAfterCycle(a: AssignmentWithTask, next: "pending" | "in_progress" | "completed") {
  a.taskStatus = next;
  if (next === "completed") {
    if (a.dayStatus !== "completed") await assignments.setStatus(a.id, "completed");
  } else if (a.dayStatus === "completed") {
    await assignments.setStatus(a.id, "pending");
  }
}

function fmtMin(m: number): string {
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  const r = m % 60;
  return r > 0 ? `${h}h ${r}m` : `${h}h`;
}
</script>

<template>
  <section class="fl-today">
    <!-- Page head -->
    <header class="fl-page-head">
      <div class="fl-date-line">
        <h1 class="fl-date-main">今日</h1>
        <span class="fl-date-sub">任务 + 专注 + 计划</span>
      </div>
      <div class="fl-page-actions">
        <button class="fl-action-btn" type="button" title="补录记录" @click="showManualSession = true">
          <Clock :size="14" /> 补录
        </button>
        <button class="fl-action-btn" type="button" title="悬浮球" @click="openBubble">
          <Minimize2 :size="14" /> 悬浮球
        </button>
      </div>
    </header>

    <!-- 双栏主体 -->
    <div class="fl-grid">
      <!-- 左栏 -->
      <div class="fl-left">
        <!-- 焦点卡(计时中) -->
        <div v-if="!timer.isIdle" class="fl-focus-card">
          <div class="fl-focus-head">
            <span class="fl-focus-label">
              {{ timer.isFreeMode ? '🌀 自由计时' : '🍅 当前专注' }}
            </span>
            <button class="fl-focus-immerse" @click="router.push('/pomodoro')">
              <Maximize2 :size="12" /> 沉浸
            </button>
          </div>

          <div class="fl-ring-wrap">
            <div class="fl-ring-holder" :class="{ 'is-free': timer.isFreeMode, 'is-break': timer.isBreak }">
              <svg class="fl-ring" viewBox="0 0 240 240">
                <circle class="fl-ring-track" cx="120" cy="120" r="105" />
                <circle
                  class="fl-ring-progress"
                  cx="120" cy="120" r="105"
                  :style="{
                    strokeDasharray: 660,
                    strokeDashoffset: timer.isFreeMode ? 0 : 660 * (1 - timer.progress),
                  }"
                />
              </svg>
              <div class="fl-ring-center">
                <div class="fl-ring-time">{{ timerDisplay }}</div>
                <div class="fl-ring-phase">
                  {{ timer.isBreak ? '休息中' : timer.isPaused ? '已暂停' : timer.isFreeMode ? '自由专注' : '专注中' }}
                </div>
              </div>
            </div>

            <div class="fl-focus-controls">
              <button class="fl-ctrl-circle" type="button" title="结束" @click="timer.abandon()">
                <Square :size="16" />
              </button>
              <button
                class="fl-ctrl-main"
                type="button"
                :title="timer.isPaused ? '继续' : '暂停'"
                @click="timer.isPaused ? timer.resume() : timer.pause()"
              >
                <Play v-if="timer.isPaused" :size="22" />
                <Pause v-else :size="22" />
              </button>
              <button class="fl-ctrl-circle" type="button" title="跳过" @click="timer.skipBreak()" v-if="timer.isBreak">
                <SkipForward :size="16" />
              </button>
            </div>

            <!-- 番茄计数 -->
            <div v-if="!timer.isFreeMode && timer.snapshot" class="fl-pomo-dots">
              <span
                v-for="i in 8"
                :key="i"
                class="fl-pomo-dot"
                :class="{
                  'is-done': i <= (timer.snapshot.pomodoroCount - 1),
                  'is-current': i === timer.snapshot.pomodoroCount,
                }"
              />
            </div>
          </div>
        </div>

        <!-- 预设选择(仅 idle) -->
        <PresetSwitcher />

        <!-- DDL 到期总览条(v1.2.2) -->
        <div v-if="showDdlBar" class="fl-ddl-bar">
          <span style="font-size:16px;line-height:1">⏰</span>
          <div class="fl-ddl-text">
            <strong v-if="dueTodayCount">{{ dueTodayCount }} 个任务今日到期</strong>
            <span v-if="dueTodayCount && overdueCount" class="fl-ddl-sep"> · </span>
            <span v-if="overdueCount" class="fl-ddl-overdue">
              {{ overdueCount }} 个逾期未完成已自动置顶
            </span>
          </div>
          <button class="fl-ddl-link" @click="router.push('/calendar')">
            日历视图 <ChevronRight :size="12" />
          </button>
        </div>

        <!-- 任务面板 -->
        <div class="fl-task-panel">
          <div class="fl-panel-head">
            <h2>今日任务</h2>
            <div class="fl-panel-actions">
              <div class="fl-view-tabs">
                <button :class="{ 'is-active': viewMode === 'list' }" @click="viewMode = 'list'">
                  <List :size="14" /> 列表
                </button>
                <button :class="{ 'is-active': viewMode === 'quadrant' }" @click="viewMode = 'quadrant'">
                  <Grid2X2 :size="14" /> 四象限
                </button>
              </div>
            </div>
          </div>

          <!-- 象限视图 -->
          <QuadrantGrid
            v-if="viewMode === 'quadrant' && tasks.tasks.length"
            :tasks-by-quadrant="tasks.tasksByQuadrant"
            :timer-idle="timer.isIdle"
            :goal-labels="goalLabels"
            @edit="editingTask = $event"
            @start="onStartPomodoro($event)"
            @change-quadrant="onChangeQuadrant"
          />

          <!-- 列表视图(按象限分组) -->
          <template v-else-if="viewMode === 'list' && tasks.tasks.length">
            <div
              v-for="(qTasks, qKey) in tasksByQuadrantList"
              :key="qKey"
              class="fl-q-group"
            >
              <template v-if="qTasks.length">
                <div class="fl-q-label" :class="QUADRANT_META[qKey]?.cls">
                  <span class="fl-q-dot" />
                  {{ QUADRANT_META[qKey]?.label }}
                  <span class="fl-q-count">· {{ qTasks.length }}</span>
                </div>
                <div
                  v-for="t in qTasks"
                  :key="t.id"
                  class="fl-task-item"
                  :class="{ 'is-done': t.status === 'completed' }"
                >
                  <button
                    class="fl-check"
                    type="button"
                    :class="{
                      'is-checked': t.status === 'completed',
                      'is-progress': t.status === 'in_progress',
                    }"
                    :title="cycleTitle(t.status)"
                    @click="onCycleTask(t)"
                  >
                    <Check v-if="t.status === 'completed'" :size="12" />
                  </button>
                  <div class="fl-t-body">
                    <div class="fl-t-name">{{ t.name }}</div>
                    <div class="fl-t-meta">
                      <span v-if="t.estimated_minutes">{{ t.estimated_minutes }}m</span>
                      <span v-if="t.is_background" class="fl-chip fl-chip-bg">后台</span>
                      <span v-if="t.due_date" class="fl-chip fl-chip-due">📅 {{ t.due_date.slice(5) }}</span>
                      <span v-if="t.milestone_id && goalLabels[t.milestone_id]" class="fl-chip fl-chip-goal">{{ goalLabels[t.milestone_id] }}</span>
                    </div>
                  </div>
                  <div class="fl-t-actions">
                    <button class="fl-t-btn" title="编辑" @click="editingTask = t"><Pencil :size="10" /></button>
                    <button
                      v-if="!t.is_background"
                      class="fl-t-btn fl-t-play"
                      :disabled="!timer.isIdle"
                      title="开始"
                      @click="onStartPomodoro(t.id)"
                    >
                      <Play :size="10" />
                    </button>
                    <button
                      class="fl-t-btn"
                      :disabled="assignedTaskIds.has(t.id)"
                      title="加入今日"
                      @click="addToPlan(t.id)"
                    >
                      <Calendar :size="10" />
                    </button>
                    <button class="fl-t-btn fl-t-danger" title="删除" @click="tasks.remove(t.id)">
                      <Trash2 :size="10" />
                    </button>
                  </div>
                </div>
              </template>
            </div>
          </template>

          <div v-else-if="tasks.loading" class="fl-empty">载入中…</div>
          <div v-else class="fl-empty">还没有任务 · 写下第一件 ↓</div>

          <!-- 添加任务 -->
          <form class="fl-add-bar" @submit.prevent="onAdd">
            <Plus :size="16" />
            <input
              v-model="name"
              type="text"
              placeholder="添加新任务… (⌘N 详细添加)"
              maxlength="80"
            />
            <label class="fl-bg-toggle" :title="isBackground ? '后台任务' : '主动任务'">
              <input v-model="isBackground" type="checkbox" class="fl-sr-only" />
              <span class="fl-bg-chip" :class="{ 'is-on': isBackground }">后台</span>
            </label>
            <button type="submit" :disabled="!name.trim()">添加</button>
            <button type="button" class="fl-quick-add-btn" @click="ui.showQuickAdd = true" title="⌘N 快速添加">+</button>
          </form>
        </div>
      </div>

      <!-- 右栏 -->
      <div class="fl-right-rail">
        <!-- Daily 卡 · AI 建议 + 昨日小结(合并,遵循 ADR-010 v1.2.1) -->
        <div class="fl-ai-card">
          <div class="fl-ai-head">
            <div class="fl-ai-avatar">✨</div>
            <div style="flex:1;min-width:0">
              <div class="fl-ai-title">今日小建议</div>
              <div class="fl-ai-text" :class="{ 'is-loading': aiLoading }">
                {{ aiLoading ? '正在生成建议...' : aiSuggestion }}
              </div>
            </div>
            <button class="fl-ai-refresh" title="刷新建议" :disabled="aiLoading" @click="loadAISuggestion">🔄</button>
          </div>

          <div v-if="settlement.yesterday" class="fl-ai-divider" />

          <div v-if="settlement.yesterday" class="fl-ai-yesterday">
            <span class="fl-ai-y-text">
              🌅 昨日 {{ settlement.yesterday.completedTasks }}/{{ settlement.yesterday.totalTasks }}
              · <strong>{{ settlement.yesterday.grade }} 级</strong>
              · 专注 {{ fmtMin(settlement.yesterday.totalFocusMinutes) }}
            </span>
            <button class="fl-ai-y-link" type="button" @click="settlement.openYesterdayDialog()">
              查看结算 <ChevronRight :size="12" />
            </button>
          </div>
        </div>

        <!-- 目标卡 -->
        <div v-if="primaryGoal" class="fl-stat-card">
          <h3>关联长线目标</h3>
          <div class="fl-goal-top">
            <div class="fl-goal-icon">🎯</div>
            <div>
              <div class="fl-goal-name">{{ primaryGoal.name }}</div>
              <div class="fl-goal-sub">{{ primaryGoal.status }}</div>
            </div>
          </div>
        </div>

        <!-- 今日进度卡 -->
        <div class="fl-stat-card fl-progress-card">
          <h3>今日进度</h3>
          <div class="fl-prog-top">
            <div>
              <span class="fl-prog-value">{{ completedCount }} <em>/ {{ totalCount }}</em></span>
              <span class="fl-prog-pct">{{ completionPct }}%</span>
            </div>
          </div>
          <div class="fl-prog-bar">
            <div class="fl-prog-fill" :style="{ width: `${completionPct}%` }" />
          </div>

          <!-- 今日计划任务列表 -->
          <ul v-if="assignments.assignments.length" class="fl-plan-list">
            <li
              v-for="a in assignments.assignments"
              :key="a.id"
              class="fl-plan-item"
              :class="{ 'is-done': a.taskStatus === 'completed' }"
            >
              <button
                class="fl-check fl-check-sm"
                :class="{
                  'is-checked': a.taskStatus === 'completed',
                  'is-progress': a.taskStatus === 'in_progress',
                }"
                :title="cycleTitle(a.taskStatus)"
                @click="onCycleAssignment(a)"
              >
                <Check v-if="a.taskStatus === 'completed'" :size="10" />
              </button>
              <span class="fl-plan-name">{{ a.taskName }}</span>
              <button class="fl-t-btn" @click="assignments.remove(a.id)"><X :size="10" /></button>
            </li>
          </ul>

          <button
            class="fl-settle-cta"
            type="button"
            :disabled="settlement.settling"
            @click="settlement.settle()"
          >
            <Moon :size="16" /> 结束今天 · 进入日结算
          </button>
        </div>
      </div>
    </div>

    <!-- 弹窗 -->
    <TaskEditModal :task="editingTask" @close="editingTask = null" />
    <QuickAddModal :visible="ui.showQuickAdd" @close="ui.showQuickAdd = false" @created="ui.showQuickAdd = false" />
    <ManualSessionModal :visible="showManualSession" @close="showManualSession = false" />
    <MorningGuide :visible="showMorningGuide" @close="showMorningGuide = false" />
  </section>
</template>

<style scoped>
.fl-today {
  max-width: 1100px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--sp-5);
}

/* ---------- Page Head ---------- */
.fl-page-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.fl-date-line { display: flex; align-items: baseline; gap: var(--sp-3); }
.fl-date-main { font-size: var(--fs-24); font-weight: var(--fw-semibold); margin: 0; }
.fl-date-sub { color: var(--color-text-secondary); font-size: var(--fs-14); }
.fl-page-actions { display: flex; gap: var(--sp-2); }
.fl-action-btn {
  display: inline-flex; align-items: center; gap: 4px;
  padding: var(--sp-2) var(--sp-3); border-radius: var(--r-md);
  border: 1px solid var(--color-border); background: var(--color-bg-elevated);
  color: var(--color-text-secondary); font-size: 11px; cursor: pointer;
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-action-btn:hover { border-color: var(--color-primary); color: var(--color-primary); }

/* ---------- Grid ---------- */
.fl-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 320px;
  gap: var(--sp-6);
}
@media (max-width: 960px) {
  .fl-grid { grid-template-columns: 1fr; }
}

.fl-left { display: flex; flex-direction: column; gap: var(--sp-5); }
.fl-right-rail { display: flex; flex-direction: column; gap: var(--sp-4); }

/* ---------- Focus Card ---------- */
.fl-focus-card {
  background: linear-gradient(180deg, var(--color-bg-elevated) 0%, var(--color-primary-soft) 100%);
  border: 1px solid color-mix(in srgb, var(--color-primary) 25%, var(--color-border));
  border-radius: var(--r-lg); padding: var(--sp-5);
  position: relative; overflow: hidden;
}
.fl-focus-card::before {
  content: ""; position: absolute; top: -80px; right: -80px;
  width: 240px; height: 240px;
  background: radial-gradient(circle, var(--color-primary-light), transparent 60%);
  opacity: 0.3; pointer-events: none;
}
.fl-focus-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--sp-4); position: relative; }
.fl-focus-label {
  display: inline-flex; align-items: center; gap: 6px;
  font-size: var(--fs-12); color: var(--color-primary-dark); font-weight: var(--fw-medium);
  padding: 4px 10px; background: rgba(255,255,255,0.6); border-radius: var(--r-pill);
}
.fl-focus-immerse {
  display: inline-flex; align-items: center; gap: 4px;
  padding: 4px 10px; border-radius: var(--r-pill);
  background: rgba(255,255,255,0.6); border: none;
  font-size: 11px; color: var(--color-text-secondary); cursor: pointer;
  position: relative;
}
.fl-focus-immerse:hover { background: rgba(255,255,255,0.9); color: var(--color-primary); }

.fl-ring-wrap { display: flex; flex-direction: column; align-items: center; gap: var(--sp-4); position: relative; }
.fl-ring-holder { position: relative; width: 180px; height: 180px; display: grid; place-items: center; }
.fl-ring { position: absolute; inset: 0; width: 180px; height: 180px; filter: drop-shadow(0 4px 16px color-mix(in srgb, var(--color-primary) 18%, transparent)); }
.fl-ring-track { fill: none; stroke: rgba(255,255,255,0.8); stroke-width: 10; }
.fl-ring-progress { fill: none; stroke: var(--color-primary); stroke-width: 10; stroke-linecap: round; transform: rotate(-90deg); transform-origin: 50% 50%; transition: stroke-dashoffset 0.3s ease; }
.fl-ring-holder.is-free .fl-ring-track { stroke: rgba(139,92,246,0.18); stroke-dasharray: 4 8; }
.fl-ring-holder.is-free .fl-ring-progress { stroke: #8B5CF6; }
.fl-ring-holder.is-break .fl-ring-progress { stroke: var(--color-success); }
.fl-ring-center { position: relative; text-align: center; line-height: 1; }
.fl-ring-time { font-family: var(--font-mono); font-size: 40px; font-weight: var(--fw-semibold); letter-spacing: -1px; color: var(--color-text-primary); }
.fl-ring-phase { font-size: 11px; color: var(--color-text-secondary); margin-top: 4px; text-transform: uppercase; letter-spacing: 0.5px; }

.fl-focus-controls { display: flex; justify-content: center; gap: var(--sp-3); position: relative; }
.fl-ctrl-circle {
  width: 44px; height: 44px; border-radius: 50%;
  background: var(--color-bg-elevated); border: 1px solid var(--color-border);
  color: var(--color-text-secondary); display: grid; place-items: center; cursor: pointer;
  transition: all var(--dur-base) var(--ease-smooth);
}
.fl-ctrl-circle:hover { color: var(--color-primary); border-color: var(--color-primary); transform: translateY(-1px); }
.fl-ctrl-main {
  width: 56px; height: 56px; border-radius: 50%;
  background: var(--color-primary); color: #fff; border: none;
  display: grid; place-items: center; cursor: pointer;
  box-shadow: 0 8px 20px color-mix(in srgb, var(--color-primary) 32%, transparent);
  transition: all var(--dur-base) var(--ease-smooth);
}
.fl-ctrl-main:hover { background: var(--color-primary-dark); transform: translateY(-1px); }

.fl-pomo-dots { display: flex; gap: 6px; }
.fl-pomo-dot {
  width: 12px; height: 12px; border-radius: 50%;
  background: var(--color-bg-elevated); border: 1.5px solid var(--color-border);
}
.fl-pomo-dot.is-done { background: #FF7A5C; border-color: #FF7A5C; }
.fl-pomo-dot.is-current { background: var(--color-primary); border-color: var(--color-primary); animation: pulse-dot 2s ease-in-out infinite; }
@keyframes pulse-dot { 0%,100% { box-shadow: 0 0 0 0 color-mix(in srgb, var(--color-primary) 40%, transparent); } 50% { box-shadow: 0 0 0 4px transparent; } }

/* ---------- DDL Bar ---------- */
.fl-ddl-bar {
  display: flex; align-items: center; gap: var(--sp-3);
  padding: 8px 12px; border-radius: var(--r-sm);
  background: color-mix(in srgb, var(--color-warning) 8%, transparent);
  border: 1px solid color-mix(in srgb, var(--color-warning) 25%, transparent);
  font-size: var(--fs-13, 13px); line-height: 1.5;
}
.fl-ddl-text { flex: 1; min-width: 0; color: var(--color-text-primary); }
.fl-ddl-text strong { color: var(--color-warning-text); font-weight: var(--fw-semibold); }
.fl-ddl-sep { color: var(--color-text-secondary); }
.fl-ddl-overdue { color: var(--color-text-secondary); }
.fl-ddl-link {
  background: none; border: none; color: var(--color-primary);
  font-size: var(--fs-12); cursor: pointer; white-space: nowrap;
  display: inline-flex; align-items: center; gap: 2px;
}
.fl-ddl-link:hover { text-decoration: underline; }

/* ---------- Task Panel ---------- */
.fl-task-panel {
  background: var(--color-bg-elevated); border: 1px solid var(--color-border);
  border-radius: var(--r-lg); padding: var(--sp-5);
}
.fl-panel-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--sp-4); }
.fl-panel-head h2 { font-size: var(--fs-16); font-weight: var(--fw-semibold); margin: 0; }
.fl-panel-actions { display: flex; gap: var(--sp-2); align-items: center; }
.fl-view-tabs {
  display: flex; gap: 2px; padding: 2px;
  background: var(--color-bg-hover); border-radius: var(--r-sm);
}
.fl-view-tabs button {
  display: inline-flex; align-items: center; gap: 4px;
  padding: 5px 12px; border: none; background: transparent;
  border-radius: var(--r-xs); font-size: var(--fs-12);
  color: var(--color-text-secondary); cursor: pointer;
}
.fl-view-tabs button.is-active {
  background: var(--color-bg-elevated); color: var(--color-primary);
  box-shadow: var(--shadow-card);
}

/* 象限分组 */
.fl-q-group { margin-top: var(--sp-4); }
.fl-q-group:first-child { margin-top: 0; }
.fl-q-label {
  display: flex; align-items: center; gap: var(--sp-2);
  font-size: var(--fs-12); font-weight: var(--fw-medium);
  color: var(--color-text-secondary); margin-bottom: var(--sp-2);
}
.fl-q-dot { width: 8px; height: 8px; border-radius: 50%; }
.q1 .fl-q-dot { background: var(--color-q1); }
.q2 .fl-q-dot { background: var(--color-q2); }
.q3 .fl-q-dot { background: var(--color-q3); }
.q4 .fl-q-dot { background: var(--color-q4); }
.fl-q-count { color: var(--color-text-muted); font-size: 11px; }

/* 任务行 */
.fl-task-item {
  display: flex; align-items: center; gap: var(--sp-3);
  padding: var(--sp-3); border-radius: var(--r-sm);
  border: 1px solid transparent; cursor: pointer;
  transition: all var(--dur-base); margin-bottom: 4px;
}
.fl-task-item:hover { background: var(--color-bg-hover); }
.fl-task-item.is-done .fl-t-name { text-decoration: line-through; color: var(--color-text-muted); }

.fl-check {
  width: 20px; height: 20px; flex: 0 0 20px; border-radius: 50%;
  border: 1.5px solid var(--color-border-strong); background: transparent;
  color: transparent; cursor: pointer; display: grid; place-items: center;
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-check:hover, .fl-check.is-checked {
  border-color: var(--color-success); background: var(--color-success); color: #fff;
}
.fl-check.is-progress {
  border-color: var(--color-primary);
  background: color-mix(in srgb, var(--color-primary) 22%, transparent);
  color: var(--color-primary);
  position: relative;
}
.fl-check.is-progress::before {
  content: ""; width: 6px; height: 6px; border-radius: 50%;
  background: var(--color-primary);
}
.fl-check-sm { width: 16px; height: 16px; flex: 0 0 16px; }
.fl-check-sm.is-progress::before { width: 5px; height: 5px; }

.fl-t-body { flex: 1; min-width: 0; }
.fl-t-name { font-size: var(--fs-14); font-weight: var(--fw-medium); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.fl-t-meta { display: flex; align-items: center; gap: var(--sp-2); font-size: var(--fs-12); color: var(--color-text-muted); margin-top: 2px; }

.fl-chip { padding: 1px 6px; border-radius: var(--r-xs); font-size: 11px; }
.fl-chip-bg { background: color-mix(in srgb, var(--color-q4) 12%, transparent); color: var(--color-q4); }
.fl-chip-due { background: color-mix(in srgb, var(--color-q3) 12%, transparent); color: var(--color-q3); }
.fl-chip-goal { background: color-mix(in srgb, var(--color-primary) 12%, transparent); color: var(--color-primary); }

.fl-t-actions { display: flex; gap: 4px; opacity: 0; transition: opacity var(--dur-fast); }
.fl-task-item:hover .fl-t-actions { opacity: 1; }
.fl-t-btn {
  width: 24px; height: 24px; border: 1px solid var(--color-border);
  background: transparent; color: var(--color-text-muted); border-radius: var(--r-xs);
  cursor: pointer; display: grid; place-items: center;
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-t-btn:hover { border-color: var(--color-primary); color: var(--color-primary); }
.fl-t-btn:disabled { opacity: 0.3; cursor: not-allowed; }
.fl-t-play:hover { border-color: var(--color-success); color: var(--color-success); }
.fl-t-danger:hover { border-color: var(--color-q1); color: var(--color-q1); }

/* 添加任务栏 */
.fl-add-bar {
  display: flex; align-items: center; gap: var(--sp-2);
  margin-top: var(--sp-3); padding: var(--sp-3);
  border: 1px dashed var(--color-border-strong); border-radius: var(--r-sm);
  color: var(--color-text-muted); font-size: var(--fs-12);
  transition: all var(--dur-base);
}
.fl-add-bar:focus-within { border-color: var(--color-primary); color: var(--color-primary); }
.fl-add-bar input {
  flex: 1; border: none; background: none; outline: none;
  color: var(--color-text-primary); font-size: var(--fs-14);
}
.fl-add-bar input::placeholder { color: var(--color-text-muted); }
.fl-add-bar button[type="submit"] {
  padding: 4px 12px; border: none; border-radius: var(--r-sm);
  background: var(--color-primary); color: #fff; font-size: var(--fs-12);
  cursor: pointer;
}
.fl-add-bar button[type="submit"]:disabled { opacity: 0.4; cursor: not-allowed; }

.fl-bg-toggle { cursor: pointer; }
.fl-bg-chip {
  padding: 2px 8px; border-radius: var(--r-pill);
  border: 1px solid var(--color-border); font-size: 11px;
  color: var(--color-text-muted); user-select: none;
  transition: all var(--dur-fast);
}
.fl-bg-chip.is-on { border-color: var(--color-q4); color: var(--color-q4); background: color-mix(in srgb, var(--color-q4) 12%, transparent); }

.fl-quick-add-btn {
  width: 28px; height: 28px; border-radius: 50%;
  border: 1px solid var(--color-primary);
  background: var(--color-primary); color: #fff;
  font-size: 16px; font-weight: 600;
  cursor: pointer; display: grid; place-items: center;
  transition: opacity var(--dur-fast);
}
.fl-quick-add-btn:hover { opacity: 0.85; }
.fl-sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0,0,0,0); }

/* ---------- Right Rail ---------- */
.fl-ai-card {
  background: linear-gradient(180deg, var(--color-primary-soft), var(--color-bg-elevated));
  border: 1px solid color-mix(in srgb, var(--color-primary) 20%, var(--color-border));
  border-radius: var(--r-md); padding: var(--sp-4);
  display: flex; flex-direction: column; gap: var(--sp-3);
}
.fl-ai-head { display: flex; gap: var(--sp-3); }
.fl-ai-avatar {
  width: 32px; height: 32px; border-radius: var(--r-sm); flex-shrink: 0;
  background: var(--color-primary-soft); display: grid; place-items: center; font-size: 16px;
}
.fl-ai-title { font-weight: var(--fw-medium); margin-bottom: 2px; font-size: var(--fs-14); }
.fl-ai-text { font-size: var(--fs-12); color: var(--color-text-secondary); line-height: 1.6; }
.fl-ai-text.is-loading { opacity: 0.5; font-style: italic; }
.fl-ai-refresh {
  background: none; border: none; font-size: 14px;
  cursor: pointer; padding: 4px; border-radius: var(--r-sm);
  flex-shrink: 0; align-self: flex-start;
}
.fl-ai-refresh:hover { background: var(--color-bg-hover); }
.fl-ai-refresh:disabled { opacity: 0.3; cursor: not-allowed; }

.fl-ai-divider {
  height: 1px;
  background: color-mix(in srgb, var(--color-primary) 14%, transparent);
  margin: 0 calc(-1 * var(--sp-4));
}

.fl-ai-yesterday {
  display: flex; align-items: center; justify-content: space-between;
  gap: var(--sp-2); font-size: var(--fs-12);
}
.fl-ai-y-text {
  color: var(--color-text-secondary);
  min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.fl-ai-y-text strong {
  color: var(--color-text-primary);
  font-weight: var(--fw-semibold);
}
.fl-ai-y-link {
  display: inline-flex; align-items: center; gap: 2px;
  padding: 0; background: transparent; border: none;
  color: var(--color-primary); font-size: var(--fs-12);
  cursor: pointer; white-space: nowrap;
}
.fl-ai-y-link:hover { text-decoration: underline; }

.fl-stat-card {
  background: var(--color-bg-elevated); border: 1px solid var(--color-border);
  border-radius: var(--r-md); padding: var(--sp-4);
}
.fl-stat-card h3 {
  font-size: var(--fs-12); color: var(--color-text-muted);
  font-weight: var(--fw-medium); text-transform: uppercase;
  letter-spacing: 0.5px; margin: 0 0 var(--sp-3);
}

.fl-goal-top { display: flex; align-items: center; gap: var(--sp-2); }
.fl-goal-icon {
  width: 32px; height: 32px; border-radius: var(--r-sm);
  background: var(--color-primary-soft); display: grid; place-items: center; font-size: 16px;
}
.fl-goal-name { font-size: var(--fs-14); font-weight: var(--fw-semibold); }
.fl-goal-sub { font-size: var(--fs-12); color: var(--color-text-secondary); }

/* 进度卡 */
.fl-prog-top { display: flex; align-items: center; justify-content: space-between; }
.fl-prog-value { font-family: var(--font-mono); font-size: 26px; font-weight: var(--fw-bold); color: var(--color-primary); }
.fl-prog-value em { font-style: normal; font-size: 16px; font-weight: var(--fw-medium); color: var(--color-text-muted); }
.fl-prog-pct { font-size: 11px; color: var(--color-text-muted); margin-left: var(--sp-2); }
.fl-prog-bar { height: 6px; background: var(--color-bg-hover); border-radius: 3px; margin: var(--sp-3) 0; overflow: hidden; }
.fl-prog-fill { height: 100%; background: var(--color-primary); border-radius: 3px; transition: width var(--dur-slow) var(--ease-out); }

/* 今日计划迷你列表 */
.fl-plan-list { list-style: none; padding: 0; margin: 0 0 var(--sp-3); display: flex; flex-direction: column; gap: 4px; }
.fl-plan-item {
  display: flex; align-items: center; gap: var(--sp-2);
  padding: var(--sp-1) var(--sp-2); border-radius: var(--r-xs);
  font-size: var(--fs-12);
}
.fl-plan-item.is-done .fl-plan-name { text-decoration: line-through; color: var(--color-text-muted); }
.fl-plan-name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

/* 结束今天 CTA */
.fl-settle-cta {
  width: 100%; padding: var(--sp-3); border: none;
  border-radius: var(--r-md); background: var(--color-primary);
  color: #fff; font-size: var(--fs-14); font-weight: var(--fw-medium);
  cursor: pointer; display: flex; align-items: center; justify-content: center; gap: var(--sp-2);
  transition: background var(--dur-fast) var(--ease-smooth);
}
.fl-settle-cta:hover { background: var(--color-primary-dark); }
.fl-settle-cta:disabled { opacity: 0.4; cursor: not-allowed; }

.fl-empty {
  text-align: center; padding: var(--sp-6) var(--sp-4);
  color: var(--color-text-muted); font-size: var(--fs-14);
  background: var(--color-bg-subtle); border-radius: var(--r-md);
}

/* fade transition */
.fl-fade-enter-active, .fl-fade-leave-active { transition: opacity var(--dur-base) var(--ease-smooth); }
.fl-fade-enter-from, .fl-fade-leave-to { opacity: 0; }
</style>

<script setup lang="ts">
/**
 * MorningGuide · 晨起 6 步引导思考 — 对齐 prototype/onboarding/morning-guide.html。
 * Step 1: 回顾昨日 · Step 2: 长线目标 · Step 3: 固定日程
 * Step 4: AI 建议 · Step 5: 能量状态 · Step 6: 推荐任务（一键加入今日计划）
 */

import { computed, ref } from "vue";

import { useFixedSchedule } from "@/composables/useFixedSchedule";
import { useAIStore } from "@/stores/useAIStore";
import { useAssignmentStore } from "@/stores/useAssignmentStore";
import { useGoalStore } from "@/stores/useGoalStore";
import { useSettlementStore } from "@/stores/useSettlementStore";
import { useTaskStore } from "@/stores/useTaskStore";

defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: [] }>();

const settlement = useSettlementStore();
const goals = useGoalStore();
const ai = useAIStore();
const tasks = useTaskStore();
const assignments = useAssignmentStore();
const { byWeekday, totalMinutesForWeekday } = useFixedSchedule();

// Step 3 · 固定日程数据
const WEEKDAY_LABELS = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
const todayWeekday = new Date().getDay();
const todaySchedule = computed(() => byWeekday.value[todayWeekday] ?? []);
const todayWeekdayLabel = WEEKDAY_LABELS[todayWeekday];
const availableHours = computed(() => {
  const base = (todayWeekday === 0 || todayWeekday === 6) ? 5 * 60 : 9 * 60;
  const available = Math.max(0, base - totalMinutesForWeekday(todayWeekday));
  return (available / 60).toFixed(1);
});

const step = ref(1);
const energyLevel = ref<string | null>(null);
const aiSuggestion = ref("");
const loadingAI = ref(false);

// Step 6 · 推荐任务
const selectedTaskIds = ref<Set<string>>(new Set());
const addingTasks = ref(false);
const tasksAdded = ref(false);

/** 取前 3 条待办任务：优先重要紧急象限，其次按有预估时长排序 */
const recommendedTasks = computed(() => {
  const pending = tasks.tasks.filter(
    (t) => t.status === "pending" || t.status === "in_progress"
  );
  const sorted = [...pending].sort((a, b) => {
    const qScore = (q: string) => (q === "important_urgent" ? 0 : q === "important_not_urgent" ? 1 : 2);
    const qs = qScore(a.quadrant ?? "") - qScore(b.quadrant ?? "");
    if (qs !== 0) return qs;
    const ae = a.estimated_minutes ?? 0;
    const be = b.estimated_minutes ?? 0;
    return be - ae;
  });
  return sorted.slice(0, 3);
});

function toggleTask(id: string) {
  const s = new Set(selectedTaskIds.value);
  if (s.has(id)) s.delete(id);
  else s.add(id);
  selectedTaskIds.value = s;
}

function selectAll() {
  selectedTaskIds.value = new Set(recommendedTasks.value.map((t) => t.id));
}

const ENERGY_OPTIONS = [
  { value: "low", emoji: "🌙", label: "比较累", desc: "轻量任务为主" },
  { value: "normal", emoji: "☀️", label: "状态一般", desc: "正常安排" },
  { value: "good", emoji: "⚡", label: "状态不错", desc: "可以推核心任务" },
  { value: "great", emoji: "🔥", label: "精神饱满", desc: "冲刺模式" },
];

const STEPS = [
  { num: 1, label: "回顾昨日" },
  { num: 2, label: "长线目标" },
  { num: 3, label: "固定日程" },
  { num: 4, label: "AI 建议" },
  { num: 5, label: "能量状态" },
  { num: 6, label: "今日计划" },
];

const yesterday = computed(() => settlement.yesterday);

function nextStep() {
  if (step.value < 6) {
    step.value++;
    if (step.value === 4) loadAISuggestion();
    if (step.value === 6) {
      if (!tasks.tasks.length) {
        void tasks.load().then(() => {
          selectedTaskIds.value = new Set(recommendedTasks.value.map((t) => t.id));
        });
      } else {
        selectedTaskIds.value = new Set(recommendedTasks.value.map((t) => t.id));
      }
    }
  } else {
    void finish();
  }
}

function prevStep() {
  if (step.value > 1) step.value--;
}

async function loadAISuggestion() {
  loadingAI.value = true;
  try {
    aiSuggestion.value = await ai.dailySuggestions(energyLevel.value ?? "正常");
  } catch {
    aiSuggestion.value = "暂时无法生成建议，请先配置 AI 服务。";
  } finally {
    loadingAI.value = false;
  }
}

async function finish() {
  if (energyLevel.value) {
    const today = new Date().toISOString().slice(0, 10);
    localStorage.setItem(`fl-energy-${today}`, energyLevel.value);
  }
  localStorage.setItem(`fl-morning-${new Date().toISOString().slice(0, 10)}`, "done");

  if (selectedTaskIds.value.size > 0 && !tasksAdded.value) {
    addingTasks.value = true;
    try {
      await Promise.all(
        [...selectedTaskIds.value].map((taskId) =>
          assignments.create({ taskId, source: "guided", isPlanned: true })
        )
      );
      tasksAdded.value = true;
    } catch (e) {
      console.error("[MorningGuide] 加入今日计划失败", e);
    } finally {
      addingTasks.value = false;
    }
  }

  step.value = 1;
  selectedTaskIds.value = new Set();
  tasksAdded.value = false;
  emit("close");
}
</script>

<template>
  <Transition name="fl-fade">
    <div v-if="visible" class="fl-mg-mask" @click.self="emit('close')">
      <div class="fl-mg-card">
        <div class="fl-mg-hero">
          <div class="fl-mg-emoji">🌅</div>
          <h2>晨起引导</h2>
          <p>6 步思考，开启高效的一天</p>
        </div>

        <div class="fl-mg-stepper">
          <div
            v-for="s in STEPS"
            :key="s.num"
            class="fl-mg-step"
            :class="{ 'is-done': s.num < step, 'is-active': s.num === step }"
          >
            <span class="fl-mg-step-num">{{ s.num }}</span>
            <span class="fl-mg-step-label">{{ s.label }}</span>
          </div>
        </div>

        <div class="fl-mg-body">
          <template v-if="step === 1">
            <h3>回顾昨日</h3>
            <div v-if="yesterday" class="fl-mg-yesterday">
              <div class="fl-mg-grade-badge">{{ yesterday.grade }}</div>
              <div>
                <div>{{ yesterday.completedTasks }}/{{ yesterday.totalTasks }} 完成 · 专注 {{ yesterday.totalFocusMinutes }}m</div>
                <div v-if="yesterday.carriedOverCount > 0" class="fl-mg-carry">
                  {{ yesterday.carriedOverCount }} 项任务已带入今天
                </div>
              </div>
            </div>
            <div v-else class="fl-mg-empty">没有昨日数据，跳过即可</div>
          </template>

          <template v-else-if="step === 2">
            <h3>审视长线目标</h3>
            <div v-if="goals.goals.length" class="fl-mg-goals">
              <div v-for="g in goals.goals.slice(0, 3)" :key="g.id" class="fl-mg-goal">
                <span class="fl-mg-goal-icon">🎯</span>
                <div>
                  <div class="fl-mg-goal-name">{{ g.name }}</div>
                  <div class="fl-mg-goal-sub">{{ g.status }}</div>
                </div>
              </div>
            </div>
            <div v-else class="fl-mg-empty">还没有设置长线目标</div>
          </template>

          <template v-else-if="step === 3">
            <h3>检查固定日程</h3>
            <p class="fl-mg-sub">看看今天({{ todayWeekdayLabel }})有哪些已经占掉的时间</p>
            <div v-if="todaySchedule.length" class="fl-mg-schedule">
              <div v-for="s in todaySchedule" :key="s.id" class="fl-mg-sch-row">
                <span class="fl-mg-sch-time">{{ s.startTime }}–{{ s.endTime }}</span>
                <span class="fl-mg-sch-title">{{ s.title }}</span>
              </div>
            </div>
            <div v-else class="fl-mg-empty">今天没有固定日程 · 整天都可以安排深度工作</div>
            <div class="fl-mg-available">
              <span>今天可用于深度工作</span>
              <strong>{{ availableHours }}h</strong>
            </div>
          </template>

          <template v-else-if="step === 4">
            <h3>AI 今日建议</h3>
            <div v-if="loadingAI" class="fl-mg-loading">正在生成建议…</div>
            <div v-else class="fl-mg-ai-suggestion">
              <div class="fl-mg-ai-avatar">✨</div>
              <div class="fl-mg-ai-text">{{ aiSuggestion }}</div>
            </div>
            <button v-if="!loadingAI" class="fl-mg-regen" @click="loadAISuggestion">
              🔄 重新生成
            </button>
          </template>

          <template v-else-if="step === 5">
            <h3>今日能量状态</h3>
            <div class="fl-mg-energy-grid">
              <button
                v-for="e in ENERGY_OPTIONS"
                :key="e.value"
                class="fl-mg-energy"
                :class="{ 'is-selected': energyLevel === e.value }"
                @click="energyLevel = e.value"
              >
                <span class="fl-mg-energy-emoji">{{ e.emoji }}</span>
                <span class="fl-mg-energy-label">{{ e.label }}</span>
                <span class="fl-mg-energy-desc">{{ e.desc }}</span>
              </button>
            </div>
          </template>

          <template v-else>
            <h3>加入今日计划</h3>
            <p class="fl-mg-sub">选择今天想完成的任务，一键加入计划</p>
            <div v-if="recommendedTasks.length" class="fl-mg-task-list">
              <label
                v-for="t in recommendedTasks"
                :key="t.id"
                class="fl-mg-task-item"
                :class="{ 'is-selected': selectedTaskIds.has(t.id) }"
              >
                <input
                  type="checkbox"
                  class="fl-mg-task-check"
                  :checked="selectedTaskIds.has(t.id)"
                  @change="toggleTask(t.id)"
                />
                <span class="fl-mg-task-quadrant">
                  {{ t.quadrant === 'important_urgent' ? '🔴' : t.quadrant === 'important_not_urgent' ? '🟡' : '⬜' }}
                </span>
                <span class="fl-mg-task-name">{{ t.name }}</span>
                <span v-if="t.estimated_minutes" class="fl-mg-task-est">{{ t.estimated_minutes }}m</span>
              </label>
              <button
                v-if="recommendedTasks.length > 1"
                class="fl-mg-regen"
                style="margin-top: var(--sp-2)"
                @click="selectAll"
              >
                ✅ 全选
              </button>
            </div>
            <div v-else class="fl-mg-empty">暂无待办任务，跳过即可</div>
          </template>
        </div>

        <div class="fl-mg-foot">
          <button v-if="step > 1" class="fl-mg-btn fl-mg-ghost" @click="prevStep">上一步</button>
          <button class="fl-mg-btn fl-mg-ghost" @click="emit('close')">跳过</button>
          <button class="fl-mg-btn fl-mg-primary" :disabled="addingTasks" @click="nextStep">
            {{ addingTasks ? '加入中…' : step < 6 ? '下一步' : '开始今天 🚀' }}
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fl-mg-mask {
  position: fixed; inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 32%, transparent);
  display: grid; place-items: center;
  z-index: var(--z-modal); padding: var(--sp-4);
}
.fl-mg-card {
  width: min(520px, 100%); max-height: 85vh; overflow-y: auto;
  background: var(--color-bg-elevated); border: 1px solid var(--color-border);
  border-radius: var(--r-lg); box-shadow: var(--shadow-modal);
}

.fl-mg-hero {
  padding: var(--sp-5); text-align: center;
  background: linear-gradient(135deg, #FFF8E1, #FFE0EC);
}
.fl-mg-emoji { font-size: 40px; margin-bottom: var(--sp-2); }
.fl-mg-hero h2 { font-size: var(--fs-20, 20px); font-weight: var(--fw-semibold); margin: 0; }
.fl-mg-hero p { font-size: var(--fs-12); color: var(--color-text-secondary); margin: var(--sp-1) 0 0; }

.fl-mg-stepper {
  display: flex; gap: var(--sp-1); padding: var(--sp-3) var(--sp-5);
  border-bottom: 1px solid var(--color-border);
}
.fl-mg-step {
  flex: 1; display: flex; align-items: center; gap: var(--sp-1);
  font-size: 11px; color: var(--color-text-muted);
}
.fl-mg-step-num {
  width: 20px; height: 20px; border-radius: 50%; display: grid; place-items: center;
  font-size: 10px; font-weight: var(--fw-semibold);
  border: 1.5px solid var(--color-border); background: transparent;
}
.fl-mg-step.is-active .fl-mg-step-num { background: var(--color-primary); border-color: var(--color-primary); color: #fff; }
.fl-mg-step.is-done .fl-mg-step-num { background: var(--color-success); border-color: var(--color-success); color: #fff; }
.fl-mg-step.is-active { color: var(--color-text-primary); font-weight: var(--fw-medium); }
.fl-mg-step-label { display: none; }
@media (min-width: 480px) { .fl-mg-step-label { display: inline; } }

.fl-mg-body { padding: var(--sp-5); min-height: 180px; }
.fl-mg-body h3 { font-size: var(--fs-16); font-weight: var(--fw-semibold); margin: 0 0 var(--sp-4); }

.fl-mg-yesterday {
  display: flex; gap: var(--sp-3); align-items: center;
  padding: var(--sp-3); background: var(--color-bg-subtle); border-radius: var(--r-md);
  font-size: var(--fs-14);
}
.fl-mg-grade-badge {
  width: 48px; height: 48px; border-radius: 50%; display: grid; place-items: center;
  font-size: 24px; font-weight: var(--fw-bold); color: #fff;
  background: linear-gradient(135deg, var(--color-gold, #FAAD14), #FFD666);
}
.fl-mg-carry { font-size: var(--fs-12); color: var(--color-text-muted); margin-top: 2px; }

.fl-mg-goals { display: flex; flex-direction: column; gap: var(--sp-2); }
.fl-mg-goal {
  display: flex; gap: var(--sp-3); align-items: center;
  padding: var(--sp-3); border: 1px solid var(--color-border); border-radius: var(--r-md);
}
.fl-mg-goal-icon { font-size: 20px; }
.fl-mg-goal-name { font-size: var(--fs-14); font-weight: var(--fw-medium); }
.fl-mg-goal-sub { font-size: var(--fs-12); color: var(--color-text-muted); }

.fl-mg-sub {
  font-size: var(--fs-12); color: var(--color-text-secondary);
  margin: 0 0 var(--sp-3); line-height: 1.5;
}
.fl-mg-schedule {
  display: flex; flex-direction: column; gap: 6px;
  padding: var(--sp-3); background: var(--color-bg-subtle); border-radius: var(--r-md);
  margin-bottom: var(--sp-3);
}
.fl-mg-sch-row {
  display: grid; grid-template-columns: auto 1fr; gap: var(--sp-3); align-items: center;
  font-size: var(--fs-13, 13px);
}
.fl-mg-sch-time {
  font-family: var(--font-mono); font-size: 11px;
  color: var(--color-text-secondary);
  padding: 2px 8px; background: var(--color-bg-elevated); border-radius: var(--r-pill);
}
.fl-mg-sch-title { color: var(--color-text-primary); font-weight: var(--fw-medium); }
.fl-mg-available {
  display: flex; align-items: baseline; justify-content: space-between;
  padding: var(--sp-3); background: var(--color-primary-soft);
  border: 1px solid color-mix(in srgb, var(--color-primary) 25%, transparent);
  border-radius: var(--r-md); font-size: var(--fs-13, 13px); color: var(--color-text-secondary);
}
.fl-mg-available strong {
  font-family: var(--font-mono); color: var(--color-primary);
  font-size: var(--fs-16); font-weight: var(--fw-bold);
}

.fl-mg-loading { color: var(--color-text-muted); font-size: var(--fs-14); }
.fl-mg-ai-suggestion {
  display: flex; gap: var(--sp-3); padding: var(--sp-3);
  background: var(--color-bg-subtle); border-radius: var(--r-md); border: 1px dashed var(--color-border);
}
.fl-mg-ai-avatar { font-size: 20px; flex-shrink: 0; }
.fl-mg-ai-text { font-size: var(--fs-14); color: var(--color-text-secondary); line-height: 1.6; }

.fl-mg-regen {
  background: none; border: 1px solid var(--color-border); border-radius: var(--r-sm);
  padding: var(--sp-1) var(--sp-3); font-size: var(--fs-12); color: var(--color-text-muted);
  cursor: pointer; margin-top: var(--sp-2);
}
.fl-mg-regen:hover { border-color: var(--color-primary); color: var(--color-primary); }

.fl-mg-energy-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: var(--sp-2); }
.fl-mg-energy {
  display: flex; flex-direction: column; align-items: center; gap: var(--sp-1);
  padding: var(--sp-3); border: 1px solid var(--color-border); border-radius: var(--r-md);
  background: transparent; cursor: pointer; transition: all var(--dur-fast);
}
.fl-mg-energy:hover { background: var(--color-bg-hover); }
.fl-mg-energy.is-selected { border-color: var(--color-primary); background: var(--color-primary-soft); }
.fl-mg-energy-emoji { font-size: 28px; }
.fl-mg-energy-label { font-size: var(--fs-14); font-weight: var(--fw-medium); }
.fl-mg-energy-desc { font-size: var(--fs-12); color: var(--color-text-muted); }

.fl-mg-task-list {
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}
.fl-mg-task-item {
  display: grid;
  grid-template-columns: auto auto 1fr auto;
  align-items: center;
  gap: var(--sp-2);
  padding: var(--sp-3);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  cursor: pointer;
  background: var(--color-bg-elevated);
  transition: all var(--dur-fast);
}
.fl-mg-task-item:hover {
  background: var(--color-bg-hover);
}
.fl-mg-task-item.is-selected {
  border-color: var(--color-primary);
  background: var(--color-primary-soft);
}
.fl-mg-task-check {
  accent-color: var(--color-primary);
}
.fl-mg-task-name {
  font-size: var(--fs-14);
  color: var(--color-text-primary);
}
.fl-mg-task-est {
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
  font-family: var(--font-mono);
}
.fl-mg-task-quadrant {
  font-size: 14px;
}

.fl-mg-empty { color: var(--color-text-muted); font-size: var(--fs-14); text-align: center; padding: var(--sp-6); }

.fl-mg-foot {
  display: flex; gap: var(--sp-2); justify-content: flex-end;
  padding: var(--sp-4) var(--sp-5); border-top: 1px solid var(--color-border);
}
.fl-mg-btn {
  padding: var(--sp-2) var(--sp-4); border-radius: var(--r-md);
  font-size: var(--fs-12); font-weight: var(--fw-medium); cursor: pointer; border: none;
}
.fl-mg-primary { background: var(--color-primary); color: #fff; }
.fl-mg-primary:hover { background: var(--color-primary-dark); }
.fl-mg-primary:disabled { opacity: 0.65; cursor: not-allowed; }
.fl-mg-ghost { background: transparent; color: var(--color-text-secondary); border: 1px solid var(--color-border); }

.fl-fade-enter-active, .fl-fade-leave-active { transition: opacity var(--dur-base) var(--ease-smooth); }
.fl-fade-enter-from, .fl-fade-leave-to { opacity: 0; }
</style>

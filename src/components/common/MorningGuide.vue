<script setup lang="ts">
/**
 * MorningGuide · 晨起 5 步引导思考 — 对齐 prototype/onboarding/morning-guide.html。
 * Step 1: 回顾昨日 · Step 2: 长线目标 · Step 3: 固定日程
 * Step 4: AI 建议 · Step 5: 能量状态
 */

import { computed, ref } from "vue";

import { useAIStore } from "@/stores/useAIStore";
import { useGoalStore } from "@/stores/useGoalStore";
import { useSettlementStore } from "@/stores/useSettlementStore";

defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: [] }>();

const settlement = useSettlementStore();
const goals = useGoalStore();
const ai = useAIStore();

const step = ref(1);
const energyLevel = ref<string | null>(null);
const aiSuggestion = ref("");
const loadingAI = ref(false);

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
];

const yesterday = computed(() => settlement.yesterday);

function nextStep() {
  if (step.value < 5) {
    step.value++;
    if (step.value === 4) loadAISuggestion();
  } else {
    finish();
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

function finish() {
  // 存储今日能量状态
  if (energyLevel.value) {
    const today = new Date().toISOString().slice(0, 10);
    localStorage.setItem(`fl-energy-${today}`, energyLevel.value);
  }
  localStorage.setItem(`fl-morning-${new Date().toISOString().slice(0, 10)}`, "done");
  step.value = 1;
  emit("close");
}
</script>

<template>
  <Transition name="fl-fade">
    <div v-if="visible" class="fl-mg-mask" @click.self="emit('close')">
      <div class="fl-mg-card">
        <!-- Hero -->
        <div class="fl-mg-hero">
          <div class="fl-mg-emoji">🌅</div>
          <h2>晨起引导</h2>
          <p>5 步思考，开启高效的一天</p>
        </div>

        <!-- 步进器 -->
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

        <!-- Step 内容 -->
        <div class="fl-mg-body">
          <!-- Step 1: 回顾昨日 -->
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

          <!-- Step 2: 长线目标 -->
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

          <!-- Step 3: 固定日程 -->
          <template v-else-if="step === 3">
            <h3>检查固定日程</h3>
            <div class="fl-mg-empty">
              固定日程功能开发中，可直接跳过
            </div>
          </template>

          <!-- Step 4: AI 建议 -->
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

          <!-- Step 5: 能量状态 -->
          <template v-else>
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
        </div>

        <!-- 底部操作 -->
        <div class="fl-mg-foot">
          <button v-if="step > 1" class="fl-mg-btn fl-mg-ghost" @click="prevStep">上一步</button>
          <button class="fl-mg-btn fl-mg-ghost" @click="emit('close')">跳过</button>
          <button class="fl-mg-btn fl-mg-primary" @click="nextStep">
            {{ step < 5 ? '下一步' : '开始今天' }}
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

/* Yesterday */
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

/* Goals */
.fl-mg-goals { display: flex; flex-direction: column; gap: var(--sp-2); }
.fl-mg-goal {
  display: flex; gap: var(--sp-3); align-items: center;
  padding: var(--sp-3); border: 1px solid var(--color-border); border-radius: var(--r-md);
}
.fl-mg-goal-icon { font-size: 20px; }
.fl-mg-goal-name { font-size: var(--fs-14); font-weight: var(--fw-medium); }
.fl-mg-goal-sub { font-size: var(--fs-12); color: var(--color-text-muted); }

/* AI */
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

/* Energy */
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

.fl-mg-empty { color: var(--color-text-muted); font-size: var(--fs-14); text-align: center; padding: var(--sp-6); }

/* Footer */
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
.fl-mg-ghost { background: transparent; color: var(--color-text-secondary); border: 1px solid var(--color-border); }

.fl-fade-enter-active, .fl-fade-leave-active { transition: opacity var(--dur-base) var(--ease-smooth); }
.fl-fade-enter-from, .fl-fade-leave-to { opacity: 0; }
</style>

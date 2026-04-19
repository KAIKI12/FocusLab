<script setup lang="ts">
/**
 * SettlementDialog · 日结算弹窗 — 对齐 prototype/screens/settlement.html。
 * S/A/B/C 四套渐变头部 + 详细数据 + 时间分布 + AI 寄语 + 感想 + 温和未完成处理。
 */

import { computed, ref, watch } from "vue";

import ParticleEffect from "@/components/settlement/ParticleEffect.vue";
import { invokeCmd } from "@/composables/useTauriInvoke";
import { useAIStore } from "@/stores/useAIStore";
import { useSettlementStore } from "@/stores/useSettlementStore";
import type { AssignmentWithTask } from "@/types";

const store = useSettlementStore();
const ai = useAIStore();
const s = computed(() => store.settlement);
const reflection = ref("");
const aiNarrative = ref("");
const loadingAI = ref(false);
const pendingTasks = ref<AssignmentWithTask[]>([]);

// 结算打开时尝试生成 AI 叙事
watch(
  () => store.showDialog,
  async (v) => {
    if (v && s.value) {
      reflection.value = "";
      aiNarrative.value = "";
      try {
        loadingAI.value = true;
        aiNarrative.value = await ai.generateNarrative(
          s.value.grade, s.value.completedTasks, s.value.totalTasks,
          s.value.totalFocusMinutes,
        );
      } catch { /* AI 可选 */ } finally { loadingAI.value = false; }
      // 加载未完成的 DTA 任务
      try {
        const all = await invokeCmd<AssignmentWithTask[]>("list_assignments", { planDate: s.value.settleDate });
        pendingTasks.value = all.filter(a => a.dayStatus === "pending");
      } catch { pendingTasks.value = []; }
    }
  },
);

const GRADE_CONFIG: Record<string, {
  emoji: string; title: string; desc: string; heroCls: string;
}> = {
  S: { emoji: "✨", title: "超额完成 · 加冕日", desc: "你今天真的很棒", heroCls: "fl-hero-s" },
  A: { emoji: "🌟", title: "完美完成", desc: "计划内全部搞定", heroCls: "fl-hero-a" },
  B: { emoji: "☁️", title: "基本完成", desc: "也是扎实的一天", heroCls: "fl-hero-b" },
  C: { emoji: "🌱", title: "今天有点忙", desc: "没关系 · 明天重新开始", heroCls: "fl-hero-c" },
};

const cfg = computed(() => GRADE_CONFIG[s.value?.grade ?? "C"] ?? GRADE_CONFIG.C);

const MORNING_INTENT_LABELS: Record<number, { emoji: string; label: string }> = {
  1: { emoji: "🌙", label: "保养档" },
  2: { emoji: "🌤", label: "温和档" },
  3: { emoji: "☀️", label: "常规档" },
  4: { emoji: "⚡", label: "进阶档" },
  5: { emoji: "🔥", label: "冲刺档" },
};
const EVENING_MOOD_LABELS: Record<number, { emoji: string; label: string }> = {
  1: { emoji: "😔", label: "疲惫" },
  2: { emoji: "😕", label: "一般" },
  3: { emoji: "😐", label: "还行" },
  4: { emoji: "🙂", label: "不错" },
  5: { emoji: "😍", label: "很好" },
};

const morningIntent = computed(() =>
  s.value?.morningIntent ? MORNING_INTENT_LABELS[s.value.morningIntent] : null,
);
const eveningMood = computed(() =>
  s.value?.eveningMood ? EVENING_MOOD_LABELS[s.value.eveningMood] : null,
);
const hasMood = computed(() => !!(morningIntent.value || eveningMood.value));

const rateText = computed(() => {
  if (!s.value) return "0%";
  return `${Math.round(s.value.completionRate * 100)}%`;
});

const barWidth = computed(() => {
  if (!s.value) return "0%";
  return `${Math.min(Math.round(s.value.completionRate * 100), 120)}%`;
});

function fmtMin(m: number): string {
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  const r = m % 60;
  return r > 0 ? `${h}h ${r}m` : `${h}h`;
}

function onClose() {
  store.closeDialog();
}

async function carryOver(a: AssignmentWithTask) {
  try {
    await invokeCmd("update_assignment_status", { id: a.id, dayStatus: "carried_forward" });
    pendingTasks.value = pendingTasks.value.filter(t => t.id !== a.id);
  } catch (e) { console.error("[settle] carryOver failed", e); }
}

async function shelveTask(a: AssignmentWithTask) {
  try {
    await invokeCmd("delete_task", { id: a.taskId });
    pendingTasks.value = pendingTasks.value.filter(t => t.id !== a.id);
  } catch { /* */ }
}

async function markDone(a: AssignmentWithTask) {
  try {
    await invokeCmd("update_assignment_status", { id: a.id, dayStatus: "completed" });
    pendingTasks.value = pendingTasks.value.filter(t => t.id !== a.id);
  } catch { /* */ }
}
</script>

<template>
  <Transition name="fl-fade">
    <div
      v-if="store.showDialog && s"
      class="fl-sd-mask"
      role="presentation"
    >
      <div class="fl-sd-card" role="dialog" aria-modal="true">
        <!-- Hero 头部(按等级变色) -->
        <div class="fl-sd-hero" :class="cfg.heroCls">
          <!-- S/A 级粒子特效 -->
          <ParticleEffect v-if="s.grade === 'S' || s.grade === 'A'" :grade="s.grade" />

          <div class="fl-hero-date">{{ s.settleDate }}</div>
          <div class="fl-hero-title">{{ cfg.emoji }} {{ cfg.title }}</div>
          <div class="fl-hero-grade">{{ s.grade }}</div>
          <div class="fl-hero-desc">{{ cfg.desc }}</div>

          <!-- 进度条 -->
          <div class="fl-hero-progress">
            <div class="fl-hero-bar" :style="{ width: barWidth }" />
          </div>
          <div class="fl-hero-pct">
            {{ rateText }}
            <small>{{ s.completedTasks }} / {{ s.totalTasks - s.extraTasks }}
              <template v-if="s.extraTasks > 0"> + {{ s.extraTasks }} 额外</template>
            </small>
          </div>
        </div>

        <!-- 数据区 -->
        <div class="fl-sd-body">
          <div class="fl-section-label">📊 今日数据</div>
          <div class="fl-data-rows">
            <div class="fl-data-row">
              <span class="fl-data-label">总专注</span>
              <span class="fl-data-value">{{ fmtMin(s.totalFocusMinutes) }}</span>
            </div>
            <div class="fl-data-row fl-data-sub">
              <span>🍅 番茄</span>
              <span>{{ s.totalPomodoros }} 个</span>
            </div>
            <div class="fl-data-row">
              <span class="fl-data-label">完成任务</span>
              <span class="fl-data-value">
                {{ s.completedTasks }} / {{ s.totalTasks }}
              </span>
            </div>
            <div class="fl-data-row">
              <span class="fl-data-label">中断次数</span>
              <span class="fl-data-value">{{ s.totalInterruptions }} 次</span>
            </div>
            <div v-if="s.longestFocusMinutes" class="fl-data-row">
              <span class="fl-data-label">最久专注</span>
              <span class="fl-data-value">{{ fmtMin(s.longestFocusMinutes) }}</span>
            </div>
          </div>

          <!-- AI 寄语 -->
          <div v-if="aiNarrative || loadingAI" class="fl-ai-msg">
            <div class="fl-ai-avatar">✨</div>
            <div class="fl-ai-body">
              <template v-if="loadingAI">正在生成…</template>
              <template v-else>{{ aiNarrative }}</template>
            </div>
          </div>

          <!-- 今日心情 -->
          <template v-if="hasMood">
            <div class="fl-section-label">🌤 今日心情</div>
            <div class="fl-mood-pair">
              <div v-if="morningIntent" class="fl-mood-slot">
                <span class="fl-mood-emoji">{{ morningIntent.emoji }}</span>
                <div class="fl-mood-txt">
                  <small>早晨意图</small>
                  <strong>{{ morningIntent.label }}</strong>
                </div>
              </div>
              <div v-if="eveningMood" class="fl-mood-slot">
                <span class="fl-mood-emoji">{{ eveningMood.emoji }}</span>
                <div class="fl-mood-txt">
                  <small>晚间情绪</small>
                  <strong>{{ eveningMood.label }}</strong>
                </div>
              </div>
            </div>
          </template>

          <!-- 感想输入 -->
          <div class="fl-reflect">
            <span style="color:var(--color-text-muted)">✏️</span>
            <input
              v-model="reflection"
              type="text"
              :placeholder="s.grade === 'C' ? '写给明天的自己...' : '今日感想 (可选)'"
              maxlength="120"
            />
            <span class="fl-reflect-count">{{ reflection.length }} / 120</span>
          </div>

          <!-- 温和未完成处理(对齐原型 gentle-card) -->
          <div v-if="pendingTasks.length" class="fl-gentle">
            <div class="fl-gentle-head">
              <span>📋</span>
              <div>
                <strong>这些任务今天没有完成</strong>
                <div class="fl-gentle-sub">不用给它们贴"失败"的标签 · 每个任务都给你三个体面的出口</div>
              </div>
            </div>
            <div v-for="a in pendingTasks" :key="a.id" class="fl-gentle-task">
              <div class="fl-gentle-name">{{ a.taskName }}</div>
              <div class="fl-gentle-options">
                <button class="fl-gentle-btn fl-gentle-primary" @click="carryOver(a)">明天继续</button>
                <button class="fl-gentle-btn" @click="shelveTask(a)">搁置</button>
                <button class="fl-gentle-btn" @click="markDone(a)">✓ 已差不多了</button>
              </div>
            </div>
            <div class="fl-gentle-quote">
              🙂 没关系，计划赶不上变化是常态。重要的是你今天确实在推进。
            </div>
          </div>
        </div>

        <!-- 底部操作 -->
        <div class="fl-sd-foot">
          <button class="fl-sd-btn fl-sd-btn-ghost" type="button" @click="onClose">
            关闭
          </button>
          <button class="fl-sd-btn fl-sd-btn-primary" type="button" @click="onClose">
            保存结算
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fl-sd-mask {
  position: fixed;
  inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 36%, transparent);
  display: grid;
  place-items: center;
  z-index: var(--z-modal);
  padding: var(--sp-4);
}

.fl-sd-card {
  width: min(480px, 100%);
  max-height: 90vh;
  overflow-y: auto;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-modal);
  display: flex;
  flex-direction: column;
}

/* ---------- Hero ---------- */
.fl-sd-hero {
  position: relative;
  padding: var(--sp-6) var(--sp-5) var(--sp-5);
  text-align: center;
  overflow: hidden;
  color: #fff;
}

.fl-hero-s { background: linear-gradient(135deg, var(--color-gold, #FAAD14), var(--color-q3, #FF8C00) 60%, var(--color-q1, #F56C6C)); }
.fl-hero-a { background: linear-gradient(135deg, var(--color-gold, #FAAD14), #FFD666); }
.fl-hero-b { background: linear-gradient(135deg, var(--color-primary), var(--color-primary-light)); }
.fl-hero-c { background: linear-gradient(135deg, #8C8C8C, #B2B6C2); }

.fl-hero-date { font-size: var(--fs-12); opacity: 0.85; margin-bottom: var(--sp-3); }
.fl-hero-title { font-size: var(--fs-16); font-weight: var(--fw-medium); margin-bottom: var(--sp-3); }
.fl-hero-grade {
  font-size: 80px; font-weight: var(--fw-bold); line-height: 1;
  letter-spacing: -3px; margin-bottom: var(--sp-2);
  text-shadow: 0 4px 20px rgba(0,0,0,0.15);
}
.fl-hero-s .fl-hero-grade {
  background: linear-gradient(180deg, #FFF8E1, #FFE58F);
  -webkit-background-clip: text; background-clip: text; -webkit-text-fill-color: transparent;
  filter: drop-shadow(0 0 16px rgba(255,235,130,0.5));
}
.fl-hero-desc { font-size: var(--fs-14); opacity: 0.95; }

.fl-hero-progress {
  margin-top: var(--sp-4); height: 10px;
  background: rgba(255,255,255,0.3); border-radius: var(--r-pill); overflow: hidden;
}
.fl-hero-bar { height: 100%; border-radius: var(--r-pill); }
.fl-hero-s .fl-hero-bar {
  background: linear-gradient(90deg, #FFB347, #FFD700, #FF7A7A, #B87FFF, var(--color-primary));
  background-size: 200% 100%;
  animation: shimmer 3s linear infinite;
  box-shadow: 0 0 16px rgba(255,221,102,0.7);
}
@keyframes shimmer { 0% { background-position: 0% 0; } 100% { background-position: 200% 0; } }
.fl-hero-a .fl-hero-bar { background: #FFF8E1; box-shadow: 0 0 12px rgba(255,235,130,0.8); }
.fl-hero-b .fl-hero-bar { background: rgba(255,255,255,0.92); }
.fl-hero-c .fl-hero-bar { background: rgba(255,255,255,0.7); }

.fl-hero-pct {
  margin-top: var(--sp-3); font-family: var(--font-mono);
  font-size: var(--fs-24); font-weight: var(--fw-semibold); letter-spacing: -0.5px;
}
.fl-hero-pct small { font-size: var(--fs-12); opacity: 0.7; font-family: var(--font-sans); margin-left: 6px; }

/* ---------- Body ---------- */
.fl-sd-body {
  padding: var(--sp-5);
  display: flex; flex-direction: column; gap: var(--sp-4);
}

.fl-section-label {
  font-size: var(--fs-12); text-transform: uppercase; letter-spacing: 0.5px;
  color: var(--color-text-muted); font-weight: var(--fw-medium);
}

.fl-data-rows { display: flex; flex-direction: column; gap: 8px; }
.fl-data-row {
  display: flex; align-items: center; justify-content: space-between;
  font-size: var(--fs-14);
}
.fl-data-label { color: var(--color-text-secondary); }
.fl-data-value { font-family: var(--font-mono); font-weight: var(--fw-semibold); }
.fl-data-sub { padding-left: var(--sp-4); font-size: var(--fs-12); color: var(--color-text-secondary); }

.fl-ai-msg {
  display: flex; gap: var(--sp-2); padding: var(--sp-3);
  border-radius: var(--r-sm); background: var(--color-bg-subtle);
  border: 1px solid var(--color-border); font-size: var(--fs-12); line-height: 1.6;
}
.fl-ai-avatar {
  width: 28px; height: 28px; border-radius: 50%; flex-shrink: 0;
  background: linear-gradient(135deg, var(--color-primary), #B87FFF);
  display: grid; place-items: center; font-size: 14px;
}
.fl-ai-body { color: var(--color-text-secondary); }

/* 今日心情 pair */
.fl-mood-pair {
  display: grid; grid-template-columns: 1fr 1fr; gap: var(--sp-2);
}
.fl-mood-pair .fl-mood-slot {
  display: flex; align-items: center; gap: var(--sp-2);
  padding: var(--sp-2) var(--sp-3);
  background: var(--color-bg-subtle);
  border: 1px solid var(--color-border);
  border-radius: var(--r-sm);
}
.fl-mood-pair .fl-mood-emoji { font-size: 22px; line-height: 1; }
.fl-mood-txt { display: flex; flex-direction: column; min-width: 0; }
.fl-mood-txt small { font-size: 10px; color: var(--color-text-muted); }
.fl-mood-txt strong { font-size: var(--fs-12); font-weight: var(--fw-medium); }

.fl-reflect {
  display: flex; gap: var(--sp-2); align-items: center;
  padding: 10px 12px; background: var(--color-bg-subtle);
  border: 1px solid var(--color-border); border-radius: var(--r-sm);
}
.fl-reflect input {
  flex: 1; border: none; outline: none; background: transparent;
  font-size: var(--fs-14); color: var(--color-text-primary);
}
.fl-reflect input::placeholder { color: var(--color-text-muted); }
.fl-reflect-count { font-size: 11px; color: var(--color-text-muted); font-family: var(--font-mono); }

/* 温和未完成处理 */
.fl-gentle {
  border: 1px solid var(--color-border); border-radius: var(--r-md);
  overflow: hidden;
}
.fl-gentle-head {
  display: flex; gap: var(--sp-3); align-items: flex-start;
  padding: var(--sp-3) var(--sp-4);
  background: var(--color-primary-soft);
}
.fl-gentle-head strong { font-size: var(--fs-14); display: block; }
.fl-gentle-sub { font-size: var(--fs-12); color: var(--color-text-secondary); margin-top: 2px; }
.fl-gentle-task {
  padding: var(--sp-3) var(--sp-4);
  border-top: 1px solid var(--color-border);
}
.fl-gentle-name { font-size: var(--fs-14); font-weight: var(--fw-medium); margin-bottom: var(--sp-2); }
.fl-gentle-options { display: flex; gap: var(--sp-2); flex-wrap: wrap; }
.fl-gentle-btn {
  padding: 4px 12px; border-radius: var(--r-sm);
  border: 1px solid var(--color-border); background: transparent;
  font-size: var(--fs-12); cursor: pointer; color: var(--color-text-secondary);
  transition: all var(--dur-fast);
}
.fl-gentle-btn:hover { border-color: var(--color-primary); color: var(--color-primary); }
.fl-gentle-primary { background: var(--color-primary-soft); color: var(--color-primary-dark); border-color: color-mix(in srgb, var(--color-primary) 25%, transparent); }
.fl-gentle-quote {
  padding: var(--sp-3) var(--sp-4);
  background: var(--color-bg-subtle); font-size: var(--fs-12);
  color: var(--color-text-secondary); line-height: 1.6;
  border-top: 1px solid var(--color-border);
}

/* ---------- Footer ---------- */
.fl-sd-foot {
  display: flex; gap: var(--sp-2);
  padding: var(--sp-4) var(--sp-5);
  border-top: 1px solid var(--color-border);
  background: var(--color-bg-subtle);
}
.fl-sd-btn {
  flex: 1; padding: var(--sp-2) var(--sp-4); border-radius: var(--r-md);
  font-size: var(--fs-12); font-weight: var(--fw-medium); cursor: pointer; border: none;
}
.fl-sd-btn-primary { background: var(--color-primary); color: #fff; }
.fl-sd-btn-primary:hover { background: var(--color-primary-dark); }
.fl-sd-btn-ghost { background: transparent; color: var(--color-text-secondary); border: 1px solid var(--color-border); }

/* ---------- Transitions ---------- */
.fl-fade-enter-active, .fl-fade-leave-active { transition: opacity var(--dur-base) var(--ease-smooth); }
.fl-fade-enter-from, .fl-fade-leave-to { opacity: 0; }
</style>

<script setup lang="ts">
/**
 * MilestoneInfoPanel · 里程碑预计完成日 + 备注列表。
 *
 * 对齐 prototype/goals/milestones.html:546-561。
 * 右栏下半区:日期输入 + 剩余天数提示 + 备注 CRUD(3 条带日期笔记风格)。
 */

import { AlertTriangle, Plus, Trash2 } from "lucide-vue-next";
import { computed, ref, watch } from "vue";

import { useAIStore } from "@/stores/useAIStore";
import type { MilestoneRiskResult } from "@/stores/useAIStore";
import { useMilestoneSubtasks } from "@/composables/useMilestoneSubtasks";
import { useGoalStore } from "@/stores/useGoalStore";
import type { Milestone } from "@/types";

const props = defineProps<{
  milestone: Milestone;
}>();

const goals = useGoalStore();
const ai = useAIStore();
const { progressOf } = useMilestoneSubtasks();
const newNote = ref("");
const adding = ref(false);
const editingDate = ref<string>("");
const riskResult = ref<MilestoneRiskResult | null>(null);
const analyzing = ref(false);

watch(
  () => props.milestone.id,
  () => {
    editingDate.value = props.milestone.target_date ?? "";
    goals.loadNotes(props.milestone.id).catch((e) => console.error(e));
  },
  { immediate: true },
);

const remainingDays = computed<number | null>(() => {
  if (!props.milestone.target_date) return null;
  const target = new Date(props.milestone.target_date);
  target.setHours(0, 0, 0, 0);
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  return Math.round((target.getTime() - today.getTime()) / 86400000);
});

/** 是否满足风险预警触发条件：剩余 ≤5 天 且 完成率 < 50% */
const showRiskTrigger = computed(() => {
  const rd = remainingDays.value;
  if (rd === null) return false;
  const { done, total } = progressOf(props.milestone.id);
  const rate = total > 0 ? done / total : 0;
  return rd <= 5 && rate < 0.5;
});

const notes = computed(() => goals.notesByMilestone[props.milestone.id] ?? []);

async function onDateChange(e: Event) {
  const v = (e.target as HTMLInputElement).value;
  editingDate.value = v;
  await goals.setMilestoneTargetDate(props.milestone.id, v || null);
}

async function onAddNote() {
  const text = newNote.value.trim();
  if (!text) return;
  adding.value = true;
  try {
    await goals.addNote(props.milestone.id, text);
    newNote.value = "";
  } finally {
    adding.value = false;
  }
}

async function onDeleteNote(id: string) {
  await goals.removeNote(props.milestone.id, id);
}

async function onAnalyzeRisk() {
  if (!props.milestone.target_date) return;
  analyzing.value = true;
  riskResult.value = null;
  try {
    const { done, total } = progressOf(props.milestone.id);
    const rd = remainingDays.value ?? 0;
    riskResult.value = await ai.milestoneRisk(
      props.milestone.name,
      "", // goal_name 从父组件传入最佳，但当前 props 无此字段，留空 AI 仍可分析
      props.milestone.target_date,
      rd,
      done,
      total,
      props.milestone.id,
    );
  } catch (e) {
    console.error("[ai] milestone risk failed", e);
  } finally {
    analyzing.value = false;
  }
}

function formatDate(iso: string): string {
  const d = new Date(iso);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}
</script>

<template>
  <div class="fl-ms-info">
    <!-- 预计完成日 -->
    <section class="fl-ms-info-sec">
      <div class="fl-ms-info-head">预计完成</div>
      <div class="fl-ms-date-row">
        <input
          type="date"
          class="fl-ms-date-input"
          :value="editingDate"
          @change="onDateChange"
        />
        <span v-if="remainingDays !== null" class="fl-ms-date-hint">
          <template v-if="remainingDays > 0">剩余约 <strong>{{ remainingDays }}</strong> 天</template>
          <template v-else-if="remainingDays === 0">就在今天</template>
          <template v-else>已过期 {{ -remainingDays }} 天</template>
        </span>
        <span v-else class="fl-ms-date-hint is-muted">未设置日期</span>
      </div>

      <!-- 风险预警触发区 -->
      <div v-if="showRiskTrigger" class="fl-ms-risk-trigger">
        <button
          class="fl-ms-risk-btn"
          type="button"
          :disabled="analyzing"
          @click="onAnalyzeRisk"
        >
          <AlertTriangle :size="13" />
          {{ analyzing ? '分析中…' : '⚠️ AI 风险分析' }}
        </button>
      </div>

      <!-- 风险结果卡片 -->
      <div v-if="riskResult" class="fl-ms-risk-card" :data-level="riskResult.risk_level">
        <div class="fl-ms-risk-head">
          <span class="fl-ms-risk-badge">
            {{ riskResult.risk_level === 'high' ? '🔴 高风险' : riskResult.risk_level === 'medium' ? '🟡 中等风险' : '🟢 低风险' }}
          </span>
        </div>
        <p class="fl-ms-risk-summary">{{ riskResult.summary }}</p>
        <ul class="fl-ms-risk-actions">
          <li v-for="(action, i) in riskResult.actions" :key="i">{{ action }}</li>
        </ul>
      </div>
    </section>

    <!-- 备注 -->
    <section class="fl-ms-info-sec">
      <div class="fl-ms-info-head">
        <span>里程碑备注</span>
        <span class="fl-ms-info-count">{{ notes.length }}</span>
      </div>

      <div v-if="notes.length" class="fl-ms-notes">
        <div v-for="n in notes" :key="n.id" class="fl-ms-note">
          <span class="fl-ms-note-date">📝 {{ formatDate(n.created_at) }}</span>
          <span class="fl-ms-note-text">{{ n.text }}</span>
          <button class="fl-ms-note-del" title="删除" @click="onDeleteNote(n.id)">
            <Trash2 :size="12" />
          </button>
        </div>
      </div>
      <div v-else class="fl-ms-notes-empty">还没有备注 · 记下进展、决策、讨论</div>

      <form class="fl-ms-note-add" @submit.prevent="onAddNote">
        <textarea
          v-model="newNote"
          class="fl-ms-note-input"
          placeholder="新增备注(Cmd/Ctrl + Enter 提交)"
          rows="2"
          maxlength="500"
          @keydown.enter.meta="onAddNote"
          @keydown.enter.ctrl="onAddNote"
        />
        <button class="fl-ms-note-add-btn" type="submit" :disabled="!newNote.trim() || adding">
          <Plus :size="12" />
        </button>
      </form>
    </section>
  </div>
</template>

<style scoped>
/* 风险预警 */
.fl-ms-risk-trigger { margin-top: var(--sp-2); }
.fl-ms-risk-btn {
  display: inline-flex; align-items: center; gap: 6px;
  padding: 5px 10px;
  border-radius: var(--r-md);
  border: 1px solid color-mix(in srgb, var(--color-danger, #ef4444) 60%, transparent);
  background: color-mix(in srgb, var(--color-danger, #ef4444) 8%, transparent);
  color: var(--color-danger, #ef4444);
  font-size: 12px; cursor: pointer;
  transition: background var(--dur-fast);
}
.fl-ms-risk-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-danger, #ef4444) 15%, transparent);
}
.fl-ms-risk-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.fl-ms-risk-card {
  margin-top: var(--sp-3);
  padding: var(--sp-3) var(--sp-3);
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-subtle);
}
.fl-ms-risk-card[data-level="high"] {
  border-color: color-mix(in srgb, var(--color-danger, #ef4444) 40%, transparent);
  background: color-mix(in srgb, var(--color-danger, #ef4444) 5%, var(--color-bg-subtle));
}
.fl-ms-risk-card[data-level="medium"] {
  border-color: color-mix(in srgb, #f59e0b 40%, transparent);
  background: color-mix(in srgb, #f59e0b 5%, var(--color-bg-subtle));
}
.fl-ms-risk-head { margin-bottom: var(--sp-1); }
.fl-ms-risk-badge { font-size: 12px; font-weight: var(--fw-semibold); }
.fl-ms-risk-summary {
  font-size: 12px; color: var(--color-text-secondary);
  margin: 0 0 var(--sp-2);
  line-height: 1.5;
}
.fl-ms-risk-actions {
  margin: 0; padding-left: var(--sp-4);
  font-size: 12px; color: var(--color-text-secondary);
  display: flex; flex-direction: column; gap: 2px;
}

.fl-ms-info { display: flex; flex-direction: column; gap: var(--sp-5); }

.fl-ms-info-sec { display: flex; flex-direction: column; gap: var(--sp-2); }

.fl-ms-info-head {
  font-size: var(--fs-12); color: var(--color-text-muted);
  text-transform: uppercase; letter-spacing: 0.5px;
  font-weight: var(--fw-medium);
  display: flex; align-items: center; justify-content: space-between;
}
.fl-ms-info-count {
  font-size: 11px; color: var(--color-text-muted);
  font-family: var(--font-mono);
}

/* Date */
.fl-ms-date-row {
  display: flex; align-items: center; gap: var(--sp-3);
  padding: 6px 10px;
  background: var(--color-bg-subtle); border: 1px solid var(--color-border);
  border-radius: var(--r-md);
}
.fl-ms-date-input {
  flex: 0 0 auto;
  background: transparent; border: none; outline: none;
  font-family: var(--font-mono); font-size: 12px;
  color: var(--color-text-primary);
  color-scheme: dark light;
}
.fl-ms-date-hint {
  font-size: var(--fs-13, 13px);
  color: var(--color-text-secondary);
}
.fl-ms-date-hint.is-muted { color: var(--color-text-muted); }
.fl-ms-date-hint strong {
  color: var(--color-primary);
  font-family: var(--font-mono);
}

/* Notes */
.fl-ms-notes {
  display: flex; flex-direction: column; gap: var(--sp-2);
  padding: var(--sp-3);
  background: var(--color-bg-subtle); border-radius: var(--r-sm);
  line-height: 1.6;
  max-height: 220px; overflow-y: auto;
}
.fl-ms-note {
  display: grid;
  grid-template-columns: auto 1fr auto;
  gap: var(--sp-2);
  align-items: start;
  font-size: var(--fs-13, 13px);
  color: var(--color-text-secondary);
}
.fl-ms-note-date {
  font-weight: var(--fw-medium);
  color: var(--color-text-primary);
  white-space: nowrap;
}
.fl-ms-note-text { word-break: break-word; }
.fl-ms-note-del {
  background: none; border: none;
  color: var(--color-text-muted); cursor: pointer;
  padding: 2px; border-radius: var(--r-xs);
  opacity: 0; transition: opacity var(--dur-fast);
}
.fl-ms-note:hover .fl-ms-note-del { opacity: 1; }
.fl-ms-note-del:hover { color: var(--color-danger, #ef4444); }

.fl-ms-notes-empty {
  font-size: 11px; color: var(--color-text-muted);
  padding: var(--sp-3); text-align: center;
}

.fl-ms-note-add { display: flex; gap: var(--sp-2); align-items: flex-end; }
.fl-ms-note-input {
  flex: 1;
  padding: 6px 10px;
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-subtle);
  color: var(--color-text-primary);
  font-family: inherit; font-size: 12px;
  outline: none; resize: vertical;
}
.fl-ms-note-input:focus { border-color: var(--color-primary); }
.fl-ms-note-add-btn {
  padding: 6px 10px;
  border-radius: var(--r-md);
  border: none;
  background: var(--color-primary);
  color: var(--color-text-on-primary);
  cursor: pointer;
  display: grid; place-items: center;
}
.fl-ms-note-add-btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>

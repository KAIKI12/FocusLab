<script setup lang="ts">
/**
 * QuickAddModal · 快速添加任务弹窗 (⌘N)。
 * 对齐 prototype/screens/modals.html §6 Quick Add Task。
 */

import { Plus, X } from "lucide-vue-next";
import { nextTick, onMounted, onUnmounted, ref, watch } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import { useGoalStore } from "@/stores/useGoalStore";
import { useTaskStore } from "@/stores/useTaskStore";
import type { Milestone } from "@/types";

import { useUIStore } from "@/stores/useUIStore";

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: []; created: [taskId: string] }>();

const tasks = useTaskStore();
const goals = useGoalStore();
const ui = useUIStore();

const titleEl = ref<HTMLInputElement | null>(null);
const title = ref("");
const quadrant = ref("important_not_urgent");
const estimatedMinutes = ref<number | null>(null);
const dueDate = ref("");
const selectedGoalId = ref<string | null>(null);
const milestoneId = ref<string | null>(null);
const isBackground = ref(false);
const localMilestones = ref<Milestone[]>([]);

const quadrants = [
  { value: "important_urgent", label: "紧急重要", cls: "q1", emoji: "🔴" },
  { value: "important_not_urgent", label: "重要不紧急", cls: "q2", emoji: "🟡" },
  { value: "not_important_urgent", label: "紧急不重要", cls: "q3", emoji: "🟠" },
  { value: "not_important_not_urgent", label: "不紧急不重要", cls: "q4", emoji: "🟢" },
];

const timePresets = [15, 30, 60, 120];

// 截止日期快速选项
function setDueDays(days: number) {
  const d = new Date();
  d.setDate(d.getDate() + days);
  dueDate.value = d.toISOString().slice(0, 10);
}

function setDueNextMonday() {
  const d = new Date();
  d.setDate(d.getDate() + ((8 - d.getDay()) % 7 || 7));
  dueDate.value = d.toISOString().slice(0, 10);
}

function clearDue() { dueDate.value = ""; }

// 加载里程碑
watch(selectedGoalId, async (gid) => {
  if (!gid) { localMilestones.value = []; milestoneId.value = null; return; }
  try {
    localMilestones.value = await invokeCmd<Milestone[]>("list_milestones", { goalId: gid });
  } catch { localMilestones.value = []; }
});

// 打开时 auto-focus + 预填充
watch(() => props.visible, (v) => {
  if (v) {
    if (ui.quickNotePrefilledTitle) {
      title.value = ui.quickNotePrefilledTitle;
      ui.quickNotePrefilledTitle = "";
    }
    if (ui.quickNotePrefilledQuadrant) {
      quadrant.value = ui.quickNotePrefilledQuadrant;
      ui.quickNotePrefilledQuadrant = "";
    }
    nextTick(() => titleEl.value?.focus());
  }
  else resetForm();
});

function resetForm() {
  title.value = "";
  quadrant.value = "important_not_urgent";
  estimatedMinutes.value = null;
  dueDate.value = "";
  selectedGoalId.value = null;
  milestoneId.value = null;
  isBackground.value = false;
}

async function onSave(startFocus = false) {
  if (!title.value.trim()) return;
  try {
    const created = await tasks.create({ name: title.value.trim(), quadrant: quadrant.value });
    // 补充更新其他字段
    const updates: Record<string, unknown> = { id: created.id };
    if (estimatedMinutes.value) updates.estimatedMinutes = estimatedMinutes.value;
    if (dueDate.value) updates.dueDate = dueDate.value;
    if (milestoneId.value) updates.milestoneId = milestoneId.value;
    if (isBackground.value) updates.isBackground = true;
    if (Object.keys(updates).length > 1) {
      await tasks.update(updates as any);
    }
    emit("created", created.id);
    if (startFocus) emit("close");
    else emit("close");
  } catch (e) { console.error("[quickadd]", e); }
}

// 全局快捷键
function onKeyDown(e: KeyboardEvent) {
  if (!props.visible) return;
  if (e.key === "Escape") { emit("close"); e.preventDefault(); }
  if ((e.metaKey || e.ctrlKey) && e.key === "Enter") { onSave(true); e.preventDefault(); }
}

onMounted(() => document.addEventListener("keydown", onKeyDown));
onUnmounted(() => document.removeEventListener("keydown", onKeyDown));
</script>

<template>
  <Transition name="fl-fade">
    <div v-if="visible" class="fl-modal-mask" @click.self="emit('close')">
      <div class="fl-qa" role="dialog" aria-modal="true">
        <!-- Head -->
        <div class="fl-qa-head">
          <Plus :size="18" class="fl-qa-icon" />
          <input
            ref="titleEl"
            v-model="title"
            class="fl-qa-title"
            type="text"
            placeholder="要做什么?"
            maxlength="80"
            @keydown.enter.exact.prevent="onSave()"
          />
          <button class="fl-qa-close" @click="emit('close')"><X :size="14" /></button>
        </div>

        <!-- Body -->
        <div class="fl-qa-body">
          <!-- Chips -->
          <div class="fl-qa-chips">
            <span
              v-for="q in quadrants" :key="q.value"
              class="fl-qa-chip" :class="[q.cls, { 'is-selected': quadrant === q.value }]"
              @click="quadrant = q.value"
            >{{ q.emoji }} {{ q.label }}</span>
            <span
              v-for="t in timePresets" :key="t"
              class="fl-qa-chip" :class="{ 'is-selected': estimatedMinutes === t }"
              @click="estimatedMinutes = estimatedMinutes === t ? null : t"
            >{{ t >= 60 ? `${t / 60}h` : `${t}m` }}</span>
            <span class="fl-qa-chip" :class="{ 'is-selected': isBackground }" @click="isBackground = !isBackground">
              后台任务
            </span>
          </div>

          <!-- Detail rows -->
          <div class="fl-qa-row">
            <span class="fl-qa-label">关联目标</span>
            <select v-model="selectedGoalId" class="fl-qa-select">
              <option :value="null">无</option>
              <option v-for="g in goals.goals" :key="g.id" :value="g.id">🎯 {{ g.name }}</option>
            </select>
          </div>
          <div v-if="selectedGoalId && localMilestones.length" class="fl-qa-row">
            <span class="fl-qa-label">里程碑</span>
            <select v-model="milestoneId" class="fl-qa-select">
              <option :value="null">无</option>
              <option v-for="m in localMilestones" :key="m.id" :value="m.id">{{ m.name }}</option>
            </select>
          </div>
          <div class="fl-qa-row" style="align-items: flex-start">
            <span class="fl-qa-label">截止日期</span>
            <div class="fl-qa-due">
              <span v-if="dueDate" class="fl-qa-val">📅 {{ dueDate }}</span>
              <span v-else class="fl-qa-val fl-muted">未设置</span>
              <div class="fl-qa-due-chips">
                <span class="fl-qa-chip fl-qa-chip-sm" @click="setDueDays(0)">今天</span>
                <span class="fl-qa-chip fl-qa-chip-sm" @click="setDueDays(1)">明天</span>
                <span class="fl-qa-chip fl-qa-chip-sm" @click="setDueDays(3)">3天后</span>
                <span class="fl-qa-chip fl-qa-chip-sm" @click="setDueNextMonday">下周一</span>
                <label class="fl-qa-chip fl-qa-chip-sm">
                  📅 自定义
                  <input type="date" class="fl-sr-only" @change="dueDate = ($event.target as HTMLInputElement).value" />
                </label>
                <span class="fl-qa-chip fl-qa-chip-sm fl-muted" @click="clearDue">不设</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="fl-qa-foot">
          <span class="fl-qa-hint">
            <span class="fl-kbd">⌘</span><span class="fl-kbd">↵</span> 保存并开始专注
          </span>
          <div class="fl-qa-btns">
            <button class="fl-btn fl-btn-ghost" @click="emit('close')">取消</button>
            <button class="fl-btn fl-btn-primary" :disabled="!title.trim()" @click="onSave()">保存</button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fl-modal-mask {
  position: fixed; inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 32%, transparent);
  display: grid; place-items: center;
  z-index: var(--z-modal); padding: var(--sp-4);
}

.fl-qa {
  width: min(520px, 100%);
  max-height: calc(100vh - 32px);
  overflow-x: hidden;
  overflow-y: auto;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-modal);
}

/* Head */
.fl-qa-head {
  display: flex; align-items: center; gap: var(--sp-2);
  padding: var(--sp-3) var(--sp-4);
  border-bottom: 1px solid var(--color-border);
}
.fl-qa-icon { color: var(--color-primary); flex-shrink: 0; }
.fl-qa-title {
  flex: 1; border: none; outline: none; background: transparent;
  font-size: var(--fs-16); font-weight: var(--fw-medium);
  color: var(--color-text-primary);
}
.fl-qa-title::placeholder { color: var(--color-text-muted); }
.fl-qa-close {
  background: none; border: none; color: var(--color-text-muted);
  cursor: pointer; padding: var(--sp-1); border-radius: var(--r-sm);
}
.fl-qa-close:hover { color: var(--color-text-primary); }

/* Body */
.fl-qa-body { padding: var(--sp-3) var(--sp-4); }

/* Chips */
.fl-qa-chips {
  display: flex; flex-wrap: wrap; gap: 6px;
  margin-bottom: var(--sp-3);
}
.fl-qa-chip {
  display: inline-flex; align-items: center; gap: 4px;
  padding: 6px 10px; background: var(--color-bg-hover);
  border-radius: var(--r-sm); font-size: var(--fs-12);
  cursor: pointer; color: var(--color-text-secondary);
  transition: all var(--dur-fast);
}
.fl-qa-chip:hover { background: var(--color-bg-subtle); }
.fl-qa-chip.is-selected { background: var(--color-primary); color: var(--color-text-on-primary, #fff); }
.fl-qa-chip-sm { padding: 4px 8px; font-size: 11px; }

/* Detail rows */
.fl-qa-row {
  display: flex; align-items: center; gap: var(--sp-3);
  padding: 6px 0; font-size: var(--fs-12);
}
.fl-qa-label {
  color: var(--color-text-muted); width: 72px; flex-shrink: 0;
}
.fl-qa-val { color: var(--color-text-primary); font-weight: var(--fw-medium); }
.fl-qa-select {
  flex: 1; padding: 4px 8px; border-radius: var(--r-sm);
  border: 1px solid var(--color-border); background: var(--color-bg-subtle);
  color: var(--color-text-primary); font-size: var(--fs-12); outline: none;
}
.fl-qa-select:focus { border-color: var(--color-primary); }

.fl-qa-due { display: flex; flex-direction: column; gap: 6px; flex: 1; }
.fl-qa-due-chips { display: flex; flex-wrap: wrap; gap: 4px; }

.fl-muted { color: var(--color-text-muted) !important; }

/* Footer */
.fl-qa-foot {
  display: flex; justify-content: space-between; align-items: center;
  padding: var(--sp-3) var(--sp-4);
  background: var(--color-bg-subtle);
  border-top: 1px solid var(--color-border);
}
.fl-qa-hint { font-size: 11px; color: var(--color-text-muted); display: flex; gap: 4px; align-items: center; }
.fl-kbd {
  display: inline-flex; align-items: center; justify-content: center;
  min-width: 18px; height: 18px; padding: 0 4px;
  background: var(--color-bg-elevated); border: 1px solid var(--color-border);
  border-radius: 3px; font-size: 10px; font-family: var(--font-mono, monospace);
}
.fl-qa-btns { display: flex; gap: 6px; }

.fl-btn {
  padding: 6px 14px; border-radius: var(--r-md);
  font-size: var(--fs-12); font-weight: var(--fw-medium);
  border: 1px solid transparent; cursor: pointer;
  transition: background var(--dur-fast);
}
.fl-btn-ghost { background: transparent; color: var(--color-text-secondary); border-color: var(--color-border); }
.fl-btn-ghost:hover { background: var(--color-bg-hover); }
.fl-btn-primary { background: var(--color-primary); color: var(--color-text-on-primary, #fff); }
.fl-btn-primary:hover:not(:disabled) { opacity: 0.9; }
.fl-btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

.fl-sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0,0,0,0); }

.fl-fade-enter-active, .fl-fade-leave-active { transition: opacity var(--dur-base) var(--ease-smooth); }
.fl-fade-enter-from, .fl-fade-leave-to { opacity: 0; }
</style>

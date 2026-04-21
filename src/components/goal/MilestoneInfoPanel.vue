<script setup lang="ts">
/**
 * MilestoneInfoPanel · 里程碑预计完成日 + 备注列表。
 *
 * 对齐 prototype/goals/milestones.html:546-561。
 * 右栏下半区:日期输入 + 剩余天数提示 + 备注 CRUD(3 条带日期笔记风格)。
 */

import { Plus, Trash2 } from "lucide-vue-next";
import { computed, ref, watch } from "vue";

import { useGoalStore } from "@/stores/useGoalStore";
import type { Milestone } from "@/types";

const props = defineProps<{
  milestone: Milestone;
}>();

const goals = useGoalStore();
const newNote = ref("");
const adding = ref(false);
const editingDate = ref<string>("");

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

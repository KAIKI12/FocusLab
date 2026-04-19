<script setup lang="ts">
/**
 * MilestoneTimeline · 里程碑时间线视图。
 * 展示选中目标下的所有里程碑 — 进度条 + 编辑 + 脉冲动画。
 */

import { Check, Circle, Pencil, Plus, X } from "lucide-vue-next";
import { computed, ref } from "vue";

import { useGoalStore } from "@/stores/useGoalStore";
import type { Milestone } from "@/types";

const goals = useGoalStore();
const newMsName = ref("");
const editingId = ref<string | null>(null);
const editName = ref("");
const editDesc = ref("");

const completedCount = computed(
  () => goals.milestones.filter((m) => m.status === "completed").length,
);

const progressPct = computed(() => {
  if (!goals.milestones.length) return 0;
  return Math.round((completedCount.value / goals.milestones.length) * 100);
});

// 第一个未完成的里程碑 = 当前活跃
const activeId = computed(() => {
  const pending = goals.milestones.find((m) => m.status !== "completed");
  return pending?.id ?? null;
});

const selectedGoal = computed(
  () => goals.goals.find((g) => g.id === goals.selectedGoalId),
);

async function onAddMilestone() {
  const name = newMsName.value.trim();
  if (!name || !goals.selectedGoalId) return;
  await goals.createMilestone({ goalId: goals.selectedGoalId, name });
  newMsName.value = "";
}

async function onComplete(id: string) {
  await goals.completeMilestone(id);
}

function startEdit(m: Milestone) {
  editingId.value = m.id;
  editName.value = m.name;
  editDesc.value = m.description ?? "";
}

async function saveEdit() {
  if (!editingId.value || !editName.value.trim()) return;
  try {
    await goals.updateMilestone({
      id: editingId.value,
      name: editName.value.trim(),
      description: editDesc.value || undefined,
    });
  } catch (e) { console.error(e); }
  editingId.value = null;
}

function cancelEdit() { editingId.value = null; }
</script>

<template>
  <div v-if="selectedGoal" class="fl-timeline">
    <header class="fl-tl-head">
      <h2>{{ selectedGoal.name }}</h2>
      <span class="fl-tl-progress">
        里程碑 {{ completedCount }} / {{ goals.milestones.length }}
      </span>
    </header>

    <!-- 进度条 -->
    <div v-if="goals.milestones.length" class="fl-tl-bar">
      <div class="fl-tl-bar-fill" :style="{ width: progressPct + '%' }" />
      <span class="fl-tl-bar-label">{{ progressPct }}%</span>
    </div>

    <p v-if="selectedGoal.description" class="fl-tl-desc">
      {{ selectedGoal.description }}
    </p>

    <!-- 里程碑列表 -->
    <div class="fl-tl-list">
      <div
        v-for="m in goals.milestones"
        :key="m.id"
        class="fl-ms-item"
        :class="{ 'is-done': m.status === 'completed', 'is-active': m.id === activeId }"
      >
        <button
          class="fl-ms-dot"
          :class="{ 'is-checked': m.status === 'completed', 'is-pulse': m.id === activeId }"
          type="button"
          :aria-label="m.status === 'completed' ? '已完成' : '标记完成'"
          @click="onComplete(m.id)"
        >
          <Check v-if="m.status === 'completed'" :size="10" />
          <Circle v-else :size="8" />
        </button>

        <!-- 编辑模式 -->
        <div v-if="editingId === m.id" class="fl-ms-edit">
          <input v-model="editName" class="fl-ms-input" type="text" maxlength="60" @keydown.enter="saveEdit" />
          <input v-model="editDesc" class="fl-ms-input fl-ms-input-sm" type="text" placeholder="描述(可选)" maxlength="200" />
          <div class="fl-ms-edit-btns">
            <button class="fl-ms-save" @click="saveEdit">保存</button>
            <button class="fl-ms-cancel" @click="cancelEdit"><X :size="12" /></button>
          </div>
        </div>

        <!-- 显示模式 -->
        <div v-else class="fl-ms-body" @dblclick="startEdit(m)">
          <span class="fl-ms-name">{{ m.name }}</span>
          <span v-if="m.description" class="fl-ms-desc">{{ m.description }}</span>
        </div>
        <button v-if="editingId !== m.id" class="fl-ms-edit-btn" title="编辑" @click="startEdit(m)">
          <Pencil :size="10" />
        </button>
      </div>
    </div>

    <!-- 添加里程碑 -->
    <form class="fl-ms-add" @submit.prevent="onAddMilestone">
      <input
        v-model="newMsName"
        class="fl-ms-input"
        type="text"
        placeholder="添加里程碑…"
        maxlength="60"
      />
      <button class="fl-ms-add-btn" type="submit" :disabled="!newMsName.trim()">
        <Plus :size="12" /> 添加
      </button>
    </form>
  </div>

  <div v-else class="fl-tl-empty">
    选择左侧的目标查看里程碑
  </div>
</template>

<style scoped>
.fl-timeline {
  display: flex;
  flex-direction: column;
  gap: var(--sp-4);
}

.fl-tl-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
}
.fl-tl-head h2 {
  font-size: var(--fs-16);
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
  margin: 0;
}
.fl-tl-progress {
  font-size: var(--fs-12);
  color: var(--color-text-muted);
}

/* 进度条 */
.fl-tl-bar {
  height: 6px; border-radius: 3px;
  background: var(--color-bg-hover); position: relative;
  overflow: hidden;
}
.fl-tl-bar-fill {
  height: 100%; border-radius: 3px;
  background: var(--color-primary);
  transition: width 0.4s var(--ease-smooth);
}
.fl-tl-bar-label {
  position: absolute; right: 0; top: -16px;
  font-size: 10px; color: var(--color-text-muted);
}

.fl-tl-desc {
  margin: 0;
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
}

.fl-tl-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-left: var(--sp-2);
  border-left: 2px solid var(--color-border);
}

.fl-ms-item {
  display: flex;
  align-items: flex-start;
  gap: var(--sp-3);
  padding: var(--sp-2) var(--sp-3);
  margin-left: calc(-1 * var(--sp-2) - 1px);
  border-radius: var(--r-sm);
}
.fl-ms-item:hover {
  background: var(--color-bg-hover);
}
.fl-ms-item.is-done .fl-ms-name {
  color: var(--color-text-muted);
  text-decoration: line-through;
}
.fl-ms-item.is-active { background: color-mix(in srgb, var(--color-primary) 5%, transparent); }

.fl-ms-dot {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
  border-radius: 50%;
  border: 1.5px solid var(--color-border-strong);
  background: var(--color-bg-elevated);
  color: transparent;
  cursor: pointer;
  display: grid;
  place-items: center;
  transition: all var(--dur-fast) var(--ease-smooth);
  margin-top: 2px;
}
.fl-ms-dot:hover,
.fl-ms-dot.is-checked {
  border-color: var(--color-success);
  background: var(--color-success);
  color: #fff;
}
.fl-ms-dot.is-pulse {
  border-color: var(--color-primary);
  animation: ms-pulse 2s ease-in-out infinite;
}
@keyframes ms-pulse {
  0%, 100% { box-shadow: 0 0 0 0 color-mix(in srgb, var(--color-primary) 40%, transparent); }
  50% { box-shadow: 0 0 0 6px color-mix(in srgb, var(--color-primary) 0%, transparent); }
}

.fl-ms-body {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
}
.fl-ms-name {
  font-size: var(--fs-14);
  color: var(--color-text-primary);
}
.fl-ms-desc {
  font-size: 11px;
  color: var(--color-text-muted);
}

.fl-ms-add {
  display: flex;
  gap: var(--sp-2);
  margin-left: var(--sp-2);
}
.fl-ms-input {
  flex: 1;
  padding: var(--sp-2) var(--sp-3);
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-subtle);
  color: var(--color-text-primary);
  font-size: var(--fs-12);
  outline: none;
}
.fl-ms-input:focus {
  border-color: var(--color-primary);
}
.fl-ms-add-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: var(--sp-2) var(--sp-3);
  border-radius: var(--r-md);
  border: none;
  background: var(--color-primary);
  color: var(--color-text-on-primary);
  font-size: 11px;
  cursor: pointer;
}
.fl-ms-add-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.fl-tl-empty {
  text-align: center;
  padding: var(--sp-10);
  color: var(--color-text-muted);
  font-size: var(--fs-14);
}

/* 编辑按钮 */
.fl-ms-edit-btn {
  opacity: 0; background: none; border: none;
  color: var(--color-text-muted); cursor: pointer;
  padding: 2px; border-radius: var(--r-sm); flex-shrink: 0;
  transition: opacity var(--dur-fast);
}
.fl-ms-item:hover .fl-ms-edit-btn { opacity: 1; }
.fl-ms-edit-btn:hover { color: var(--color-primary); }

/* 编辑表单 */
.fl-ms-edit { display: flex; flex-direction: column; gap: 4px; flex: 1; }
.fl-ms-input-sm { font-size: 11px !important; }
.fl-ms-edit-btns { display: flex; gap: 4px; }
.fl-ms-save {
  padding: 2px 10px; border-radius: var(--r-sm);
  background: var(--color-primary); color: #fff;
  border: none; font-size: 11px; cursor: pointer;
}
.fl-ms-cancel {
  background: none; border: none; color: var(--color-text-muted);
  cursor: pointer; padding: 2px;
}
.fl-ms-cancel:hover { color: var(--color-text-primary); }
</style>

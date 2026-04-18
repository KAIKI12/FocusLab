<script setup lang="ts">
/**
 * MilestoneTimeline · 里程碑时间线视图。
 * 展示选中目标下的所有里程碑(N/M 完成状态 + 列表)。
 */

import { Check, Circle, Plus } from "lucide-vue-next";
import { computed, ref } from "vue";

import { useGoalStore } from "@/stores/useGoalStore";

const goals = useGoalStore();
const newMsName = ref("");

const completedCount = computed(
  () => goals.milestones.filter((m) => m.status === "completed").length,
);

async function onAddMilestone() {
  const name = newMsName.value.trim();
  if (!name || !goals.selectedGoalId) return;
  await goals.createMilestone({
    goalId: goals.selectedGoalId,
    name,
  });
  newMsName.value = "";
}

async function onComplete(id: string) {
  await goals.completeMilestone(id);
}

const selectedGoal = computed(
  () => goals.goals.find((g) => g.id === goals.selectedGoalId),
);
</script>

<template>
  <div v-if="selectedGoal" class="fl-timeline">
    <header class="fl-tl-head">
      <h2>{{ selectedGoal.name }}</h2>
      <span class="fl-tl-progress">
        里程碑 {{ completedCount }} / {{ goals.milestones.length }}
      </span>
    </header>

    <p v-if="selectedGoal.description" class="fl-tl-desc">
      {{ selectedGoal.description }}
    </p>

    <!-- 里程碑列表 -->
    <div class="fl-tl-list">
      <div
        v-for="m in goals.milestones"
        :key="m.id"
        class="fl-ms-item"
        :class="{ 'is-done': m.status === 'completed' }"
      >
        <button
          class="fl-ms-dot"
          :class="{ 'is-checked': m.status === 'completed' }"
          type="button"
          :aria-label="m.status === 'completed' ? '已完成' : '标记完成'"
          @click="onComplete(m.id)"
        >
          <Check v-if="m.status === 'completed'" :size="10" />
          <Circle v-else :size="8" />
        </button>
        <div class="fl-ms-body">
          <span class="fl-ms-name">{{ m.name }}</span>
          <span v-if="m.description" class="fl-ms-desc">{{ m.description }}</span>
        </div>
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
</style>

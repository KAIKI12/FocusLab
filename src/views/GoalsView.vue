<script setup lang="ts">
/**
 * GoalsView · 长线目标管理页面。
 *
 * 左栏: 目标列表 + 新建目标表单
 * 右栏: 选中目标的里程碑时间线
 */

import { Plus } from "lucide-vue-next";
import { onMounted, ref } from "vue";

import GoalCard from "@/components/goal/GoalCard.vue";
import MilestoneTimeline from "@/components/goal/MilestoneTimeline.vue";
import { useGoalStore } from "@/stores/useGoalStore";

const goals = useGoalStore();
const newGoalName = ref("");

onMounted(async () => {
  await goals.loadGoals();
  // 自动选中第一个
  if (goals.goals.length > 0 && !goals.selectedGoalId) {
    await goals.selectGoal(goals.goals[0].id);
  }
});

async function onAddGoal() {
  const name = newGoalName.value.trim();
  if (!name) return;
  const created = await goals.createGoal({ name });
  newGoalName.value = "";
  await goals.selectGoal(created.id);
}
</script>

<template>
  <section class="fl-goals">
    <header class="fl-page-head">
      <h1>长线目标</h1>
      <p class="fl-page-sub">追踪长期目标与里程碑进展</p>
    </header>

    <div class="fl-goals-layout">
      <!-- 左栏:目标列表 -->
      <div class="fl-goals-left">
        <form class="fl-goal-form" @submit.prevent="onAddGoal">
          <input
            v-model="newGoalName"
            class="fl-goal-input"
            type="text"
            placeholder="新建目标…"
            maxlength="60"
          />
          <button class="fl-goal-add" type="submit" :disabled="!newGoalName.trim()">
            <Plus :size="14" />
          </button>
        </form>

        <div v-if="goals.loading" class="fl-goals-empty">载入中…</div>
        <div v-else-if="!goals.goals.length" class="fl-goals-empty">
          还没有长线目标 · 写下你的第一个 ↑
        </div>
        <div v-else class="fl-goals-list">
          <GoalCard
            v-for="g in goals.goals"
            :key="g.id"
            :goal="g"
            :selected="goals.selectedGoalId === g.id"
            @select="goals.selectGoal(g.id)"
            @archive="goals.archiveGoal(g.id)"
          />
        </div>
      </div>

      <!-- 右栏:里程碑时间线 -->
      <div class="fl-goals-right">
        <MilestoneTimeline />
      </div>
    </div>
  </section>
</template>

<style scoped>
.fl-goals {
  max-width: 960px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--sp-5);
}

.fl-page-head h1 {
  font-size: var(--fs-24);
  font-weight: var(--fw-semibold);
  margin: 0;
  color: var(--color-text-primary);
}
.fl-page-sub {
  margin: var(--sp-1) 0 0;
  color: var(--color-text-secondary);
  font-size: var(--fs-12);
}

.fl-goals-layout {
  display: grid;
  grid-template-columns: 280px 1fr;
  gap: var(--sp-5);
  min-height: 400px;
}

.fl-goals-left {
  display: flex;
  flex-direction: column;
  gap: var(--sp-3);
}

.fl-goal-form {
  display: flex;
  gap: var(--sp-2);
}
.fl-goal-input {
  flex: 1;
  padding: var(--sp-2) var(--sp-3);
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
  font-size: var(--fs-13, var(--fs-14));
  outline: none;
}
.fl-goal-input:focus {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}
.fl-goal-add {
  padding: var(--sp-2);
  border-radius: var(--r-md);
  border: none;
  background: var(--color-primary);
  color: var(--color-text-on-primary);
  cursor: pointer;
  display: grid;
  place-items: center;
}
.fl-goal-add:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.fl-goals-list {
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}

.fl-goals-right {
  padding: var(--sp-4);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
}

.fl-goals-empty {
  text-align: center;
  padding: var(--sp-6);
  color: var(--color-text-muted);
  font-size: var(--fs-12);
}
</style>

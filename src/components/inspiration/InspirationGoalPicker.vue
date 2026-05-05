<script setup lang="ts">
/**
 * InspirationGoalPicker · 给灵感卡片"挂到目标"的内联面板。
 * - U1: native select 改为 fuzzy search input + 列表(目标多时也好用)
 * - "AI 推荐归属"按钮触发后端推荐
 * - 显示推荐结果 + 接受推荐
 *
 * 父组件负责调用 store(assign / suggest / acceptSuggestion),
 * 子组件只关注 UI 与交互。
 */

import { Check, Sparkles, X } from "lucide-vue-next";
import { computed, ref } from "vue";

import type { InspirationItem } from "@/stores/useInspirationStore";

interface GoalLite {
  id: string;
  name: string;
}
interface GoalSuggestion {
  goalId: string | null;
  reason: string;
}

const props = defineProps<{
  item: InspirationItem;
  goals: GoalLite[];
  goalNameById: Record<string, string>;
  suggestion?: GoalSuggestion | null;
  loading?: boolean;
}>();

const emit = defineEmits<{
  (e: "assign", goalId: string | null): void;
  (e: "suggest"): void;
  (e: "accept-suggestion"): void;
}>();

const query = ref("");

const filteredGoals = computed(() => {
  const q = query.value.trim().toLowerCase();
  if (!q) return props.goals.slice(0, 8);
  return props.goals
    .filter((g) => g.name.toLowerCase().includes(q))
    .slice(0, 8);
});

const currentGoalName = computed(() =>
  props.item.goalId ? props.goalNameById[props.item.goalId] ?? "(未知目标)" : null,
);
</script>

<template>
  <div class="fl-goal-picker" @click.stop>
    <div class="fl-goal-picker-head">
      <label class="fl-goal-picker-label">挂到目标</label>
      <span v-if="currentGoalName" class="fl-goal-picker-current">
        当前: {{ currentGoalName }}
        <button
          class="fl-goal-picker-clear"
          type="button"
          title="取消挂载"
          @click="emit('assign', null)"
        >
          <X :size="11" />
        </button>
      </span>
    </div>

    <input
      v-model="query"
      type="search"
      class="fl-goal-picker-search"
      placeholder="搜索目标名…"
      spellcheck="false"
    />

    <div class="fl-goal-picker-list">
      <button
        v-for="goal in filteredGoals"
        :key="goal.id"
        class="fl-goal-picker-item"
        :class="{ 'is-current': goal.id === item.goalId }"
        type="button"
        @click="emit('assign', goal.id)"
      >
        <Check v-if="goal.id === item.goalId" :size="12" class="fl-goal-picker-tick" />
        <span class="fl-goal-picker-name">{{ goal.name }}</span>
      </button>
      <div v-if="filteredGoals.length === 0" class="fl-goal-picker-empty">
        {{ goals.length === 0 ? "暂无目标,请先去目标页创建" : "没找到匹配的目标" }}
      </div>
    </div>

    <button
      class="fl-goal-picker-suggest-btn"
      type="button"
      :disabled="loading || !goals.length"
      @click="emit('suggest')"
    >
      <Sparkles :size="11" />
      {{ loading ? "推荐中…" : "AI 推荐归属" }}
    </button>

    <div v-if="suggestion" class="fl-goal-picker-result">
      <template v-if="suggestion.goalId">
        <div class="fl-goal-picker-result-head">
          <Sparkles :size="11" />
          <span>
            建议挂到: <strong>{{ goalNameById[suggestion.goalId] || "(未知目标)" }}</strong>
          </span>
        </div>
        <p class="fl-goal-picker-result-reason">{{ suggestion.reason }}</p>
        <button
          class="fl-goal-picker-result-apply"
          type="button"
          @click="emit('accept-suggestion')"
        >
          接受推荐
        </button>
      </template>
      <template v-else>
        <div class="fl-goal-picker-result-head is-none">
          <Sparkles :size="11" />
          <span>AI 认为没有合适目标</span>
        </div>
        <p class="fl-goal-picker-result-reason">{{ suggestion.reason }}</p>
      </template>
    </div>
  </div>
</template>

<style scoped>
.fl-goal-picker {
  margin-top: var(--sp-2);
  padding: var(--sp-2);
  border-radius: var(--r-sm);
  background: var(--color-bg-subtle);
  border: 1px solid var(--color-divider);
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}
.fl-goal-picker-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-2);
}
.fl-goal-picker-label {
  font-size: 11px;
  color: var(--color-text-secondary);
}
.fl-goal-picker-current {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 6px 2px 8px;
  font-size: 11px;
  border-radius: var(--r-pill);
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
  font-weight: 600;
}
.fl-goal-picker-clear {
  border: none;
  background: transparent;
  padding: 0;
  display: inline-grid;
  place-items: center;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  color: inherit;
  cursor: pointer;
  opacity: 0.6;
}
.fl-goal-picker-clear:hover {
  opacity: 1;
  background: rgba(0, 0, 0, 0.08);
}
.fl-goal-picker-search {
  border: 1px solid var(--color-border);
  border-radius: var(--r-sm);
  padding: 6px 10px;
  font-size: 12px;
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
}
.fl-goal-picker-search:focus {
  outline: none;
  border-color: var(--color-primary);
}
.fl-goal-picker-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 200px;
  overflow-y: auto;
}
.fl-goal-picker-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  font-size: 12px;
  text-align: left;
  border: 1px solid var(--color-border);
  border-radius: var(--r-sm);
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
  cursor: pointer;
  transition: all 100ms;
}
.fl-goal-picker-item:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}
.fl-goal-picker-item.is-current {
  background: color-mix(in srgb, var(--color-primary) 10%, var(--color-bg-elevated));
  border-color: color-mix(in srgb, var(--color-primary) 30%, var(--color-border));
  color: var(--color-primary-dark);
  font-weight: 600;
}
.fl-goal-picker-tick {
  flex-shrink: 0;
  color: var(--color-primary);
}
.fl-goal-picker-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fl-goal-picker-empty {
  padding: 8px 10px;
  font-size: 11px;
  color: var(--color-text-muted);
  text-align: center;
}
.fl-goal-picker-suggest-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 5px 10px;
  font-size: 11px;
  font-weight: 600;
  border: 1px dashed color-mix(in srgb, var(--color-primary) 50%, transparent);
  border-radius: var(--r-sm);
  background: color-mix(in srgb, var(--color-primary) 6%, transparent);
  color: var(--color-primary);
  cursor: pointer;
  align-self: flex-start;
  transition: all 120ms;
}
.fl-goal-picker-suggest-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--color-primary) 14%, transparent);
}
.fl-goal-picker-suggest-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.fl-goal-picker-result {
  margin-top: 4px;
  padding: 8px 10px;
  border-radius: var(--r-sm);
  background: color-mix(in srgb, var(--color-primary) 5%, transparent);
  border-left: 2px solid var(--color-primary);
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.fl-goal-picker-result-head {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11.5px;
  color: var(--color-text-primary);
}
.fl-goal-picker-result-head.is-none {
  color: var(--color-text-secondary);
}
.fl-goal-picker-result-reason {
  margin: 0;
  font-size: 11px;
  color: var(--color-text-secondary);
  line-height: 1.5;
}
.fl-goal-picker-result-apply {
  align-self: flex-start;
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 600;
  border: none;
  border-radius: var(--r-sm);
  background: var(--color-primary);
  color: white;
  cursor: pointer;
}
</style>

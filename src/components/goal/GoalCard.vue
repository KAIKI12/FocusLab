<script setup lang="ts">
/**
 * GoalCard · 目标卡片(左栏列表中的一项)。
 */

import { Archive, Target } from "lucide-vue-next";
import { computed } from "vue";

import type { Goal } from "@/types";

const props = defineProps<{
  goal: Goal;
  selected: boolean;
}>();

const emit = defineEmits<{
  select: [];
  archive: [];
}>();

/** 已投入天数(从创建到现在) */
const daysSinceCreated = computed(() => {
  const created = new Date(props.goal.created_at).getTime();
  const now = Date.now();
  return Math.max(1, Math.floor((now - created) / 86400000));
});
</script>

<template>
  <div
    class="fl-goal-card"
    :class="{ 'is-selected': selected }"
    role="button"
    tabindex="0"
    @click="emit('select')"
    @keydown.enter="emit('select')"
  >
    <div class="fl-gc-main">
      <Target :size="14" class="fl-gc-icon" />
      <div class="fl-gc-info">
        <span class="fl-gc-name">{{ goal.name }}</span>
        <span class="fl-gc-meta">已投入 {{ daysSinceCreated }} 天</span>
      </div>
    </div>
    <button
      class="fl-gc-archive"
      type="button"
      title="归档"
      @click.stop="emit('archive')"
    >
      <Archive :size="12" />
    </button>
  </div>
</template>

<style scoped>
.fl-goal-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sp-3) var(--sp-4);
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-elevated);
  cursor: pointer;
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-goal-card:hover {
  border-color: var(--color-primary);
}
.fl-goal-card.is-selected {
  border-color: var(--color-primary);
  background: color-mix(in srgb, var(--color-primary) 6%, var(--color-bg-elevated));
}

.fl-gc-main {
  display: flex;
  align-items: center;
  gap: var(--sp-3);
}
.fl-gc-icon {
  color: var(--color-primary);
  flex-shrink: 0;
}
.fl-gc-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.fl-gc-name {
  font-size: var(--fs-14);
  font-weight: var(--fw-medium);
  color: var(--color-text-primary);
}
.fl-gc-meta {
  font-size: 11px;
  color: var(--color-text-muted);
}

.fl-gc-archive {
  opacity: 0;
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: var(--sp-1);
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-goal-card:hover .fl-gc-archive {
  opacity: 1;
}
.fl-gc-archive:hover {
  color: var(--color-q1);
}
</style>

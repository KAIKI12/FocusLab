<script setup lang="ts">
/**
 * TodayView · Week 1a 最小 CRUD 验证页。
 * - 输入任务名 → 添加
 * - 列表展示 pending / in_progress 任务
 * - 点 ✓ 把任务标为 completed(从列表移除)
 */

import { Check, Plus } from "lucide-vue-next";
import { onMounted, ref } from "vue";

import { useTaskStore } from "@/stores/useTaskStore";

const store = useTaskStore();
const name = ref("");

onMounted(() => store.load());

async function onAdd() {
  const trimmed = name.value.trim();
  if (!trimmed) return;
  await store.create({ name: trimmed });
  name.value = "";
}
</script>

<template>
  <section class="fl-today">
    <header class="fl-page-head">
      <h1>今日</h1>
      <p class="fl-page-sub">Week 1a 验证页 · 跑通前后端 CRUD + SQLite 持久化</p>
    </header>

    <form class="fl-task-form" @submit.prevent="onAdd">
      <input
        v-model="name"
        class="fl-input"
        type="text"
        placeholder="添加今天要做的一件事…"
        maxlength="80"
      />
      <button class="fl-btn" type="submit" :disabled="!name.trim()">
        <Plus :size="16" />
        添加
      </button>
    </form>

    <div v-if="store.loading" class="fl-empty">载入中…</div>
    <ul v-else-if="store.tasks.length" class="fl-task-list">
      <li v-for="t in store.tasks" :key="t.id" class="fl-task-item">
        <button
          class="fl-task-check"
          type="button"
          :aria-label="`完成「${t.name}」`"
          @click="store.complete(t.id)"
        >
          <Check :size="14" />
        </button>
        <span class="fl-task-name">{{ t.name }}</span>
        <span class="fl-task-quadrant">{{ t.quadrant }}</span>
      </li>
    </ul>
    <div v-else class="fl-empty">还没有任务 · 写下第一件 ↑</div>
  </section>
</template>

<style scoped>
.fl-today {
  max-width: 720px;
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

.fl-task-form {
  display: flex;
  gap: var(--sp-2);
}

.fl-input {
  flex: 1;
  padding: var(--sp-3) var(--sp-4);
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
  font-size: var(--fs-14);
  outline: none;
  transition:
    border-color var(--dur-fast) var(--ease-smooth),
    box-shadow var(--dur-fast) var(--ease-smooth);
}

.fl-input::placeholder {
  color: var(--color-text-muted);
}

.fl-input:focus {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.fl-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--sp-1);
  padding: 0 var(--sp-5);
  border-radius: var(--r-md);
  background: var(--color-primary);
  color: var(--color-text-on-primary);
  border: none;
  font-size: var(--fs-14);
  font-weight: var(--fw-medium);
  cursor: pointer;
  transition: background var(--dur-fast) var(--ease-smooth);
}

.fl-btn:hover:not(:disabled) {
  background: var(--color-primary-dark);
}

.fl-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.fl-task-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}

.fl-task-item {
  display: flex;
  align-items: center;
  gap: var(--sp-3);
  padding: var(--sp-3) var(--sp-4);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  box-shadow: var(--shadow-card);
}

.fl-task-check {
  width: 22px;
  height: 22px;
  flex: 0 0 22px;
  border-radius: var(--r-pill);
  border: 1.5px solid var(--color-border-strong);
  background: transparent;
  color: transparent;
  cursor: pointer;
  display: grid;
  place-items: center;
  transition:
    border-color var(--dur-fast) var(--ease-smooth),
    background var(--dur-fast) var(--ease-smooth),
    color var(--dur-fast) var(--ease-smooth);
}

.fl-task-check:hover {
  border-color: var(--color-success);
  background: var(--color-success-soft);
  color: var(--color-success-text);
}

.fl-task-name {
  flex: 1;
  color: var(--color-text-primary);
}

.fl-task-quadrant {
  font-size: var(--fs-12);
  color: var(--color-text-muted);
  padding: 2px var(--sp-2);
  border-radius: var(--r-pill);
  background: var(--color-bg-subtle);
}

.fl-empty {
  text-align: center;
  padding: var(--sp-10) var(--sp-4);
  color: var(--color-text-muted);
  font-size: var(--fs-14);
}
</style>

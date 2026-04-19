<script setup lang="ts">
/**
 * ParkedView · 搁置区 — 对齐 prototype/screens/parked.html。
 * 搁置 ≠ 失败。展示已搁置任务,提供恢复/删除操作。
 */

import { onMounted, ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import { useTaskStore } from "@/stores/useTaskStore";
import type { Task } from "@/types";

const tasks = useTaskStore();
const shelvedTasks = ref<Task[]>([]);
const loading = ref(true);

async function loadShelved() {
  loading.value = true;
  try {
    // 查询所有 shelved 的任务
    const all = await invokeCmd<Task[]>("list_tasks", { statusFilter: "pending" });
    // shelved_at 不为空的
    shelvedTasks.value = all.filter((t) => t.shelved_at);
  } catch {
    // 如果过滤不行，用完整列表
    shelvedTasks.value = [];
  } finally {
    loading.value = false;
  }
}

async function restoreTask(task: Task) {
  try {
    // 恢复 = 清除 shelved_at（通过 update）
    await tasks.update({ id: task.id, name: task.name });
    shelvedTasks.value = shelvedTasks.value.filter((t) => t.id !== task.id);
  } catch (e) {
    console.error("[parked] restore failed", e);
  }
}

onMounted(loadShelved);

function daysSince(dateStr: string): number {
  const d = new Date(dateStr);
  const now = new Date();
  return Math.floor((now.getTime() - d.getTime()) / (1000 * 60 * 60 * 24));
}
</script>

<template>
  <section class="fl-parked">
    <header class="fl-parked-head">
      <div>
        <h1>🍃 搁置区</h1>
        <p class="fl-parked-sub">搁置 ≠ 失败 · 有些事情需要等合适的时机</p>
      </div>
    </header>

    <!-- 理念说明 -->
    <div class="fl-parked-philosophy">
      <div class="fl-phil-icon">🌿</div>
      <div>
        <strong>搁置是一种智慧</strong>
        <p>不是所有任务都适合现在做。搁置它们,等到合适的时候再捡起来。</p>
      </div>
    </div>

    <div v-if="loading" class="fl-empty">载入中…</div>

    <div v-else-if="shelvedTasks.length === 0" class="fl-empty">
      没有搁置的任务 · 一切都在推进中 ✨
    </div>

    <ul v-else class="fl-parked-list">
      <li v-for="t in shelvedTasks" :key="t.id" class="fl-parked-item">
        <div class="fl-parked-body">
          <div class="fl-parked-name">{{ t.name }}</div>
          <div class="fl-parked-meta">
            <span v-if="t.shelved_at">搁置 {{ daysSince(t.shelved_at) }} 天</span>
            <span v-if="t.estimated_minutes"> · 预估 {{ t.estimated_minutes }}m</span>
          </div>
        </div>
        <div class="fl-parked-actions">
          <button class="fl-p-btn fl-p-restore" @click="restoreTask(t)">恢复</button>
          <button class="fl-p-btn fl-p-danger" @click="tasks.remove(t.id); shelvedTasks = shelvedTasks.filter(x => x.id !== t.id)">删除</button>
        </div>
      </li>
    </ul>

    <!-- 自动降级规则说明 -->
    <div class="fl-parked-rules">
      <h3>自动降级规则</h3>
      <div class="fl-rules-timeline">
        <div class="fl-rule">
          <span class="fl-rule-day">D1</span>
          <span>搁置,保留在搁置区</span>
        </div>
        <div class="fl-rule">
          <span class="fl-rule-day">D7</span>
          <span>7 天未恢复,提示一次"还需要吗?"</span>
        </div>
        <div class="fl-rule">
          <span class="fl-rule-day">D30</span>
          <span>30 天未恢复,自动归档(不删除)</span>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.fl-parked {
  max-width: 640px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--sp-5);
}

.fl-parked-head h1 {
  font-size: var(--fs-24);
  font-weight: var(--fw-semibold);
  margin: 0;
}
.fl-parked-sub {
  margin: var(--sp-1) 0 0;
  color: var(--color-text-secondary);
  font-size: var(--fs-14);
}

.fl-parked-philosophy {
  display: flex;
  gap: var(--sp-3);
  padding: var(--sp-4);
  background: var(--color-primary-soft);
  border: 1px solid color-mix(in srgb, var(--color-primary) 20%, var(--color-border));
  border-radius: var(--r-md);
}
.fl-phil-icon { font-size: 24px; flex-shrink: 0; }
.fl-parked-philosophy strong { font-size: var(--fs-14); }
.fl-parked-philosophy p { font-size: var(--fs-12); color: var(--color-text-secondary); margin: 4px 0 0; }

.fl-parked-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}

.fl-parked-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-3);
  padding: var(--sp-3) var(--sp-4);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
}

.fl-parked-name {
  font-size: var(--fs-14);
  font-weight: var(--fw-medium);
}
.fl-parked-meta {
  font-size: var(--fs-12);
  color: var(--color-text-muted);
  margin-top: 2px;
}

.fl-parked-actions {
  display: flex;
  gap: var(--sp-2);
  flex-shrink: 0;
}

.fl-p-btn {
  padding: 4px 12px;
  border-radius: var(--r-sm);
  border: 1px solid var(--color-border);
  background: transparent;
  font-size: var(--fs-12);
  cursor: pointer;
  transition: all var(--dur-fast);
}
.fl-p-restore {
  color: var(--color-primary);
  border-color: var(--color-primary);
}
.fl-p-restore:hover {
  background: var(--color-primary-soft);
}
.fl-p-danger {
  color: var(--color-text-muted);
}
.fl-p-danger:hover {
  color: var(--color-q1);
  border-color: var(--color-q1);
}

.fl-parked-rules {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  padding: var(--sp-4);
}
.fl-parked-rules h3 {
  font-size: var(--fs-12);
  color: var(--color-text-muted);
  font-weight: var(--fw-medium);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin: 0 0 var(--sp-3);
}

.fl-rules-timeline {
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}
.fl-rule {
  display: flex;
  align-items: center;
  gap: var(--sp-3);
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
}
.fl-rule-day {
  width: 32px;
  text-align: center;
  font-family: var(--font-mono);
  font-weight: var(--fw-semibold);
  color: var(--color-primary);
  font-size: var(--fs-12);
}

.fl-empty {
  text-align: center;
  padding: var(--sp-8);
  color: var(--color-text-muted);
  font-size: var(--fs-14);
  background: var(--color-bg-subtle);
  border-radius: var(--r-md);
}
</style>

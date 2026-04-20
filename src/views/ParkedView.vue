<script setup lang="ts">
/**
 * ParkedView · 搁置区 — 对齐 prototype/screens/parked.html。
 * 搁置 ≠ 失败。展示已搁置任务,提供恢复/删除/拆分操作。
 * onMounted 调用 check_shelved_tasks 执行 D30 自动归档。
 */

import { onMounted, ref } from "vue";
import { X } from "lucide-vue-next";

import TaskDecomposePanel from "@/components/ai/TaskDecomposePanel.vue";
import { invokeCmd } from "@/composables/useTauriInvoke";
import { useTaskStore } from "@/stores/useTaskStore";
import type { Task } from "@/types";

const tasks = useTaskStore();
const shelvedTasks = ref<Task[]>([]);
const loading = ref(true);
const staleWarnings = ref<string[]>([]);
const decomposingTask = ref<Task | null>(null);

async function loadShelved() {
  loading.value = true;
  try {
    const all = await invokeCmd<Task[]>("list_tasks", { statusFilter: "pending" });
    shelvedTasks.value = all.filter((t) => t.shelved_at);
  } catch {
    shelvedTasks.value = [];
  } finally {
    loading.value = false;
  }
}

async function restoreTask(task: Task) {
  try {
    await invokeCmd("unshelve_task", { id: task.id });
    shelvedTasks.value = shelvedTasks.value.filter((t) => t.id !== task.id);
  } catch (e) {
    console.error("[parked] restore failed", e);
  }
}

onMounted(async () => {
  // 执行 D30 自动归档，D7-D29 返回提示列表
  try {
    const warnings = await invokeCmd<string[]>("check_shelved_tasks");
    staleWarnings.value = warnings;
  } catch { /* */ }
  await loadShelved();
});

function daysSince(dateStr: string): number {
  const d = new Date(dateStr);
  const now = new Date();
  return Math.floor((now.getTime() - d.getTime()) / (1000 * 60 * 60 * 24));
}

function openDecompose(task: Task) {
  decomposingTask.value = task;
}

/** 拆解采纳后,提示用户(原任务留在搁置区,用户决定是否删除) */
function onDecomposed() {
  // 采纳后关闭弹窗;原搁置任务保持不变,由用户手动决定是否"删除"
  decomposingTask.value = null;
}

/** 降级规则(对齐 prototype/screens/parked.html:493-524 四档) */
const DOWNGRADE_RULES = [
  { day: "D1", title: "第 1 天未完成", desc: "正常显示为「昨日遗留」· 顶部高亮" },
  { day: "D2", title: "第 2 天仍未完成", desc: "标记为「连续遗留」· AI 建议拆分或调整" },
  { day: "D3", title: "第 3 天仍未完成", desc: "系统弹卡片建议搁置 · 用户可选「继续 / 搁置 / 拆小 / 删除」" },
  { day: "D30", title: "搁置超过 30 天", desc: "温和提醒是否永久删除 · 不自动删" },
];

/** 语言原则(对齐 prototype parked.html:530 词表) */
const LANGUAGE_PRINCIPLES = [
  { avoid: "失败", use: "搁置" },
  { avoid: "落后", use: "调整" },
  { avoid: "拖延", use: "重新安排" },
  { avoid: "没完成", use: "暂时不做" },
  { avoid: "红色警告", use: "中性灰" },
];
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

    <!-- D7+ 提醒横幅 -->
    <div v-if="staleWarnings.length" class="fl-stale-banner">
      <strong>⏰ 以下任务已搁置超 7 天，考虑恢复或归档：</strong>
      <ul>
        <li v-for="(name, i) in staleWarnings" :key="i">{{ name }}</li>
      </ul>
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
          <button class="fl-p-btn fl-p-split" title="拆分成更小任务" @click="openDecompose(t)">拆分</button>
          <button class="fl-p-btn fl-p-danger" @click="tasks.remove(t.id); shelvedTasks = shelvedTasks.filter(x => x.id !== t.id)">删除</button>
        </div>
      </li>
    </ul>

    <!-- 自动降级规则说明(四档 D1/D2/D3/D30) -->
    <div class="fl-parked-rules">
      <h3>🔀 自动降级规则</h3>
      <p class="fl-rules-sub">不是一扔了之 · 系统会观察任务的状态,随着时间推移给出越来越温和的处理建议。</p>
      <div class="fl-rules-timeline">
        <div v-for="r in DOWNGRADE_RULES" :key="r.day" class="fl-rule">
          <span class="fl-rule-day">{{ r.day }}</span>
          <div class="fl-rule-body">
            <div class="fl-rule-title">{{ r.title }}</div>
            <div class="fl-rule-desc">{{ r.desc }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- 语言原则(对齐 parked.html:530) -->
    <div class="fl-parked-lang">
      <h3>💬 语言原则</h3>
      <p class="fl-rules-sub">FocusLab 在搁置场景下严格遵守的词汇原则</p>
      <table class="fl-lang-table">
        <thead>
          <tr>
            <th>❌ 不用</th>
            <th>✓ 用</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(p, i) in LANGUAGE_PRINCIPLES" :key="i">
            <td class="fl-lang-avoid">{{ p.avoid }}</td>
            <td class="fl-lang-use">{{ p.use }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 拆分 Modal -->
    <Transition name="fl-fade">
      <div
        v-if="decomposingTask"
        class="fl-dc-mask"
        role="presentation"
        @click.self="decomposingTask = null"
      >
        <div class="fl-dc-card" role="dialog" aria-modal="true">
          <button class="fl-dc-close-btn" type="button" @click="decomposingTask = null">
            <X :size="16" />
          </button>
          <TaskDecomposePanel
            :initial-name="decomposingTask.name"
            @close="decomposingTask = null"
            @adopted="onDecomposed"
          />
        </div>
      </div>
    </Transition>
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

.fl-stale-banner {
  padding: var(--sp-3) var(--sp-4);
  background: color-mix(in srgb, var(--color-warning) 10%, var(--color-bg-elevated));
  border: 1px solid color-mix(in srgb, var(--color-warning) 30%, var(--color-border));
  border-radius: var(--r-md); font-size: var(--fs-12);
}
.fl-stale-banner strong { color: var(--color-warning-text); }
.fl-stale-banner ul { margin: var(--sp-2) 0 0; padding-left: var(--sp-4); color: var(--color-text-secondary); }

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
.fl-p-split {
  color: var(--color-text-secondary);
}
.fl-p-split:hover {
  color: var(--color-primary);
  border-color: var(--color-primary);
  background: var(--color-primary-soft);
}
.fl-p-danger {
  color: var(--color-text-muted);
}
.fl-p-danger:hover {
  color: var(--color-q1);
  border-color: var(--color-q1);
}

/* ---------- Rules / Language 面板 ---------- */
.fl-parked-rules,
.fl-parked-lang {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  padding: var(--sp-4);
}
.fl-parked-rules h3,
.fl-parked-lang h3 {
  font-size: var(--fs-14);
  font-weight: var(--fw-semibold);
  margin: 0 0 var(--sp-1);
}
.fl-rules-sub {
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
  margin: 0 0 var(--sp-3);
  line-height: 1.6;
}

.fl-rules-timeline {
  display: flex;
  flex-direction: column;
  gap: var(--sp-3);
}
.fl-rule {
  display: flex;
  align-items: flex-start;
  gap: var(--sp-3);
}
.fl-rule-day {
  width: 36px;
  flex-shrink: 0;
  text-align: center;
  padding: 2px 0;
  border-radius: var(--r-pill);
  background: var(--color-primary-soft);
  font-family: var(--font-mono);
  font-weight: var(--fw-semibold);
  color: var(--color-primary);
  font-size: 11px;
}
.fl-rule-body { flex: 1; min-width: 0; }
.fl-rule-title { font-size: var(--fs-13, 13px); font-weight: var(--fw-medium); color: var(--color-text-primary); }
.fl-rule-desc { font-size: var(--fs-12); color: var(--color-text-secondary); margin-top: 2px; line-height: 1.5; }

/* Language principles table */
.fl-lang-table {
  width: 100%;
  font-size: var(--fs-13, 13px);
  border-collapse: collapse;
}
.fl-lang-table th {
  text-align: left;
  padding: 6px 0;
  border-bottom: 1px solid var(--color-border);
  font-size: 11px;
  color: var(--color-text-muted);
  font-weight: var(--fw-medium);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.fl-lang-table td { padding: 6px 0; }
.fl-lang-avoid {
  color: var(--color-text-muted);
  text-decoration: line-through;
}
.fl-lang-use {
  color: var(--color-success);
  font-weight: var(--fw-medium);
}

/* ---------- Decompose Modal ---------- */
.fl-dc-mask {
  position: fixed;
  inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 36%, transparent);
  display: grid;
  place-items: center;
  z-index: var(--z-modal);
  padding: var(--sp-4);
}
.fl-dc-card {
  position: relative;
  width: min(480px, 100%);
  max-height: 90vh;
  overflow-y: auto;
}
.fl-dc-close-btn {
  position: absolute;
  top: 10px;
  right: 10px;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  color: var(--color-text-muted);
  display: grid;
  place-items: center;
  cursor: pointer;
  z-index: 1;
}
.fl-dc-close-btn:hover {
  color: var(--color-q1);
  border-color: var(--color-q1);
}

.fl-empty {
  text-align: center;
  padding: var(--sp-8);
  color: var(--color-text-muted);
  font-size: var(--fs-14);
  background: var(--color-bg-subtle);
  border-radius: var(--r-md);
}

/* Transitions */
.fl-fade-enter-active, .fl-fade-leave-active { transition: opacity var(--dur-base) var(--ease-smooth); }
.fl-fade-enter-from, .fl-fade-leave-to { opacity: 0; }
</style>

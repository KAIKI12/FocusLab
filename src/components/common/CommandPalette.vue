<script setup lang="ts">
/**
 * CommandPalette · 全局命令面板 (Cmd+K / Ctrl+K)。
 * 快速搜索任务、跳转页面、执行操作。
 */

import { Search, X } from "lucide-vue-next";
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";

import { useTaskStore } from "@/stores/useTaskStore";
import { useTimerStore } from "@/stores/useTimerStore";

const visible = ref(false);
const query = ref("");
const inputEl = ref<HTMLInputElement | null>(null);
const selectedIndex = ref(0);

const tasks = useTaskStore();
const timer = useTimerStore();
const router = useRouter();

interface CmdItem {
  id: string;
  label: string;
  hint?: string;
  action: () => void;
}

const navItems: CmdItem[] = [
  { id: "nav-today", label: "今日", hint: "导航", action: () => router.push("/today") },
  { id: "nav-goals", label: "长线目标", hint: "导航", action: () => router.push("/goals") },
  { id: "nav-stats", label: "数据洞察", hint: "导航", action: () => router.push("/stats") },
  { id: "nav-settings", label: "设置", hint: "导航", action: () => router.push("/settings") },
];

const results = computed<CmdItem[]>(() => {
  const q = query.value.trim().toLowerCase();

  // 任务搜索
  const taskItems: CmdItem[] = tasks.tasks
    .filter((t) => !q || t.name.toLowerCase().includes(q))
    .slice(0, 8)
    .map((t) => ({
      id: `task-${t.id}`,
      label: t.name,
      hint: t.quadrant.replace(/_/g, " "),
      action: () => {
        if (timer.isIdle) {
          timer.startFree(t.id);
        }
      },
    }));

  // 导航 + 动作
  const filteredNav = navItems.filter(
    (n) => !q || n.label.toLowerCase().includes(q),
  );

  return [...filteredNav, ...taskItems];
});

watch(visible, (v) => {
  if (v) {
    query.value = "";
    selectedIndex.value = 0;
    nextTick(() => inputEl.value?.focus());
  }
});

watch(results, () => {
  selectedIndex.value = 0;
});

function toggle() {
  visible.value = !visible.value;
}

function onKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === "k") {
    e.preventDefault();
    toggle();
    return;
  }
  if (!visible.value) return;
  if (e.key === "Escape") {
    visible.value = false;
    return;
  }
  if (e.key === "ArrowDown") {
    e.preventDefault();
    selectedIndex.value = Math.min(selectedIndex.value + 1, results.value.length - 1);
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
  } else if (e.key === "Enter") {
    e.preventDefault();
    const item = results.value[selectedIndex.value];
    if (item) {
      item.action();
      visible.value = false;
    }
  }
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onUnmounted(() => window.removeEventListener("keydown", onKeydown));

function selectItem(item: CmdItem) {
  item.action();
  visible.value = false;
}
</script>

<template>
  <Transition name="fl-fade">
    <div
      v-if="visible"
      class="fl-cp-mask"
      role="presentation"
      @click.self="visible = false"
    >
      <div class="fl-cp-card" role="dialog" aria-modal="true">
        <div class="fl-cp-search">
          <Search :size="16" class="fl-cp-icon" />
          <input
            ref="inputEl"
            v-model="query"
            class="fl-cp-input"
            type="text"
            placeholder="搜索任务或命令…"
          />
          <button class="fl-cp-close" type="button" @click="visible = false">
            <X :size="14" />
          </button>
        </div>

        <ul v-if="results.length" class="fl-cp-list" role="listbox">
          <li
            v-for="(item, i) in results"
            :key="item.id"
            class="fl-cp-item"
            :class="{ 'is-selected': i === selectedIndex }"
            role="option"
            :aria-selected="i === selectedIndex"
            @click="selectItem(item)"
            @mouseenter="selectedIndex = i"
          >
            <span class="fl-cp-label">{{ item.label }}</span>
            <span v-if="item.hint" class="fl-cp-hint">{{ item.hint }}</span>
          </li>
        </ul>
        <div v-else class="fl-cp-empty">无匹配结果</div>

        <div class="fl-cp-footer">
          <kbd>↑↓</kbd> 选择 <kbd>↵</kbd> 确认 <kbd>esc</kbd> 关闭
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fl-cp-mask {
  position: fixed;
  inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 28%, transparent);
  display: flex;
  justify-content: center;
  padding-top: 15vh;
  z-index: var(--z-modal);
}

.fl-cp-card {
  width: min(520px, 90%);
  max-height: 420px;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-modal);
  display: flex;
  flex-direction: column;
  align-self: flex-start;
}

.fl-cp-search {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
  padding: var(--sp-3) var(--sp-4);
  border-bottom: 1px solid var(--color-border);
}

.fl-cp-icon { color: var(--color-text-muted); flex-shrink: 0; }

.fl-cp-input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  color: var(--color-text-primary);
  font-size: var(--fs-14);
}
.fl-cp-input::placeholder { color: var(--color-text-muted); }

.fl-cp-close {
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 2px;
}
.fl-cp-close:hover { color: var(--color-text-primary); }

.fl-cp-list {
  list-style: none;
  margin: 0;
  padding: var(--sp-2);
  overflow-y: auto;
  flex: 1;
}

.fl-cp-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sp-2) var(--sp-3);
  border-radius: var(--r-sm);
  cursor: pointer;
  transition: background var(--dur-fast) var(--ease-smooth);
}
.fl-cp-item.is-selected {
  background: var(--color-primary-soft);
}

.fl-cp-label {
  color: var(--color-text-primary);
  font-size: var(--fs-14);
}

.fl-cp-hint {
  font-size: 11px;
  color: var(--color-text-muted);
  padding: 1px var(--sp-2);
  background: var(--color-bg-subtle);
  border-radius: var(--r-pill);
}

.fl-cp-empty {
  padding: var(--sp-6);
  text-align: center;
  color: var(--color-text-muted);
  font-size: var(--fs-14);
}

.fl-cp-footer {
  padding: var(--sp-2) var(--sp-4);
  border-top: 1px solid var(--color-border);
  font-size: 11px;
  color: var(--color-text-muted);
  display: flex;
  gap: var(--sp-3);
  align-items: center;
}
.fl-cp-footer kbd {
  padding: 1px 4px;
  border: 1px solid var(--color-border);
  border-radius: 3px;
  font-size: 10px;
  background: var(--color-bg-subtle);
}

.fl-fade-enter-active,
.fl-fade-leave-active {
  transition: opacity var(--dur-base) var(--ease-smooth);
}
.fl-fade-enter-from,
.fl-fade-leave-to { opacity: 0; }
</style>

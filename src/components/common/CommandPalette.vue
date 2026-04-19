<script setup lang="ts">
/**
 * CommandPalette · 全局命令面板 (⌘/ 呼出)。
 * 支持 4 种输入模式：混合搜索 / > 命令 / # 任务 / @ 目标。
 */

import { Search, X } from "lucide-vue-next";
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";

import { useSettlementStore } from "@/stores/useSettlementStore";
import { useTaskStore } from "@/stores/useTaskStore";
import { useTimerStore } from "@/stores/useTimerStore";
import { useGoalStore } from "@/stores/useGoalStore";
import { useUIStore } from "@/stores/useUIStore";
import { useTheme } from "@/composables/useTheme";

const visible = ref(false);
const query = ref("");
const inputEl = ref<HTMLInputElement | null>(null);
const selectedIndex = ref(0);

const tasks = useTaskStore();
const timer = useTimerStore();
const settlement = useSettlementStore();
const goals = useGoalStore();
const ui = useUIStore();
const router = useRouter();
const { mode: themeMode, setMode } = useTheme();

interface CmdItem {
  id: string;
  label: string;
  hint?: string;
  keys?: string;
  action: () => void;
}

// ---------- 命令定义(对齐 docs/05 §8) ----------

const timerCmds: CmdItem[] = [
  {
    id: "cmd-pause", label: "暂停 / 继续专注", hint: "焦点", keys: "Space",
    action: () => { if (timer.isPaused) timer.resume(); else if (timer.isRunning) timer.pause(); },
  },
  {
    id: "cmd-abandon", label: "结束当前番茄钟", hint: "焦点",
    action: () => timer.abandon(),
  },
];

const taskCmds: CmdItem[] = [
  {
    id: "cmd-add-task", label: "快速添加任务", hint: "任务", keys: "⌘N",
    action: () => { ui.showQuickAdd = true; },
  },
  {
    id: "cmd-settle", label: "结束今天 · 进入日结算", hint: "日结算", keys: "⌘⇧E",
    action: () => settlement.settle(),
  },
  {
    id: "cmd-yesterday", label: "查看昨日复盘", hint: "日结算",
    action: () => settlement.loadYesterday(),
  },
];

const navCmds: CmdItem[] = [
  { id: "nav-today", label: "今日计划", hint: "导航", keys: "⌘1", action: () => router.push("/today") },
  { id: "nav-goals", label: "长线目标", hint: "导航", keys: "⌘2", action: () => router.push("/goals") },
  { id: "nav-calendar", label: "日历视图", hint: "导航", keys: "⌘3", action: () => router.push("/calendar") },
  { id: "nav-stats", label: "数据分析", hint: "导航", keys: "⌘4", action: () => router.push("/stats") },
  { id: "nav-settings", label: "设置", hint: "导航", keys: "⌘,", action: () => router.push("/settings") },
];

const settingCmds: CmdItem[] = [
  {
    id: "cmd-toggle-theme", label: "切换深色/浅色主题", hint: "设置", keys: "⌘⇧T",
    action: () => setMode(themeMode.value === "dark" ? "light" : "dark"),
  },
  {
    id: "cmd-toggle-sidebar", label: "切换侧边栏", hint: "设置",
    action: () => ui.toggleSidebar(),
  },
  {
    id: "cmd-toggle-sound", label: "切换音效", hint: "设置",
    action: () => ui.toggleSound(),
  },
];

const allCommands = [...timerCmds, ...taskCmds, ...navCmds, ...settingCmds];

// ---------- 搜索逻辑 ----------

const inputMode = computed(() => {
  const q = query.value;
  if (q.startsWith(">")) return "command";
  if (q.startsWith("#")) return "task";
  if (q.startsWith("@")) return "goal";
  return "mixed";
});

const modeHint = computed(() => {
  switch (inputMode.value) {
    case "command": return "命令模式";
    case "task": return "搜索任务";
    case "goal": return "跳转目标";
    default: return "";
  }
});

const searchQuery = computed(() => {
  const q = query.value.trim();
  if (inputMode.value !== "mixed") return q.slice(1).trim().toLowerCase();
  return q.toLowerCase();
});

const results = computed<CmdItem[]>(() => {
  const q = searchQuery.value;

  if (inputMode.value === "command") {
    return allCommands.filter((c) => !q || c.label.toLowerCase().includes(q));
  }

  if (inputMode.value === "task") {
    return tasks.tasks
      .filter((t) => !q || t.name.toLowerCase().includes(q))
      .slice(0, 12)
      .map((t) => ({
        id: `task-${t.id}`,
        label: t.name,
        hint: t.is_background ? "后台" : t.quadrant.replace(/_/g, " "),
        action: () => {
          if (!t.is_background && timer.isIdle) timer.startFree(t.id);
        },
      }));
  }

  if (inputMode.value === "goal") {
    return goals.goals
      .filter((g) => !q || g.name.toLowerCase().includes(q))
      .slice(0, 8)
      .map((g) => ({
        id: `goal-${g.id}`,
        label: g.name,
        hint: "目标",
        action: () => router.push("/goals"),
      }));
  }

  // mixed: 命令优先 → 任务
  const cmdResults = allCommands.filter(
    (c) => !q || c.label.toLowerCase().includes(q),
  );
  const taskResults = tasks.tasks
    .filter((t) => !q || t.name.toLowerCase().includes(q))
    .slice(0, 6)
    .map((t) => ({
      id: `task-${t.id}`,
      label: t.name,
      hint: t.is_background ? "后台" : t.quadrant.replace(/_/g, " "),
      action: () => {
        if (!t.is_background && timer.isIdle) timer.startFree(t.id);
      },
    }));
  return [...cmdResults, ...taskResults];
});

watch(visible, (v) => {
  if (v) {
    query.value = "";
    selectedIndex.value = 0;
    nextTick(() => inputEl.value?.focus());
  }
});

watch(results, () => { selectedIndex.value = 0; });

function onKeydown(e: KeyboardEvent) {
  // ⌘/ 或 Ctrl+/ 唤起
  if ((e.metaKey || e.ctrlKey) && e.key === "/") {
    e.preventDefault();
    visible.value = !visible.value;
    return;
  }
  if (!visible.value) return;
  if (e.key === "Escape") { visible.value = false; return; }
  if (e.key === "ArrowDown") {
    e.preventDefault();
    selectedIndex.value = Math.min(selectedIndex.value + 1, results.value.length - 1);
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
  } else if (e.key === "Enter") {
    e.preventDefault();
    const item = results.value[selectedIndex.value];
    if (item) { item.action(); visible.value = false; }
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
            :placeholder="modeHint || '搜索任务或命令… (> 命令 / # 任务 / @ 目标)'"
          />
          <span v-if="modeHint" class="fl-cp-mode">{{ modeHint }}</span>
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
            <span class="fl-cp-right">
              <span v-if="item.keys" class="fl-cp-keys">{{ item.keys }}</span>
              <span v-if="item.hint" class="fl-cp-hint">{{ item.hint }}</span>
            </span>
          </li>
        </ul>
        <div v-else class="fl-cp-empty">无匹配结果</div>

        <div class="fl-cp-footer">
          <kbd>↑↓</kbd> 选择 <kbd>↵</kbd> 确认 <kbd>esc</kbd> 关闭
          <span class="fl-cp-sep">|</span>
          <kbd>></kbd> 命令 <kbd>#</kbd> 任务 <kbd>@</kbd> 目标
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
  z-index: calc(var(--z-modal) + 1);
}

.fl-cp-card {
  width: min(560px, 90%);
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

.fl-cp-mode {
  font-size: 11px;
  color: var(--color-primary);
  padding: 2px var(--sp-2);
  background: var(--color-primary-soft);
  border-radius: var(--r-pill);
  white-space: nowrap;
}

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

.fl-cp-right {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
}

.fl-cp-keys {
  font-size: 10px;
  color: var(--color-text-muted);
  padding: 1px 4px;
  border: 1px solid var(--color-border);
  border-radius: 3px;
  font-family: var(--font-mono, monospace);
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
  gap: var(--sp-2);
  align-items: center;
}
.fl-cp-footer kbd {
  padding: 1px 4px;
  border: 1px solid var(--color-border);
  border-radius: 3px;
  font-size: 10px;
  background: var(--color-bg-subtle);
}
.fl-cp-sep { color: var(--color-border); }

.fl-fade-enter-active,
.fl-fade-leave-active {
  transition: opacity var(--dur-base) var(--ease-smooth);
}
.fl-fade-enter-from,
.fl-fade-leave-to { opacity: 0; }
</style>

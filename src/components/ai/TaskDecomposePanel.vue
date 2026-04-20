<script setup lang="ts">
/**
 * TaskDecomposePanel · AI 任务拆解面板。
 *
 * 输入任务名 → AI 返回子任务列表 → 用户可采纳/编辑/拒绝。
 * 采纳时批量创建 tasks。
 */

import { Sparkles, X } from "lucide-vue-next";
import { ref, watch } from "vue";

import { useAIStore, type SubTask } from "@/stores/useAIStore";
import { useTaskStore } from "@/stores/useTaskStore";

const props = defineProps<{ initialName?: string; initialDescription?: string }>();
const emit = defineEmits<{ close: []; adopted: [] }>();

const ai = useAIStore();
const tasks = useTaskStore();

const taskName = ref(props.initialName ?? "");
const description = ref(props.initialDescription ?? "");
const subTasks = ref<SubTask[]>([]);
const adopted = ref(false);

// 支持外部切换预填对象(例如在搁置区切换不同任务拆分)
watch(
  () => [props.initialName, props.initialDescription],
  ([n, d]) => {
    taskName.value = n ?? "";
    description.value = d ?? "";
    subTasks.value = [];
    adopted.value = false;
  },
);

async function onDecompose() {
  if (!taskName.value.trim()) return;
  try {
    subTasks.value = await ai.decomposeTask(
      taskName.value.trim(),
      description.value.trim() || undefined,
    );
    adopted.value = false;
  } catch {
    subTasks.value = [];
  }
}

async function onAdopt() {
  for (const st of subTasks.value) {
    await tasks.create({
      name: st.name,
      quadrant: st.quadrant,
    });
  }
  adopted.value = true;
  emit("adopted");
}

function onRemoveSub(idx: number) {
  subTasks.value.splice(idx, 1);
}
</script>

<template>
  <div class="fl-decompose">
    <div class="fl-dc-head">
      <Sparkles :size="14" />
      <span>AI 任务拆解</span>
      <button class="fl-dc-close" type="button" @click="emit('close')">
        <X :size="14" />
      </button>
    </div>

    <div class="fl-dc-input">
      <input
        v-model="taskName"
        class="fl-dc-field"
        type="text"
        placeholder="输入大任务名…"
        maxlength="80"
      />
      <textarea
        v-model="description"
        class="fl-dc-field fl-dc-area"
        placeholder="描述(可选)…"
        rows="2"
        maxlength="300"
      />
      <button
        class="fl-dc-btn"
        type="button"
        :disabled="!taskName.trim() || ai.loading"
        @click="onDecompose"
      >
        {{ ai.loading ? '拆解中…' : '🪄 AI 拆解' }}
      </button>
    </div>

    <div v-if="subTasks.length" class="fl-dc-results">
      <div class="fl-dc-result-head">
        <span>拆解结果({{ subTasks.length }} 项)</span>
        <button
          v-if="!adopted"
          class="fl-dc-adopt"
          type="button"
          @click="onAdopt"
        >
          全部采纳
        </button>
        <span v-else class="fl-dc-adopted">已采纳 ✓</span>
      </div>
      <div
        v-for="(st, idx) in subTasks"
        :key="idx"
        class="fl-dc-item"
      >
        <span class="fl-dc-name">{{ st.name }}</span>
        <span class="fl-dc-est">{{ st.estimatedMinutes }}min</span>
        <button class="fl-dc-rm" type="button" @click="onRemoveSub(idx)">
          <X :size="10" />
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fl-decompose {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  padding: var(--sp-4);
  display: flex;
  flex-direction: column;
  gap: var(--sp-3);
}

.fl-dc-head {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
  font-size: var(--fs-14);
  font-weight: var(--fw-semibold);
  color: var(--color-primary);
}
.fl-dc-close {
  margin-left: auto;
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
}

.fl-dc-input {
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}
.fl-dc-field {
  padding: var(--sp-2) var(--sp-3);
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-subtle);
  color: var(--color-text-primary);
  font-size: var(--fs-12);
  outline: none;
  font-family: inherit;
}
.fl-dc-field:focus {
  border-color: var(--color-primary);
}
.fl-dc-area {
  resize: none;
}

.fl-dc-btn {
  padding: var(--sp-2) var(--sp-4);
  border-radius: var(--r-md);
  border: none;
  background: var(--color-primary);
  color: var(--color-text-on-primary);
  font-size: var(--fs-12);
  cursor: pointer;
}
.fl-dc-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.fl-dc-results {
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}
.fl-dc-result-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
}
.fl-dc-adopt {
  padding: 4px var(--sp-3);
  border-radius: var(--r-md);
  border: none;
  background: var(--color-success);
  color: #fff;
  font-size: 11px;
  cursor: pointer;
}
.fl-dc-adopted {
  color: var(--color-success);
  font-size: 11px;
}

.fl-dc-item {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
  padding: var(--sp-2) var(--sp-3);
  background: var(--color-bg-subtle);
  border-radius: var(--r-sm);
}
.fl-dc-name {
  flex: 1;
  font-size: var(--fs-12);
  color: var(--color-text-primary);
}
.fl-dc-est {
  font-size: 10px;
  color: var(--color-text-muted);
  font-variant-numeric: tabular-nums;
}
.fl-dc-rm {
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  padding: 2px;
}
.fl-dc-rm:hover {
  color: var(--color-q1);
}
</style>

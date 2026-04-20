<script setup lang="ts">
/**
 * ExportModal · 数据导出弹窗。
 * 对齐 prototype/screens/modals.html §5 Data Export。
 * 3 格式 + 5 内容开关。
 */

import { Download, FileJson, FileText, Table } from "lucide-vue-next";
import { ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";

defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: [] }>();

const format = ref<"json" | "markdown" | "csv">("json");
const loading = ref(false);
const result = ref("");

const content = ref({
  tasks: true,
  goals: true,
  sessions: true,
  settlements: true,
  ai: false,
});

const formats = [
  { value: "json" as const, label: "JSON", desc: "完整数据，可导入", icon: FileJson },
  { value: "markdown" as const, label: "Markdown", desc: "人类可读", icon: FileText },
  { value: "csv" as const, label: "CSV", desc: "Excel 兼容", icon: Table },
];

async function onExport() {
  loading.value = true;
  result.value = "";
  try {
    if (format.value === "json") {
      const data = await invokeCmd<string>("export_tasks_json");
      result.value = `JSON 导出成功 (${(data.length / 1024).toFixed(1)} KB)`;
    } else if (format.value === "csv") {
      const data = await invokeCmd<string>("export_sessions_csv");
      result.value = `CSV 导出成功 (${(data.length / 1024).toFixed(1)} KB)`;
    } else {
      result.value = "Markdown 导出暂未实现";
    }
  } catch (e) {
    result.value = `导出失败: ${e}`;
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <Transition name="fl-fade">
    <div v-if="visible" class="fl-modal-mask" @click.self="emit('close')">
      <div class="fl-em" role="dialog" aria-modal="true">
        <header class="fl-em-head">
          <h2><Download :size="16" /> 导出数据</h2>
          <button class="fl-em-close" @click="emit('close')">✕</button>
        </header>

        <div class="fl-em-body">
          <!-- Format grid -->
          <div class="fl-em-formats">
            <div
              v-for="f in formats" :key="f.value"
              class="fl-em-fmt" :class="{ 'is-selected': format === f.value }"
              @click="format = f.value"
            >
              <component :is="f.icon" :size="20" />
              <span class="fl-em-ext">.{{ f.value === 'markdown' ? 'md' : f.value }}</span>
              <span class="fl-em-desc">{{ f.desc }}</span>
            </div>
          </div>

          <!-- Content toggles -->
          <div class="fl-em-toggles">
            <label class="fl-em-toggle">
              <input v-model="content.tasks" type="checkbox" />
              <span>任务</span>
            </label>
            <label class="fl-em-toggle">
              <input v-model="content.goals" type="checkbox" />
              <span>目标</span>
            </label>
            <label class="fl-em-toggle">
              <input v-model="content.sessions" type="checkbox" />
              <span>专注记录</span>
            </label>
            <label class="fl-em-toggle">
              <input v-model="content.settlements" type="checkbox" />
              <span>日结算</span>
            </label>
            <label class="fl-em-toggle">
              <input v-model="content.ai" type="checkbox" />
              <span>AI 交互</span>
            </label>
          </div>

          <div v-if="result" class="fl-em-result" :class="{ 'is-error': result.includes('失败') }">
            {{ result }}
          </div>
        </div>

        <footer class="fl-em-foot">
          <button class="fl-btn fl-btn-ghost" @click="emit('close')">取消</button>
          <button class="fl-btn fl-btn-primary" :disabled="loading" @click="onExport">
            <Download :size="14" /> {{ loading ? '导出中...' : '导出' }}
          </button>
        </footer>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fl-modal-mask {
  position: fixed; inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 32%, transparent);
  display: grid; place-items: center;
  z-index: var(--z-modal); padding: var(--sp-4);
}

.fl-em {
  width: min(460px, 100%);
  max-height: calc(100vh - 32px);
  overflow-x: hidden;
  overflow-y: auto;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-modal);
}

.fl-em-head {
  display: flex; align-items: center; justify-content: space-between;
  padding: var(--sp-4) var(--sp-5);
  border-bottom: 1px solid var(--color-border);
}
.fl-em-head h2 {
  display: flex; align-items: center; gap: var(--sp-2);
  font-size: var(--fs-16); font-weight: var(--fw-semibold); margin: 0;
}
.fl-em-close {
  background: none; border: none; color: var(--color-text-muted);
  cursor: pointer; font-size: 14px;
}
.fl-em-close:hover { color: var(--color-text-primary); }

.fl-em-body { padding: var(--sp-4) var(--sp-5); }

/* Format grid */
.fl-em-formats { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--sp-2); margin-bottom: var(--sp-4); }
.fl-em-fmt {
  display: flex; flex-direction: column; align-items: center; gap: 4px;
  padding: var(--sp-3); border-radius: var(--r-md);
  border: 1.5px solid var(--color-border);
  cursor: pointer; transition: all var(--dur-fast);
  color: var(--color-text-secondary);
}
.fl-em-fmt:hover { border-color: var(--color-primary); }
.fl-em-fmt.is-selected {
  border-color: var(--color-primary);
  background: color-mix(in srgb, var(--color-primary) 8%, transparent);
  color: var(--color-primary);
}
.fl-em-ext { font-size: 11px; font-family: var(--font-mono, monospace); font-weight: var(--fw-semibold); }
.fl-em-desc { font-size: 10px; color: var(--color-text-muted); }

/* Content toggles */
.fl-em-toggles { display: flex; flex-direction: column; gap: var(--sp-2); }
.fl-em-toggle {
  display: flex; align-items: center; gap: var(--sp-2);
  padding: 6px var(--sp-3); border-radius: var(--r-sm);
  font-size: var(--fs-12); cursor: pointer;
}
.fl-em-toggle:hover { background: var(--color-bg-hover); }
.fl-em-toggle input { accent-color: var(--color-primary); }

.fl-em-result {
  margin-top: var(--sp-3); padding: var(--sp-2) var(--sp-3);
  border-radius: var(--r-sm); font-size: var(--fs-12);
  background: color-mix(in srgb, var(--color-success) 10%, transparent);
  color: var(--color-success);
}
.fl-em-result.is-error {
  background: color-mix(in srgb, #ef4444 10%, transparent);
  color: #ef4444;
}

/* Footer */
.fl-em-foot {
  display: flex; justify-content: flex-end; gap: var(--sp-2);
  padding: var(--sp-3) var(--sp-5);
  border-top: 1px solid var(--color-border);
  background: var(--color-bg-subtle);
}
.fl-btn {
  display: inline-flex; align-items: center; gap: 6px;
  padding: 8px 16px; border-radius: var(--r-md);
  font-size: var(--fs-12); font-weight: var(--fw-medium);
  border: 1px solid transparent; cursor: pointer;
}
.fl-btn-ghost { background: transparent; color: var(--color-text-secondary); border-color: var(--color-border); }
.fl-btn-ghost:hover { background: var(--color-bg-hover); }
.fl-btn-primary { background: var(--color-primary); color: var(--color-text-on-primary, #fff); }
.fl-btn-primary:hover:not(:disabled) { opacity: 0.9; }
.fl-btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

.fl-fade-enter-active, .fl-fade-leave-active { transition: opacity var(--dur-base) var(--ease-smooth); }
.fl-fade-enter-from, .fl-fade-leave-to { opacity: 0; }
</style>

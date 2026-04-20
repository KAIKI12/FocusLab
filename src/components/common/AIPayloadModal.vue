<script setup lang="ts">
/**
 * AIPayloadModal · 发送给 AI 的数据示例弹窗。
 * 对齐 prototype/screens/modals.html §5 AI Payload 预览。
 * 静态展示各 AI 调用场景的 System Prompt + User Prompt 示例,
 * 用于帮助用户透明理解"会发送什么"。
 */

import { ChevronDown, Check, X } from "lucide-vue-next";
import { computed, ref } from "vue";

defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: [] }>();

interface Scene {
  key: string;
  label: string;
  systemPromptNote: string;
  systemPromptBody: string;
  userPrompt: string;
}

const SCENES: Scene[] = [
  {
    key: "settlement",
    label: "日结算寄语",
    systemPromptNote: "// 包含:语气风格、安全边界、格式要求(全文约 280 字,可在设置中查看)",
    systemPromptBody: "You are FocusLab, a warm companion for graduate students.\nBe warm, avoid toxic productivity language...",
    userPrompt: JSON.stringify(
      {
        date: "2026-04-17",
        level: "A",
        completion_rate: 1.0,
        focus_time: "7h 15m",
        pomodoro_count: 12,
        tasks_completed: [
          "复现 baseline 代码",
          "回复导师邮件",
          "阅读文献 2 篇",
        ],
        related_goal: {
          name: "论文 A",
          current_milestone: "M3 方法论",
          milestone_progress: 52,
        },
        tone: "温暖鼓励",
      },
      null,
      2,
    ),
  },
  {
    key: "daily_suggestions",
    label: "每日建议",
    systemPromptNote: "// 包含:注意力曲线引导、当日象限分布解读",
    systemPromptBody: "You are FocusLab's daily suggestion advisor.\nGive actionable, non-judgmental nudges...",
    userPrompt: JSON.stringify(
      {
        date: "2026-04-20",
        energy_level: "正常",
        quadrant_counts: {
          important_urgent: 2,
          important_not_urgent: 3,
          not_important_urgent: 1,
          not_important_not_urgent: 0,
        },
      },
      null,
      2,
    ),
  },
  {
    key: "decompose",
    label: "任务拆解",
    systemPromptNote: "// 包含:返回 JSON 格式约束、子任务粒度建议",
    systemPromptBody: "You are a task decomposer. Return JSON array of subtasks...",
    userPrompt: JSON.stringify(
      {
        task_name: "复现 baseline 代码",
        description: "跑通作者的官方仓库",
      },
      null,
      2,
    ),
  },
  {
    key: "classify",
    label: "象限分类",
    systemPromptNote: "// 返回单个象限 key,不夹杂其它内容",
    systemPromptBody: "Classify the task into one of 4 quadrants (important_urgent, important_not_urgent, not_important_urgent, not_important_not_urgent)...",
    userPrompt: JSON.stringify(
      {
        task_name: "整理桌面",
        description: "",
      },
      null,
      2,
    ),
  },
  {
    key: "weekly_summary",
    label: "周度小结",
    systemPromptNote: "// 关注趋势与亮点,不罗列全部数据",
    systemPromptBody: "Summarize the past 7 days with highlights and gentle suggestions...",
    userPrompt: JSON.stringify(
      {
        week_range: "2026-04-14 → 2026-04-20",
        total_focus_minutes: 1485,
        total_pomodoros: 56,
        completion_rate_avg: 0.72,
        best_day: { date: "2026-04-17", grade: "A" },
      },
      null,
      2,
    ),
  },
];

const currentKey = ref<string>(SCENES[0].key);
const showPicker = ref(false);

const currentScene = computed<Scene>(
  () => SCENES.find((s) => s.key === currentKey.value) ?? SCENES[0],
);

function pickScene(key: string) {
  currentKey.value = key;
  showPicker.value = false;
}
</script>

<template>
  <Transition name="fl-fade">
    <div v-if="visible" class="fl-modal-mask" @click.self="emit('close')">
      <div class="fl-ap" role="dialog" aria-modal="true">
        <header class="fl-ap-head">
          <div class="fl-ap-title">
            <h3>发送给 AI 的数据示例</h3>
            <p class="fl-ap-sub">这是「{{ currentScene.label }}」调用时的实际 payload · 不是示意</p>
          </div>
          <div class="fl-ap-head-right">
            <span class="fl-ap-chip">{{ currentScene.label }}</span>
            <button class="fl-ap-icon-btn" type="button" @click="emit('close')">
              <X :size="14" />
            </button>
          </div>
        </header>

        <div class="fl-ap-body">
          <h4>System Prompt (FocusLab 预设)</h4>
          <pre class="fl-code-block"><span class="fl-code-comment">{{ currentScene.systemPromptNote }}</span>
{{ currentScene.systemPromptBody }}</pre>

          <h4>User Prompt (本次调用)</h4>
          <pre class="fl-code-block">{{ currentScene.userPrompt }}</pre>

          <div class="fl-ap-note">
            <Check :size="14" />
            <span>不含用户姓名、学校、邮箱等 PII · 不含其它任务的详细内容 · 不含完整历史</span>
          </div>
        </div>

        <footer class="fl-ap-foot">
          <div class="fl-ap-picker-wrap">
            <button class="fl-btn fl-btn-ghost" type="button" @click="showPicker = !showPicker">
              选择其它调用场景 <ChevronDown :size="12" />
            </button>
            <div v-if="showPicker" class="fl-ap-picker">
              <button
                v-for="s in SCENES"
                :key="s.key"
                type="button"
                class="fl-ap-picker-item"
                :class="{ 'is-active': s.key === currentKey }"
                @click="pickScene(s.key)"
              >
                {{ s.label }}
              </button>
            </div>
          </div>
          <button class="fl-btn fl-btn-primary" type="button" @click="emit('close')">
            知道了
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

.fl-ap {
  width: min(560px, 100%);
  max-height: 85vh;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-modal);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.fl-ap-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--sp-3);
  padding: var(--sp-4) var(--sp-5);
  border-bottom: 1px solid var(--color-border);
}
.fl-ap-title h3 {
  font-size: var(--fs-14);
  font-weight: var(--fw-semibold);
  margin: 0 0 2px;
}
.fl-ap-sub {
  font-size: var(--fs-12);
  color: var(--color-text-muted);
  margin: 0;
}
.fl-ap-head-right {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
}
.fl-ap-chip {
  padding: 2px 10px;
  border-radius: var(--r-pill);
  background: var(--color-primary-soft);
  color: var(--color-primary-dark, var(--color-primary));
  font-size: 11px;
  font-weight: var(--fw-medium);
  white-space: nowrap;
}
.fl-ap-icon-btn {
  width: 28px; height: 28px;
  border-radius: var(--r-sm);
  border: 1px solid var(--color-border);
  background: transparent;
  color: var(--color-text-muted);
  display: grid; place-items: center;
  cursor: pointer;
}
.fl-ap-icon-btn:hover { color: var(--color-q1); border-color: var(--color-q1); }

/* Body */
.fl-ap-body {
  padding: var(--sp-4) var(--sp-5);
  overflow-y: auto;
  flex: 1;
}
.fl-ap-body h4 {
  font-size: 11px;
  font-weight: var(--fw-semibold);
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin: var(--sp-3) 0 var(--sp-2);
}
.fl-ap-body h4:first-child { margin-top: 0; }

.fl-code-block {
  margin: 0;
  padding: var(--sp-3);
  background: var(--color-bg-subtle);
  border: 1px solid var(--color-border);
  border-radius: var(--r-sm);
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.55;
  color: var(--color-text-primary);
  white-space: pre-wrap;
  word-break: break-word;
  overflow-x: auto;
}
.fl-code-comment {
  color: var(--color-text-muted);
  font-style: italic;
  display: block;
  margin-bottom: 4px;
}

.fl-ap-note {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  margin-top: var(--sp-3);
  padding: 10px 12px;
  background: color-mix(in srgb, var(--color-success) 10%, transparent);
  color: var(--color-success);
  border-radius: var(--r-sm);
  font-size: var(--fs-12);
  line-height: 1.55;
}

/* Footer */
.fl-ap-foot {
  display: flex;
  gap: var(--sp-2);
  padding: var(--sp-3) var(--sp-5);
  border-top: 1px solid var(--color-border);
  background: var(--color-bg-subtle);
  justify-content: space-between;
}
.fl-ap-picker-wrap { position: relative; }
.fl-ap-picker {
  position: absolute;
  bottom: calc(100% + 6px);
  left: 0;
  min-width: 160px;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  box-shadow: var(--shadow-card);
  padding: 4px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  z-index: 1;
}
.fl-ap-picker-item {
  text-align: left;
  padding: 6px 10px;
  border-radius: var(--r-sm);
  background: transparent;
  border: none;
  font-size: var(--fs-12);
  color: var(--color-text-primary);
  cursor: pointer;
}
.fl-ap-picker-item:hover { background: var(--color-bg-hover); }
.fl-ap-picker-item.is-active {
  background: var(--color-primary-soft);
  color: var(--color-primary);
  font-weight: var(--fw-medium);
}

.fl-btn {
  padding: 6px 14px;
  border-radius: var(--r-md);
  font-size: var(--fs-12);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border: 1px solid transparent;
}
.fl-btn-ghost {
  background: transparent;
  border-color: var(--color-border);
  color: var(--color-text-secondary);
}
.fl-btn-ghost:hover { border-color: var(--color-primary); color: var(--color-primary); }
.fl-btn-primary {
  background: var(--color-primary);
  color: #fff;
  border-color: var(--color-primary);
}
.fl-btn-primary:hover { background: var(--color-primary-dark, var(--color-primary)); }

/* Transitions */
.fl-fade-enter-active, .fl-fade-leave-active { transition: opacity var(--dur-base) var(--ease-smooth); }
.fl-fade-enter-from, .fl-fade-leave-to { opacity: 0; }
</style>

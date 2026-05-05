<script setup lang="ts">
/**
 * InspirationCorrectionPanel · AI 纠偏分析结果面板。
 *
 * 当 AI 推荐 contradicts 类型时,父组件可触发 ai.analyzeCorrection,
 * 获得结构化"旧判断 / 新证据 / 建议"。本组件展示这一结果,并提供 3 个动作:
 * 1. 标记旧灵感待复查
 * 2. 建立修正连接(直接 accept 推荐)
 * 3. 起草后续实验(创建子灵感)
 */

import { Sparkles } from "lucide-vue-next";

export interface CorrectionResult {
  summary: string;
  oldJudgment: string;
  newEvidence: string;
  suggestion: string;
}

defineProps<{
  result: CorrectionResult;
}>();

const emit = defineEmits<{
  (e: "mark-needs-check"): void;
  (e: "mark-overturned"): void;
  (e: "accept-correction"): void;
  (e: "create-followup"): void;
}>();
</script>

<template>
  <div class="fl-correction" @click.stop>
    <div class="fl-correction-head">
      <Sparkles :size="12" />
      <strong>{{ result.summary }}</strong>
    </div>
    <dl class="fl-correction-grid">
      <dt>旧判断</dt>
      <dd>{{ result.oldJudgment }}</dd>
      <dt>新证据</dt>
      <dd>{{ result.newEvidence }}</dd>
      <dt>建议</dt>
      <dd>{{ result.suggestion }}</dd>
    </dl>
    <div class="fl-correction-actions">
      <button
        class="fl-correction-btn fl-correction-btn-warn"
        type="button"
        @click.stop="emit('mark-needs-check')"
      >
        标记旧灵感待复查
      </button>
      <button
        class="fl-correction-btn"
        type="button"
        @click.stop="emit('accept-correction')"
      >
        建立修正连接
      </button>
      <button
        class="fl-correction-btn fl-correction-btn-danger"
        type="button"
        title="有充分证据反驳旧灵感时使用"
        @click.stop="emit('mark-overturned')"
      >
        标记已推翻
      </button>
      <button
        class="fl-correction-btn fl-correction-btn-ai"
        type="button"
        @click.stop="emit('create-followup')"
      >
        起草后续实验
      </button>
    </div>
  </div>
</template>

<style scoped>
.fl-correction {
  margin-top: 6px;
  padding: 8px 10px;
  border-radius: var(--r-sm);
  background: color-mix(in srgb, var(--color-q1, #ef4444) 5%, transparent);
  border-left: 2px solid color-mix(in srgb, var(--color-q1, #ef4444) 60%, transparent);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.fl-correction-head {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  color: var(--color-text-primary);
}
.fl-correction-grid {
  display: grid;
  grid-template-columns: 56px 1fr;
  gap: 3px 8px;
  margin: 0;
  font-size: 11.5px;
  line-height: 1.55;
}
.fl-correction-grid dt {
  color: var(--color-text-muted);
  font-weight: 600;
}
.fl-correction-grid dd {
  margin: 0;
  color: var(--color-text-primary);
}
.fl-correction-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 2px;
}
.fl-correction-btn {
  height: 26px;
  padding: 0 10px;
  border-radius: var(--r-sm);
  border: 1px solid color-mix(in srgb, var(--color-primary) 20%, transparent);
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
  font-size: 12px;
  font-weight: var(--fw-medium);
  cursor: pointer;
  transition: all 120ms;
}
.fl-correction-btn:hover {
  opacity: 0.85;
}
.fl-correction-btn-warn {
  color: var(--color-warning-text, #b45309);
  background: color-mix(in srgb, var(--color-warning) 10%, transparent);
  border-color: color-mix(in srgb, var(--color-warning) 18%, transparent);
}
.fl-correction-btn-danger {
  color: var(--color-q1, #b91c1c);
  background: color-mix(in srgb, var(--color-q1, #ef4444) 10%, transparent);
  border-color: color-mix(in srgb, var(--color-q1, #ef4444) 30%, transparent);
}
.fl-correction-btn-ai {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border: 1px dashed color-mix(in srgb, var(--color-primary) 50%, transparent);
  background: color-mix(in srgb, var(--color-primary) 6%, transparent);
  color: var(--color-primary);
}
.fl-correction-btn-ai:hover {
  background: color-mix(in srgb, var(--color-primary) 14%, transparent);
}
</style>

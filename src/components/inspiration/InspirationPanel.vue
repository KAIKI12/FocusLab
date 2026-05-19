<script setup lang="ts">
/**
 * InspirationPanel · 灵感速记面板(今日页右栏简版)。
 * - 速记输入 + 保存
 * - 卡片列表(最近 3 条)
 * - "查看全部" 跳转到 /inspirations 完整页
 * - 一键转任务 / 删除
 */

import { ArrowRight, Check, Lightbulb, Plus, Trash2, X } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";

import { useInspirationImageDraft } from "@/composables/useInspirationImageDraft";
import { getInspirationImageSrc } from "@/composables/useInspirationImageAsset";
import { renderMarkdown } from "@/composables/useMarkdown";
import { useInspirationStore } from "@/stores/useInspirationStore";

const inspiration = useInspirationStore();
const router = useRouter();

const draft = ref("");
const textareaEl = ref<HTMLTextAreaElement | null>(null);
const justSaved = ref<string | null>(null); // 刚保存的卡片 id,用于短暂高亮
const { imageDraft, imageError, clearImage, handlePasteImage, toUploadPayload } = useInspirationImageDraft();
const canSave = computed(() => !!draft.value.trim() || !!imageDraft.value);

onMounted(() => {
  inspiration.ensureLoaded();
});

async function onSave() {
  const item = await inspiration.create(draft.value, {
    image: await toUploadPayload(),
  });
  draft.value = "";
  clearImage();
  if (item) {
    justSaved.value = item.id;
    setTimeout(() => {
      justSaved.value = null;
    }, 1800);
  }
  textareaEl.value?.focus();
}

function onKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
    e.preventDefault();
    onSave();
  }
}

function onPaste(event: ClipboardEvent) {
  handlePasteImage(event);
}

async function onConvert(id: string) {
  await inspiration.convertToTask(id);
}

function fmtTime(iso: string) {
  try {
    const d = new Date(iso);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffMin = Math.floor(diffMs / 60_000);
    if (diffMin < 1) return "刚刚";
    if (diffMin < 60) return `${diffMin} 分钟前`;
    const diffH = Math.floor(diffMin / 60);
    if (diffH < 24) return `${diffH} 小时前`;
    return d.toLocaleDateString("zh-CN", { month: "numeric", day: "numeric" });
  } catch {
    return "";
  }
}
</script>

<template>
  <div class="fl-insp">
    <!-- 标题行 -->
    <div class="fl-insp-head">
      <div class="fl-insp-title">
        <Lightbulb :size="16" class="fl-insp-icon" />
        <span>灵感</span>
        <span v-if="inspiration.pendingCount > 0" class="fl-insp-badge">
          {{ inspiration.pendingCount }}
        </span>
      </div>
    </div>

    <!-- 速记输入区 -->
    <div class="fl-insp-input-wrap">
      <textarea
        ref="textareaEl"
        v-model="draft"
        class="fl-insp-textarea"
        placeholder="支持 Markdown，也可以直接粘贴截图… (⌘↵ 保存)"
        rows="3"
        spellcheck="false"
        @keydown="onKeydown"
        @paste="onPaste"
      />
      <div v-if="imageDraft" class="fl-insp-image-draft">
        <img :src="imageDraft.previewUrl" alt="待保存的灵感截图" class="fl-insp-image-preview" />
        <button class="fl-insp-image-remove" type="button" @click="clearImage">
          <X :size="12" />
          移除图片
        </button>
      </div>
      <p v-if="imageError" class="fl-insp-image-error">{{ imageError }}</p>
      <div class="fl-insp-input-foot">
        <span class="fl-insp-hint">{{ draft.length }} 字 · 支持 Markdown / 截图粘贴</span>
        <button
          class="fl-insp-save"
          type="button"
          :disabled="!canSave"
          @click="onSave"
        >
          <Plus :size="14" />
          保存灵感
        </button>
      </div>
    </div>

    <!-- 灵感卡片列表 -->
    <div v-if="inspiration.items.length" class="fl-insp-list">
      <TransitionGroup name="fl-insp-card" tag="div" class="fl-insp-cards">
        <div
          v-for="item in inspiration.latestItems"
          :key="item.id"
          class="fl-insp-card"
          :class="{
            'is-converted': !!item.convertedTaskId,
            'is-new': justSaved === item.id,
          }"
        >
          <div class="fl-insp-card-body">
            <img
              v-if="item.imagePath"
              :src="getInspirationImageSrc(item.imagePath)"
              alt="灵感附图"
              class="fl-insp-card-image"
              loading="lazy"
            />
            <div v-if="item.content" class="fl-insp-card-text fl-markdown" v-html="renderMarkdown(item.content)" />
            <p v-else class="fl-insp-card-image-only">图片灵感</p>
            <div class="fl-insp-card-meta">
              <span class="fl-insp-time">{{ fmtTime(item.createdAt) }}</span>
              <span v-if="item.convertedTaskId" class="fl-insp-tag fl-insp-tag-done">
                <Check :size="10" /> 已转任务
              </span>
            </div>
          </div>
          <div class="fl-insp-card-actions">
            <button
              v-if="item.content.trim() && !item.convertedTaskId"
              class="fl-insp-btn fl-insp-btn-convert"
              type="button"
              :disabled="inspiration.saving"
              title="转为任务"
              @click="onConvert(item.id)"
            >
              <ArrowRight :size="12" />
            </button>
            <button
              class="fl-insp-btn fl-insp-btn-del"
              type="button"
              title="删除"
              @click="inspiration.remove(item.id)"
            >
              <Trash2 :size="12" />
            </button>
          </div>
        </div>
      </TransitionGroup>

      <div v-if="inspiration.totalCount > 3" class="fl-insp-more">
        <button class="fl-insp-more-btn" type="button" @click="router.push('/inspirations')">
          查看全部 {{ inspiration.totalCount }} 条
          <ArrowRight :size="11" />
        </button>
      </div>
    </div>

    <!-- 空态 -->
    <div v-else class="fl-insp-empty">
      还没有灵感 · 随手记下你的第一个想法 ✨
      <button
        class="fl-insp-empty-link"
        type="button"
        @click="router.push('/inspirations')"
      >
        前往灵感页 →
      </button>
    </div>
  </div>
</template>

<style scoped>
.fl-insp {
  display: flex;
  flex-direction: column;
  gap: var(--sp-3);
}

/* ---------- 标题 ---------- */
.fl-insp-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.fl-insp-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--fs-14);
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
}
.fl-insp-icon {
  color: var(--color-primary);
}
.fl-insp-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: var(--r-pill);
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
  font-size: 11px;
  font-weight: var(--fw-semibold);
}

/* ---------- 输入区 ---------- */
.fl-insp-input-wrap {
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  background: var(--color-bg-elevated);
  overflow: hidden;
  transition: border-color var(--dur-fast) var(--ease-smooth), box-shadow var(--dur-fast) var(--ease-smooth);
  position: relative;
}
.fl-insp-input-wrap::before {
  content: "";
  position: absolute;
  inset: 0 0 auto;
  height: 2px;
  background: linear-gradient(90deg, var(--color-primary), color-mix(in srgb, var(--color-primary) 20%, transparent));
}
.fl-insp-input-wrap:focus-within {
  border-color: color-mix(in srgb, var(--color-primary) 50%, var(--color-border));
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-primary) 10%, transparent);
}
.fl-insp-textarea {
  width: 100%;
  padding: var(--sp-3) var(--sp-3) var(--sp-2);
  border: none;
  outline: none;
  resize: none;
  background: transparent;
  color: var(--color-text-primary);
  font-size: var(--fs-13, 13px);
  line-height: 1.65;
  font-family: var(--font-sans);
}
.fl-insp-textarea::placeholder {
  color: var(--color-text-muted);
}
.fl-insp-input-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sp-2) var(--sp-3);
  border-top: 1px solid var(--color-divider);
}
.fl-insp-image-draft {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 0 var(--sp-3) var(--sp-3);
}
.fl-insp-image-preview {
  width: 88px;
  height: 88px;
  object-fit: cover;
  border-radius: var(--r-sm);
  border: 1px solid var(--color-border);
}
.fl-insp-image-remove {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 10px;
  border-radius: var(--r-pill);
  border: 1px solid var(--color-border);
  background: var(--color-bg);
  color: var(--color-text-secondary);
  cursor: pointer;
}
.fl-insp-image-error {
  margin: 0;
  padding: 0 var(--sp-3) var(--sp-3);
  color: var(--color-danger, #ef4444);
  font-size: 12px;
}
.fl-insp-hint {
  font-size: 11px;
  color: var(--color-text-muted);
}
.fl-insp-save {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: var(--r-sm);
  background: var(--color-primary);
  color: #fff;
  border: none;
  font-size: 12px;
  font-weight: var(--fw-medium);
  cursor: pointer;
  transition: opacity var(--dur-fast) var(--ease-smooth);
}
.fl-insp-save:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.fl-insp-save:not(:disabled):hover {
  opacity: 0.88;
}

/* ---------- 卡片列表 ---------- */
.fl-insp-list {
  display: flex;
  flex-direction: column;
  gap: var(--sp-1);
}
.fl-insp-cards {
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}
.fl-insp-card {
  display: flex;
  align-items: flex-start;
  gap: var(--sp-2);
  padding: var(--sp-3);
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  background: var(--color-bg-elevated);
  transition:
    border-color var(--dur-fast) var(--ease-smooth),
    background var(--dur-fast) var(--ease-smooth),
    transform var(--dur-base) var(--ease-smooth),
    opacity var(--dur-base) var(--ease-smooth);
}
.fl-insp-card.is-converted {
  opacity: 0.6;
  background: var(--color-bg-subtle);
}
.fl-insp-card.is-new {
  border-color: color-mix(in srgb, var(--color-primary) 55%, var(--color-border));
  background: color-mix(in srgb, var(--color-primary) 5%, var(--color-bg-elevated));
}
.fl-insp-card-body {
  flex: 1;
  min-width: 0;
}
.fl-insp-card-image {
  width: 100%;
  max-height: 152px;
  object-fit: cover;
  border-radius: var(--r-sm);
  border: 1px solid var(--color-border);
  margin-bottom: 8px;
}
.fl-insp-card-text {
  font-size: var(--fs-13, 13px);
  color: var(--color-text-primary);
  line-height: 1.6;
  word-break: break-word;
  margin: 0 0 4px;
}
.fl-insp-card-image-only {
  margin: 0 0 4px;
  font-size: 12px;
  color: var(--color-text-secondary);
}
.fl-markdown :deep(p) {
  margin: 0;
}
.fl-markdown :deep(p + p) {
  margin-top: 6px;
}
.fl-markdown :deep(code) {
  padding: 1px 4px;
  border-radius: 6px;
  background: color-mix(in srgb, var(--color-primary) 10%, var(--color-bg));
  font-size: 12px;
}
.fl-insp-card-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.fl-insp-time {
  font-size: 11px;
  color: var(--color-text-muted);
}
.fl-insp-tag {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 1px 6px;
  border-radius: var(--r-pill);
  font-size: 10px;
  font-weight: var(--fw-medium);
}
.fl-insp-tag-done {
  background: color-mix(in srgb, var(--color-success, #22c55e) 15%, transparent);
  color: var(--color-success-text, #16a34a);
}

/* ---------- 操作按钮 ---------- */
.fl-insp-card-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity var(--dur-fast) var(--ease-smooth);
}
.fl-insp-card:hover .fl-insp-card-actions {
  opacity: 1;
}
.fl-insp-btn {
  display: grid;
  place-items: center;
  width: 24px;
  height: 24px;
  border-radius: var(--r-sm);
  border: none;
  cursor: pointer;
  transition: background var(--dur-fast) var(--ease-smooth), color var(--dur-fast) var(--ease-smooth);
}
.fl-insp-btn-convert {
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
}
.fl-insp-btn-convert:hover:not(:disabled) {
  background: var(--color-primary);
  color: #fff;
}
.fl-insp-btn-convert:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.fl-insp-btn-del {
  background: var(--color-bg-hover);
  color: var(--color-text-muted);
}
.fl-insp-btn-del:hover {
  background: color-mix(in srgb, var(--color-danger, #ef4444) 12%, transparent);
  color: var(--color-danger, #ef4444);
}

/* ---------- 底部 ---------- */
.fl-insp-more {
  padding-top: var(--sp-1);
  display: flex;
  justify-content: center;
}
.fl-insp-more-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: var(--r-pill);
  border: 1px solid var(--color-border);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all var(--dur-fast) var(--ease-smooth);
}
.fl-insp-more-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

/* ---------- 空态 ---------- */
.fl-insp-empty {
  font-size: var(--fs-13, 13px);
  color: var(--color-text-muted);
  text-align: center;
  padding: var(--sp-4) 0;
  line-height: 1.7;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--sp-2);
}
.fl-insp-empty-link {
  background: none;
  border: none;
  color: var(--color-primary);
  cursor: pointer;
  font-size: 11px;
  padding: 0;
}
.fl-insp-empty-link:hover { text-decoration: underline; }

/* ---------- 动画 ---------- */
.fl-insp-card-enter-active,
.fl-insp-card-leave-active {
  transition: all var(--dur-base) var(--ease-smooth);
}
.fl-insp-card-enter-from {
  opacity: 0;
  transform: translateY(-8px);
}
.fl-insp-card-leave-to {
  opacity: 0;
  transform: translateX(8px);
}
.fl-insp-card-move {
  transition: transform var(--dur-base) var(--ease-smooth);
}
</style>

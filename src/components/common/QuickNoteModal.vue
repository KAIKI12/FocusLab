<script setup lang="ts">
/**
 * QuickNoteModal · 速记便签 (⌘⇧N)。
 * 同页连续流：输入区在上，AI 三版候选在下。
 */

import { Sparkles, X, Check, Copy, Edit3, Zap, RefreshCw, MessageSquare } from "lucide-vue-next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { computed, nextTick, ref, watch } from "vue";

import { useInspirationImageDraft } from "@/composables/useInspirationImageDraft";
import { useAIStore, type QuickNoteCandidate } from "@/stores/useAIStore";
import { useChatStore } from "@/stores/useChatStore";
import { useInspirationStore } from "@/stores/useInspirationStore";
import { useUIStore } from "@/stores/useUIStore";

const props = withDefaults(defineProps<{
  standalone?: boolean;
  visible: boolean;
}>(), {
  standalone: false,
});
const emit = defineEmits<{
  close: [];
  "create-task": [text: string, quadrant?: string];
  saved: [];
}>();

const ai = useAIStore();
const chat = useChatStore();
const inspiration = useInspirationStore();
const ui = useUIStore();
const appWindow = getCurrentWindow();

const rawText = ref("");
const { imageDraft, imageError, clearImage, handlePasteImage, toUploadPayload } = useInspirationImageDraft();
const textareaEl = ref<HTMLTextAreaElement | null>(null);
const expanded = ref(false);
const candidates = ref<QuickNoteCandidate[]>([]);
const pickedIndex = ref(0);
const aiLoading = ref(false);
const aiError = ref("");
const copiedCandIdx = ref<number | null>(null);
const canSaveRaw = computed(() => !!rawText.value.trim() || !!imageDraft.value);

const candidateIcons = [Check, Edit3, Zap];

watch(() => props.visible, (v) => {
  if (v) {
    rawText.value = ui.quickNotePrefilledText || "";
    ui.quickNotePrefilledText = "";
    expanded.value = false;
    candidates.value = [];
    pickedIndex.value = 0;
    aiError.value = "";
    clearImage();
    nextTick(() => textareaEl.value?.focus());
    if (props.standalone) {
      appWindow.setFocus().catch((err) => {
        console.error("[quick-note] set focus failed", err);
      });
    }
  }
});

function onClose() {
  emit("close");
}

function onCopyCand(idx: number, text: string) {
  navigator.clipboard.writeText(text);
  copiedCandIdx.value = idx;
  setTimeout(() => { if (copiedCandIdx.value === idx) copiedCandIdx.value = null; }, 1500);
}

function finishAfterSave() {
  emit("saved");
  if (!props.standalone) emit("close");
}

async function onSaveRaw() {
  await inspiration.ensureLoaded();
  await inspiration.create(rawText.value, {
    image: await toUploadPayload(),
  });
  clearImage();
  finishAfterSave();
}

async function onAiOptimize() {
  const text = rawText.value.trim();
  if (!text || text.length < 5) return;
  aiLoading.value = true;
  aiError.value = "";
  try {
    candidates.value = await ai.optimizeQuickNote(text);
    pickedIndex.value = 0;
    expanded.value = true;
  } catch {
    aiError.value = "AI 服务暂时不可用，你可以直接保存原文。";
  } finally {
    aiLoading.value = false;
  }
}

function pickCandidate(i: number) {
  pickedIndex.value = i;
}

function onContinueEdit() {
  const c = candidates.value[pickedIndex.value];
  if (c) rawText.value = c.text;
  expanded.value = false;
  candidates.value = [];
  nextTick(() => textareaEl.value?.focus());
}

async function onSaveNote() {
  const c = candidates.value[pickedIndex.value];
  if (!c) return;
  await inspiration.ensureLoaded();
  await inspiration.create(c.text, {
    image: await toUploadPayload(),
  });
  clearImage();
  finishAfterSave();
}

function onCreateTask() {
  const c = candidates.value[pickedIndex.value];
  if (!c) return;
  emit("create-task", c.text, c.quadrant);
}

/// 灵感"研究问题版"专属:保存为灵感,然后打开右侧 ChatPanel 与 AI 深入聊聊。
async function onChatAboutQuestion() {
  const c = candidates.value[pickedIndex.value];
  if (!c || c.style !== "question") return;
  const raw = rawText.value.trim();
  await inspiration.ensureLoaded();
  const created = await inspiration.create(c.text, {
    image: await toUploadPayload(),
  });
  clearImage();
  emit("saved");
  const noteId = created?.id ?? "";
  await chat.createFromInspiration(noteId, raw, c.text);
  if (!ui.showChat) ui.toggleChat();
  emit("close");
}

function collapseAi() {
  expanded.value = false;
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.preventDefault();
    onClose();
  }
}

function onPaste(event: ClipboardEvent) {
  handlePasteImage(event);
}
</script>

<template>
  <Transition name="fl-fade">
    <div
      v-if="visible"
      class="fl-qn-mask"
      :class="{ 'is-standalone': standalone }"
      @click.self="standalone ? undefined : onClose"
      @keydown="onKeydown"
    >
      <section class="fl-qn" :class="{ 'is-standalone': standalone }" role="dialog" aria-modal="true">
        <!-- Head -->
        <div class="fl-qn-head">
          <div class="fl-qn-title-wrap">
            <div class="fl-qn-icon-box">
              <Sparkles :size="18" />
            </div>
            <div>
              <h2 class="fl-qn-title">速记便签</h2>
              <p class="fl-qn-subtitle">有点乱也没关系，先记下来。AI 整理只是一个可选项。</p>
            </div>
          </div>
          <button class="fl-qn-close" type="button" aria-label="关闭" @click="onClose">
            <X :size="14" />
          </button>
        </div>

        <!-- Body -->
        <div class="fl-qn-body">
          <!-- 输入区 -->
          <div class="fl-qn-section-label">
            <span>先随手记下关键点</span>
            <span class="fl-qn-hint-pill">
              <Zap :size="12" /> 低门槛记录
            </span>
          </div>

          <div class="fl-qn-note-box">
            <textarea
              ref="textareaEl"
              v-model="rawText"
              class="fl-qn-textarea"
              spellcheck="false"
              placeholder="支持 Markdown，也可以直接粘贴截图。比如：明天和导师对一下实验设计……"
              @click.stop
              @dblclick.stop
              @mousedown.stop
              @paste="onPaste"
            />
            <div v-if="imageDraft" class="fl-qn-image-draft">
              <img :src="imageDraft.previewUrl" alt="待保存的灵感截图" class="fl-qn-image-preview" />
              <button class="fl-qn-image-remove" type="button" @click="clearImage">
                <X :size="12" />
                移除图片
              </button>
            </div>
            <p v-if="imageError" class="fl-qn-image-error">{{ imageError }}</p>
            <div class="fl-qn-box-foot">
              <span>支持关键词、半句话、Markdown 和截图粘贴。</span>
              <span>{{ rawText.length }} 字</span>
            </div>
          </div>

          <div class="fl-qn-inline-tip">
            <Sparkles :size="14" />
            <span>如果你觉得这条速记已经够用了，可以直接保存；如果想让表达更清楚，再点"AI 整理一下"。</span>
          </div>

          <!-- 操作行 -->
          <div class="fl-qn-input-actions">
            <div class="fl-qn-actions-left">
              <button class="fl-qn-btn fl-qn-btn-ghost" type="button" @click="onClose">取消</button>
              <button
                class="fl-qn-btn fl-qn-btn-secondary"
                type="button"
                :disabled="!canSaveRaw"
                @click="onSaveRaw"
              >直接保存原文</button>
            </div>
            <div class="fl-qn-actions-right">
              <button
                class="fl-qn-btn fl-qn-btn-primary"
                type="button"
                :disabled="!rawText.trim() || rawText.trim().length < 5 || aiLoading"
                @click="onAiOptimize"
              >
                <Sparkles :size="14" />
                {{ aiLoading ? 'AI 整理中…' : expanded ? '已展开 AI 结果' : 'AI 整理一下' }}
              </button>
            </div>
          </div>

          <!-- AI Error -->
          <div v-if="aiError" class="fl-qn-ai-error">
            {{ aiError }}
          </div>

          <!-- AI Section -->
          <div v-if="expanded && candidates.length" class="fl-qn-ai-section">
            <div class="fl-qn-ai-head">
              <div>
                <div class="fl-qn-ai-title">
                  <Sparkles :size="16" /> AI 给了你 3 种整理方式
                </div>
                <p class="fl-qn-ai-sub">保留原始速记在上方，下面直接对比三版表达。</p>
              </div>
              <button
                class="fl-qn-btn fl-qn-btn-secondary fl-qn-btn-sm"
                type="button"
                :disabled="aiLoading"
                @click="onAiOptimize"
              >
                <RefreshCw :size="12" /> 重新生成
              </button>
            </div>

            <!-- Candidate Grid -->
            <div class="fl-qn-candidate-grid">
              <article
                v-for="(c, i) in candidates"
                :key="c.label"
                class="fl-qn-candidate"
                :class="{ 'is-picked': pickedIndex === i }"
                @click="pickCandidate(i)"
              >
                <div class="fl-qn-candidate-head">
                  <div class="fl-qn-candidate-title">
                    <component :is="candidateIcons[i] ?? Check" :size="14" />
                    方案 {{ c.label }} · {{ c.styleName }}
                  </div>
                  <div class="fl-qn-candidate-meta">
                    <span class="fl-qn-chip" :class="c.style === 'task' ? 'chip-brand' : 'chip-neutral'">
                      {{ c.style === 'task' ? '更适合创建任务' : c.style === 'note' ? '更适合先保存笔记' : '更适合快速执行' }}
                    </span>
                    <span v-if="c.quadrant" class="fl-qn-chip chip-q2">
                      {{ c.quadrant === 'important_urgent' ? '重要紧急'
                        : c.quadrant === 'important_not_urgent' ? '重要不紧急'
                        : c.quadrant === 'not_important_urgent' ? '紧急不重要'
                        : '不紧急不重要' }}
                    </span>
                  </div>
                </div>
                <div class="fl-qn-candidate-body">
                  <p>{{ c.text }}</p>
                  <button class="fl-qn-copy-btn" type="button" title="复制" @click.stop="onCopyCand(i, c.text)">
                    <Check v-if="copiedCandIdx === i" :size="12" />
                    <Copy v-else :size="12" />
                  </button>
                </div>
              </article>
            </div>

            <!-- Picked Actions -->
            <div class="fl-qn-picked-actions">
              <span class="fl-qn-picked-label">
                已选 方案 {{ candidates[pickedIndex]?.label }} · {{ candidates[pickedIndex]?.styleName }}
              </span>
              <div class="fl-qn-picked-btns">
                <button class="fl-qn-btn fl-qn-btn-secondary fl-qn-btn-sm" type="button" @click="onContinueEdit">
                  继续编辑这版
                </button>
                <button class="fl-qn-btn fl-qn-btn-secondary fl-qn-btn-sm" type="button" @click="onSaveNote">
                  保存笔记
                </button>
                <button
                  v-if="!standalone && candidates[pickedIndex]?.style === 'question'"
                  class="fl-qn-btn fl-qn-btn-secondary fl-qn-btn-sm fl-qn-btn-chat"
                  type="button"
                  @click="onChatAboutQuestion"
                >
                  <MessageSquare :size="12" /> 深入聊聊 →
                </button>
                <button class="fl-qn-btn fl-qn-btn-primary fl-qn-btn-sm" type="button" @click="onCreateTask">
                  创建任务
                </button>
              </div>
            </div>

            <!-- Footer Actions -->
            <div class="fl-qn-footer-actions">
              <button class="fl-qn-btn fl-qn-btn-ghost" type="button" @click="collapseAi">
                收起 AI 结果
              </button>
            </div>
          </div>

          <!-- Loading Skeleton -->
          <div v-if="aiLoading && !expanded" class="fl-qn-skeleton">
            <div class="fl-qn-skeleton-bar" />
            <div class="fl-qn-skeleton-bar short" />
            <div class="fl-qn-skeleton-bar" />
          </div>
        </div>
      </section>
    </div>
  </Transition>
</template>

<style scoped>
.fl-qn-mask {
  position: fixed;
  inset: 0;
  background: color-mix(in srgb, var(--color-text-primary) 32%, transparent);
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding-top: 8vh;
  z-index: var(--z-modal);
  padding-left: var(--sp-4);
  padding-right: var(--sp-4);
}

.fl-qn-mask.is-standalone {
  background: transparent;
  padding: 0;
}

.fl-qn {
  width: min(680px, 100%);
  max-height: calc(100vh - 12vh);
  overflow-y: auto;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
  box-shadow: var(--shadow-modal);
}

.fl-qn.is-standalone {
  width: 100%;
  max-height: 100vh;
  min-height: 100vh;
  border-radius: 0;
  border: 0;
  box-shadow: none;
}

.fl-qn.is-standalone .fl-qn-head,
.fl-qn.is-standalone .fl-qn-body {
  padding-left: var(--sp-5);
  padding-right: var(--sp-5);
}

/* ---------- Head ---------- */
.fl-qn-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--sp-3);
  padding: var(--sp-4) var(--sp-5);
  border-bottom: 1px solid var(--color-divider);
  background: linear-gradient(135deg, color-mix(in srgb, var(--color-primary) 12%, transparent), transparent 72%);
}
.fl-qn-title-wrap {
  display: flex;
  gap: var(--sp-3);
  align-items: flex-start;
}
.fl-qn-icon-box {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  background: linear-gradient(135deg, var(--color-primary), var(--color-primary-light, var(--color-primary)));
  color: #fff;
  display: grid;
  place-items: center;
  box-shadow: 0 8px 18px color-mix(in srgb, var(--color-primary) 24%, transparent);
  flex-shrink: 0;
}
.fl-qn-title {
  font-size: var(--fs-20, 20px);
  font-weight: var(--fw-semibold);
  margin: 0 0 4px;
}
.fl-qn-subtitle {
  color: var(--color-text-secondary);
  font-size: var(--fs-13, 13px);
  line-height: 1.6;
  max-width: 420px;
  margin: 0;
}
.fl-qn-close {
  color: var(--color-text-muted);
  background: none;
  border: none;
  cursor: pointer;
  padding: var(--sp-1);
  border-radius: var(--r-sm);
}
.fl-qn-close:hover { color: var(--color-text-primary); }

/* ---------- Body ---------- */
.fl-qn-body {
  padding: var(--sp-4) var(--sp-5) var(--sp-5);
}

/* Section Label */
.fl-qn-section-label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-2);
  margin-bottom: 8px;
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
  font-weight: var(--fw-medium);
}
.fl-qn-hint-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 24px;
  padding: 0 10px;
  border-radius: var(--r-pill);
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
  font-weight: var(--fw-medium);
  font-size: var(--fs-12);
}

/* Note Box */
.fl-qn-note-box {
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  background: var(--color-bg-elevated);
  box-shadow: var(--shadow-card);
  position: relative;
  overflow: hidden;
}
.fl-qn-note-box::before {
  content: "";
  position: absolute;
  inset: 0 0 auto 0;
  height: 3px;
  background: linear-gradient(90deg, var(--color-primary), color-mix(in srgb, var(--color-primary) 18%, transparent));
}
.fl-qn-textarea {
  width: 100%;
  min-height: 140px;
  max-height: 240px;
  border: none;
  outline: none;
  resize: vertical;
  background: transparent;
  color: var(--color-text-primary);
  padding: var(--sp-4);
  font-size: var(--fs-14);
  line-height: 1.7;
  font-family: var(--font-sans);
  cursor: text;
  user-select: text;
  -webkit-user-select: text;
}
.fl-qn-textarea::placeholder { color: var(--color-text-muted); }
.fl-qn-image-draft {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 0 var(--sp-4) var(--sp-3);
}
.fl-qn-image-preview {
  width: 96px;
  height: 96px;
  object-fit: cover;
  border-radius: 12px;
  border: 1px solid var(--color-border);
}
.fl-qn-image-remove {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 32px;
  padding: 0 12px;
  border-radius: var(--r-pill);
  border: 1px solid var(--color-border);
  background: var(--color-bg);
  color: var(--color-text-secondary);
  cursor: pointer;
}
.fl-qn-image-error {
  margin: 0;
  padding: 0 var(--sp-4) var(--sp-3);
  color: var(--color-danger, #ef4444);
  font-size: var(--fs-12);
}
.fl-qn-box-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-3);
  padding: 0 var(--sp-4) var(--sp-3);
  color: var(--color-text-muted);
  font-size: var(--fs-12);
}

/* Inline Tip */
.fl-qn-inline-tip {
  margin-top: var(--sp-3);
  padding: 10px 12px;
  border-radius: var(--r-sm);
  background: var(--color-bg-subtle);
  color: var(--color-text-secondary);
  font-size: var(--fs-13, 13px);
  line-height: 1.6;
  display: flex;
  gap: 8px;
  align-items: flex-start;
}

/* Input Actions */
.fl-qn-input-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-3);
  padding-top: var(--sp-4);
  margin-top: var(--sp-4);
  border-top: 1px solid var(--color-divider);
  flex-wrap: wrap;
}
.fl-qn-actions-left,
.fl-qn-actions-right {
  display: flex;
  gap: var(--sp-2);
  flex-wrap: wrap;
}

/* ---------- Buttons ---------- */
.fl-qn-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-radius: var(--r-md);
  font-size: var(--fs-13, 13px);
  font-weight: var(--fw-medium);
  border: 1px solid transparent;
  cursor: pointer;
  transition: all var(--dur-fast) var(--ease-smooth);
  white-space: nowrap;
}
.fl-qn-btn-sm {
  padding: 5px 12px;
  font-size: var(--fs-12);
}
.fl-qn-btn-ghost {
  background: transparent;
  color: var(--color-text-secondary);
  border-color: transparent;
}
.fl-qn-btn-ghost:hover { background: var(--color-bg-hover); }
.fl-qn-btn-secondary {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
  border-color: var(--color-border);
}
.fl-qn-btn-secondary:hover:not(:disabled) { background: var(--color-bg-subtle); }
.fl-qn-btn-primary {
  background: var(--color-primary);
  color: var(--color-text-on-primary, #fff);
}
.fl-qn-btn-primary:hover:not(:disabled) { opacity: 0.9; }
.fl-qn-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

/* ---------- AI Error ---------- */
.fl-qn-ai-error {
  margin-top: var(--sp-3);
  padding: 10px 14px;
  border-radius: var(--r-sm);
  background: color-mix(in srgb, var(--color-danger, #ef4444) 8%, var(--color-bg-elevated));
  color: var(--color-danger, #ef4444);
  font-size: var(--fs-13, 13px);
  border: 1px solid color-mix(in srgb, var(--color-danger, #ef4444) 20%, var(--color-border));
}

/* ---------- AI Section ---------- */
.fl-qn-ai-section {
  margin-top: var(--sp-4);
  padding-top: var(--sp-4);
  border-top: 1px dashed var(--color-divider);
}
.fl-qn-ai-head {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: var(--sp-3);
  margin-bottom: var(--sp-3);
  flex-wrap: wrap;
}
.fl-qn-ai-title {
  font-size: var(--fs-16, 16px);
  font-weight: var(--fw-semibold);
  display: flex;
  align-items: center;
  gap: 8px;
}
.fl-qn-ai-sub {
  margin: 4px 0 0;
  font-size: var(--fs-12);
  color: var(--color-text-secondary);
  line-height: 1.6;
}

/* ---------- Candidate Grid ---------- */
.fl-qn-candidate-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--sp-4);
  align-items: start;
}
.fl-qn-candidate {
  border: 1px solid var(--color-border);
  border-radius: var(--r-md);
  background: var(--color-bg-elevated);
  box-shadow: var(--shadow-card);
  overflow: hidden;
  cursor: pointer;
  transition: border-color var(--dur-base) var(--ease-smooth),
    box-shadow var(--dur-base) var(--ease-smooth),
    transform var(--dur-base) var(--ease-smooth);
}
.fl-qn-candidate:hover {
  border-color: color-mix(in srgb, var(--color-primary) 45%, var(--color-border));
  box-shadow: var(--shadow-float, var(--shadow-card));
  transform: translateY(-1px);
}
.fl-qn-candidate.is-picked {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-primary) 12%, transparent), var(--shadow-card);
}
.fl-qn-candidate-head {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: var(--sp-3) var(--sp-3);
  background: color-mix(in srgb, var(--color-bg-subtle) 82%, transparent);
  border-bottom: 1px solid var(--color-divider);
}
.fl-qn-candidate-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--fs-13, 13px);
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
}
.fl-qn-candidate-meta {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}
.fl-qn-chip {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: var(--r-pill);
  font-size: 11px;
  font-weight: var(--fw-medium);
  white-space: nowrap;
}
.chip-brand {
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
}
.chip-neutral {
  background: var(--color-bg-hover);
  color: var(--color-text-secondary);
}
.chip-q2 {
  background: color-mix(in srgb, var(--color-warning, #f59e0b) 15%, transparent);
  color: color-mix(in srgb, var(--color-warning, #f59e0b) 80%, #000);
}
.fl-qn-candidate-body {
  padding: var(--sp-3);
  position: relative;
}
.fl-qn-copy-btn {
  position: absolute;
  top: var(--sp-2);
  right: var(--sp-2);
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-bg-elevated);
  color: var(--color-text-muted);
  cursor: pointer;
  opacity: 0;
  transition: opacity .15s, color .15s;
}
.fl-qn-candidate:hover .fl-qn-copy-btn { opacity: 1; }
.fl-qn-copy-btn:hover { color: var(--color-primary); border-color: var(--color-primary); }
.fl-qn-candidate-body p {
  font-size: var(--fs-14);
  line-height: 1.75;
  color: var(--color-text-primary);
  margin: 0;
  display: -webkit-box;
  -webkit-line-clamp: 5;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* ---------- Picked Actions ---------- */
.fl-qn-picked-actions {
  margin-top: var(--sp-4);
  padding: var(--sp-3) var(--sp-4);
  border-radius: var(--r-md);
  background: color-mix(in srgb, var(--color-primary) 6%, var(--color-bg-elevated));
  border: 1px solid color-mix(in srgb, var(--color-primary) 25%, var(--color-border));
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-3);
  flex-wrap: wrap;
}
.fl-qn-picked-label {
  font-size: var(--fs-13, 13px);
  font-weight: var(--fw-semibold);
  color: var(--color-primary-dark);
  display: flex;
  align-items: center;
  gap: 6px;
}
.fl-qn-picked-label::before {
  content: "";
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--color-primary);
}
.fl-qn-picked-btns {
  display: flex;
  gap: var(--sp-2);
  flex-wrap: wrap;
}

/* ---------- Footer ---------- */
.fl-qn-footer-actions {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  padding-top: var(--sp-4);
  margin-top: var(--sp-4);
  border-top: 1px solid var(--color-divider);
}

/* ---------- Skeleton ---------- */
.fl-qn-skeleton {
  margin-top: var(--sp-4);
  padding-top: var(--sp-4);
  border-top: 1px dashed var(--color-divider);
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.fl-qn-skeleton-bar {
  height: 14px;
  border-radius: var(--r-sm);
  background: linear-gradient(90deg, var(--color-bg-hover) 25%, var(--color-bg-subtle) 50%, var(--color-bg-hover) 75%);
  background-size: 200% 100%;
  animation: fl-qn-shimmer 1.5s ease-in-out infinite;
}
.fl-qn-skeleton-bar.short { width: 60%; }

@keyframes fl-qn-shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

/* ---------- Transition ---------- */
.fl-fade-enter-active,
.fl-fade-leave-active {
  transition: opacity var(--dur-base) var(--ease-smooth);
}
.fl-fade-enter-from,
.fl-fade-leave-to { opacity: 0; }

/* ---------- Responsive ---------- */
@media (max-width: 720px) {
  .fl-qn-candidate-grid {
    grid-template-columns: 1fr;
  }
}
</style>

<script setup lang="ts">
/**
 * InspirationLinkModal · 灵感"手动关联"面板。
 *
 * 注:虽然命名带 Modal,但实际形态是行内展开面板(inline panel),
 * 与原型 v5 一致。父组件控制开关,子组件负责:
 * - 展示已关联列表(可取消)
 * - 切换 related / contradicts 关系
 * - 搜索 + 选择候选灵感建立新关联
 */

import { ArrowRight, X } from "lucide-vue-next";

import type { InspirationItem } from "@/stores/useInspirationStore";
import type { InspirationLink } from "@/types";

const props = defineProps<{
  /** 当前灵感 id,用来识别 link 中谁是"对端" */
  currentItemId: string;
  /** 已关联列表(双向) */
  links: InspirationLink[];
  /** 候选灵感(已过滤掉自身) */
  candidates: InspirationItem[];
  /** 搜索关键字(双向绑定) */
  query: string;
  /** 当前选中的关系类型 */
  relation: "related" | "contradicts";
  /** 通过对端 id 查内容(由父组件传入) */
  peerContent: (id: string) => string;
}>();

const emit = defineEmits<{
  (e: "update:query", value: string): void;
  (e: "update:relation", value: "related" | "contradicts"): void;
  (e: "link", targetId: string, relation: "related" | "contradicts"): void;
  (e: "unlink", peerId: string): void;
}>();

function peerIdOf(link: InspirationLink): string {
  return link.sourceId === props.currentItemId ? link.targetId : link.sourceId;
}
</script>

<template>
  <div class="fl-link-modal" @click.stop>
    <!-- 已关联列表 -->
    <template v-if="links.length">
      <label class="fl-link-modal-label">已关联（{{ links.length }}）</label>
      <div class="fl-link-modal-linked-list">
        <div
          v-for="link in links"
          :key="link.id"
          class="fl-link-modal-linked-item"
          :class="{ 'is-warn': link.relation === 'contradicts' }"
        >
          <span
            class="fl-link-modal-linked-relation"
            :title="link.relation === 'contradicts' ? '矛盾/纠偏' : '相关'"
          >
            {{ link.relation === "contradicts" ? "⚠" : "🔗" }}
          </span>
          <span class="fl-link-modal-linked-text">{{ peerContent(peerIdOf(link)) }}</span>
          <button
            class="fl-link-modal-linked-unlink"
            type="button"
            title="取消关联"
            @click.stop="emit('unlink', peerIdOf(link))"
          >
            <X :size="12" />
          </button>
        </div>
      </div>
    </template>

    <!-- 添加新关联 -->
    <label class="fl-link-modal-label">添加新关联</label>
    <div class="fl-link-modal-tabs">
      <button
        type="button"
        class="fl-link-modal-tab"
        :class="{ 'is-active': relation === 'related' }"
        @click="emit('update:relation', 'related')"
      >
        🔗 相关
      </button>
      <button
        type="button"
        class="fl-link-modal-tab"
        :class="{ 'is-active': relation === 'contradicts' }"
        @click="emit('update:relation', 'contradicts')"
      >
        ⚠ 矛盾/纠偏
      </button>
    </div>

    <input
      :value="query"
      class="fl-link-modal-input"
      type="search"
      placeholder="搜索要关联的灵感…"
      @input="emit('update:query', ($event.target as HTMLInputElement).value)"
    />

    <div class="fl-link-modal-list">
      <button
        v-for="candidate in candidates"
        :key="candidate.id"
        class="fl-link-modal-item"
        type="button"
        @click="emit('link', candidate.id, relation)"
      >
        <span>{{ candidate.content }}</span>
        <ArrowRight :size="12" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.fl-link-modal {
  margin-top: var(--sp-2);
  padding: var(--sp-2);
  border-radius: var(--r-sm);
  background: var(--color-bg-subtle);
  border: 1px solid var(--color-divider);
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}
.fl-link-modal-label {
  font-size: 11px;
  color: var(--color-text-secondary);
}
.fl-link-modal-linked-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 180px;
  overflow-y: auto;
  padding: 4px 0;
}
.fl-link-modal-linked-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px;
  border-radius: var(--r-sm);
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  font-size: 11.5px;
  color: var(--color-text-primary);
}
.fl-link-modal-linked-item.is-warn {
  border-color: color-mix(in srgb, var(--color-q1, #ef4444) 35%, var(--color-border));
  background: color-mix(in srgb, var(--color-q1, #ef4444) 5%, var(--color-bg));
}
.fl-link-modal-linked-relation {
  flex-shrink: 0;
  font-size: 12px;
}
.fl-link-modal-linked-text {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-text-secondary);
}
.fl-link-modal-linked-unlink {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: var(--r-sm);
  border: none;
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all 100ms;
}
.fl-link-modal-linked-unlink:hover {
  background: color-mix(in srgb, var(--color-q1, #ef4444) 12%, transparent);
  color: var(--color-q1, #ef4444);
}
.fl-link-modal-tabs {
  display: flex;
  gap: 6px;
}
.fl-link-modal-tab {
  flex: 1;
  padding: 5px 8px;
  font-size: 11px;
  border: 1px solid var(--color-border);
  border-radius: var(--r-sm);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 120ms;
}
.fl-link-modal-tab:hover {
  border-color: var(--color-primary);
}
.fl-link-modal-tab.is-active {
  background: color-mix(in srgb, var(--color-primary) 12%, transparent);
  border-color: var(--color-primary);
  color: var(--color-primary);
  font-weight: 600;
}
.fl-link-modal-input {
  border: 1px solid var(--color-border);
  border-radius: var(--r-sm);
  padding: 6px 10px;
  font-size: 12px;
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
}
.fl-link-modal-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.fl-link-modal-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-2);
  padding: 8px 10px;
  border-radius: var(--r-sm);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  font-size: 12px;
  color: var(--color-text-primary);
  text-align: left;
  cursor: pointer;
  transition: all 120ms;
}
.fl-link-modal-item:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}
</style>

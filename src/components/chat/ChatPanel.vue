<script setup lang="ts">
/**
 * ChatPanel · AI 聊天侧面板。
 *
 * 设计:
 * - 三段式布局: 顶栏(56px) / 消息列表(flex) / 输入区(auto-height)
 * - 风格:微交互 + 极简 + 微玻璃,Light/Dark 双适配
 * - 流式: 渐入 + 呼吸光标 + 思考点
 *
 * 注:逻辑层与 store 调用与原版完全一致,仅重构模板与样式。
 */

import { nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import {
  ChevronDown,
  MessageSquarePlus,
  Pin,
  PinOff,
  X,
  ArrowUp,
  Square,
  Copy,
  Check,
  MoreHorizontal,
  Pencil,
  Archive,
  Trash2,
  Bot,
  User,
  Settings,
  Sparkles,
  GripVertical,
  Maximize2,
  Minimize2,
  PanelLeft,
  PanelLeftClose,
} from "lucide-vue-next";
import type { Conversation, ChatMessage } from "@/stores/useChatStore";
import { useChatStore } from "@/stores/useChatStore";
import { useUIStore } from "@/stores/useUIStore";
import { useAIProfileStore } from "@/stores/useAIProfileStore";
import { renderMarkdown } from "@/composables/useMarkdown";

const chat = useChatStore();
const ui = useUIStore();
const aiProfile = useAIProfileStore();

// ---- 本地状态 ----

const inputText = ref("");
const messageListEl = ref<HTMLElement | null>(null);
const showConvDropdown = ref(false);
const showConvMenu = ref<string | null>(null);
const renamingId = ref<string | null>(null);
const renameInput = ref("");
const copiedId = ref<string | null>(null);
const textareaEl = ref<HTMLTextAreaElement | null>(null);
const isResizing = ref(false);
const isExpanded = ref(false);
const showModelDropdown = ref(false);
const selectedModel = ref("");
// 所有 provider 的模型列表（按 provider 分组）
const allProviderModels = ref<{ profileId: string; profileName: string; provider: string; models: string[] }[]>([]);
// 侧边会话列表显隐(localStorage 持久化)
const showSidebar = ref(localStorage.getItem("fl-chat-sidebar") === "true");
watch(showSidebar, (v) => localStorage.setItem("fl-chat-sidebar", String(v)));

// ---- 会话分组 ----

const pinnedConvs = () =>
  chat.conversations.filter((c) => c.pinned && !c.archived);
const recentConvs = () =>
  chat.conversations.filter((c) => !c.pinned && !c.archived);
const archivedConvs = () => chat.conversations.filter((c) => c.archived);

// ---- 活跃 profile 模型信息 ----

const activeProfile = () =>
  aiProfile.chatProfiles.find((p) => p.id === aiProfile.activeChatId) ?? null;

// 从各 profile 的 selectedModels 构建模型列表
function rebuildModelList() {
  const profiles = aiProfile.chatProfiles;
  if (!profiles.length) { allProviderModels.value = []; return; }
  allProviderModels.value = profiles
    .filter((p) => p.selectedModels.length > 0)
    .map((p) => ({
      profileId: p.id,
      profileName: p.name,
      provider: p.provider,
      models: p.selectedModels,
    }));
}

// ---- 发送 / 中断 ----

async function handleSend() {
  const text = inputText.value.trim();
  if (!text || chat.isSending) return;
  inputText.value = "";
  autoResize();
  if (!chat.currentId) {
    await chat.createConversation();
  }
  await chat.sendMessage(text);
  scrollToBottom();
}

function handleNewline() {
  // v-model 自动处理换行
}

function handleAbort() {
  chat.abortStreaming();
}

// ---- 消息列表滚动 ----

async function scrollToBottom() {
  await nextTick();
  messageListEl.value?.scrollTo({
    top: messageListEl.value.scrollHeight,
    behavior: "smooth",
  });
}

// ---- textarea 自适应高度 ----

function autoResize() {
  const el = textareaEl.value;
  if (!el) return;
  el.style.height = "auto";
  const maxH = 160;
  el.style.height = Math.min(el.scrollHeight, maxH) + "px";
}

// ---- 会话操作 ----

async function selectConv(id: string) {
  showConvDropdown.value = false;
  await chat.selectConversation(id);
  scrollToBottom();
}

async function newConversation() {
  showConvDropdown.value = false;
  await chat.createConversation();
  scrollToBottom();
}

function startRename(c: Conversation) {
  renamingId.value = c.id;
  renameInput.value = c.title;
  showConvMenu.value = null;
}

async function confirmRename(id: string) {
  const t = renameInput.value.trim();
  if (t) await chat.renameConversation(id, t);
  renamingId.value = null;
}

async function handlePin(id: string, pinned: boolean) {
  await chat.pinConversation(id, pinned);
  showConvMenu.value = null;
}

async function handleArchive(id: string, archived: boolean) {
  await chat.archiveConversation(id, archived);
  showConvMenu.value = null;
}

async function handleDelete(id: string) {
  await chat.deleteConversation(id);
  showConvMenu.value = null;
}

// ---- 模型切换 ----

async function pickModel(model: string) {
  selectedModel.value = model;
  if (chat.currentId) {
    await chat.setConversationModel(chat.currentId, model);
  }
  showModelDropdown.value = false;
}

// ---- 复制消息 ----

async function copyMessage(msg: ChatMessage) {
  try {
    await navigator.clipboard.writeText(msg.content);
    copiedId.value = msg.id;
    setTimeout(() => {
      copiedId.value = null;
    }, 1500);
  } catch {
    // 静默失败
  }
}

// ---- 时间格式化 ----

function formatMsgTime(iso: string | null | undefined): string {
  if (!iso) return "";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "";
  const now = new Date();
  const sameDay = d.toDateString() === now.toDateString();
  const yest = new Date(now); yest.setDate(yest.getDate() - 1);
  const isYesterday = d.toDateString() === yest.toDateString();
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  if (sameDay) return `今天 ${hh}:${mm}`;
  if (isYesterday) return `昨天 ${hh}:${mm}`;
  const yyyy = d.getFullYear();
  const mo = String(d.getMonth() + 1).padStart(2, "0");
  const da = String(d.getDate()).padStart(2, "0");
  return `${yyyy}-${mo}-${da} ${hh}:${mm}`;
}

// ---- 点击外部关闭下拉 ----

function onDocumentClick(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (!target.closest(".fl-chat-dropdown-trigger")) {
    showConvDropdown.value = false;
  }
  if (!target.closest(".fl-conv-menu-wrap")) {
    showConvMenu.value = null;
  }
  if (!target.closest(".fl-model-dropdown-wrap")) {
    showModelDropdown.value = false;
  }
}

// ---- 重试 ----

async function retryMessage(msg: ChatMessage) {
  if (msg.role !== "user") return;
  if (!chat.currentId) return;
  await chat.sendMessage(msg.content);
  scrollToBottom();
}

// ---- 拖拽调整宽度(从左边缘) ----

function startResize(e: MouseEvent) {
  e.preventDefault();
  isResizing.value = true;
  const startX = e.clientX;
  const startWidth = ui.chatPanelWidth;

  function onMove(ev: MouseEvent) {
    const dx = startX - ev.clientX;
    const w = Math.min(720, Math.max(320, startWidth + dx));
    ui.chatPanelWidth = w;
  }
  function onUp() {
    isResizing.value = false;
    document.removeEventListener("mousemove", onMove);
    document.removeEventListener("mouseup", onUp);
  }
  document.addEventListener("mousemove", onMove);
  document.addEventListener("mouseup", onUp);
}

// ---- 生命周期 ----

onMounted(async () => {
  try {
    await Promise.all([chat.loadConversations(), chat.checkConfigured()]);
  } catch (e) {
    console.warn("[chat] load conversations failed", e);
  }
  try {
    await aiProfile.loadAll();
  } catch (e) {
    console.warn("[chat] load AI profiles failed", e);
  }
  try {
    rebuildModelList();
    const p = activeProfile();
    if (p) selectedModel.value = p.modelFast;
  } catch (e) {
    console.warn("[chat] rebuild model list failed", e);
  }
  document.addEventListener("click", onDocumentClick);
});

onBeforeUnmount(() => {
  document.removeEventListener("click", onDocumentClick);
});

watch(
  () => chat.currentId,
  () => scrollToBottom(),
);

// 面板打开后补滚（处理先发消息再开面板的场景）
watch(
  () => ui.showChat,
  (visible) => {
    if (visible && chat.messages.length) {
      nextTick(() => scrollToBottom());
    }
  },
);

watch(
  () => chat.prefillMessage,
  (val) => {
    if (val) {
      inputText.value = val;
      chat.prefillMessage = "";
      autoResize();
      textareaEl.value?.focus();
    }
  },
);

watch(
  () => chat.streamingContent.length,
  () => scrollToBottom(),
);

// 进入全屏自动展开侧边会话列表(用户体验更接近完整聊天页)
watch(isExpanded, (v) => {
  if (v) showSidebar.value = true;
});

// profile 变化时刷新模型列表
watch(() => aiProfile.chatProfiles, () => rebuildModelList(), { deep: true });
</script>

<template>
  <!-- 浮动触发按钮(面板关闭时) -->
  <Transition name="fl-fab">
    <button
      v-if="!ui.showChat"
      class="fl-chat-fab"
      aria-label="打开 AI 聊天"
      title="AI 聊天"
      @click="ui.showChat = true"
    >
      <Sparkles :size="18" />
    </button>
  </Transition>

  <!-- 聊天面板(浮动抽屉) -->
  <Transition name="fl-drawer">
    <aside
      v-if="ui.showChat"
      class="fl-chat-panel"
      :class="{ 'is-resizing': isResizing, 'is-expanded': isExpanded }"
      :style="isExpanded ? {} : { width: ui.chatPanelWidth + 'px' }"
      aria-label="AI 聊天面板"
    >
    <!-- 左边缘拖拽手柄 -->
    <div
      class="fl-chat-resize"
      role="separator"
      aria-label="拖动调整聊天面板宽度"
      @mousedown="startResize"
    >
      <GripVertical :size="10" class="fl-chat-resize-icon" />
    </div>

    <!-- ====== 顶栏 ====== -->
    <header class="fl-chat-topbar">
      <button
        class="fl-topbar-icon fl-topbar-sidebar-toggle"
        :class="{ 'is-active': showSidebar }"
        :aria-label="showSidebar ? '隐藏会话列表' : '显示会话列表'"
        :title="showSidebar ? '隐藏会话列表' : '显示会话列表'"
        @click="showSidebar = !showSidebar"
      >
        <component :is="showSidebar ? PanelLeftClose : PanelLeft" :size="14" />
      </button>

      <button
        class="fl-chat-topbar-title fl-chat-dropdown-trigger"
        :aria-expanded="showConvDropdown"
        aria-haspopup="listbox"
        @click="showConvDropdown = !showConvDropdown"
      >
        <span class="fl-topbar-title-icon" aria-hidden="true">
          <Sparkles :size="12" />
        </span>
        <span class="fl-topbar-title-text">
          {{ chat.currentConversation?.title || "AI 助手" }}
        </span>
        <ChevronDown
          :size="14"
          class="fl-topbar-chevron"
          :class="{ 'is-open': showConvDropdown }"
        />
      </button>

      <div class="fl-topbar-actions">
        <span v-if="selectedModel" class="fl-topbar-model-label">{{ selectedModel }}</span>

        <button
          class="fl-topbar-icon"
          :aria-label="isExpanded ? '退出全屏' : '全屏展开'"
          :title="isExpanded ? '退出全屏' : '全屏'"
          @click="isExpanded = !isExpanded"
        >
          <component :is="isExpanded ? Minimize2 : Maximize2" :size="14" />
        </button>
        <button
          class="fl-topbar-icon"
          aria-label="关闭聊天面板"
          title="关闭"
          @click="ui.showChat = false"
        >
          <X :size="15" />
        </button>
      </div>
    </header>

    <div class="fl-chat-body">
      <!-- ====== 侧边会话列表 ====== -->
      <Transition name="fl-sidebar">
        <aside v-if="showSidebar" class="fl-chat-sidebar" aria-label="会话列表">
          <button class="fl-sidebar-new" @click="newConversation">
            <MessageSquarePlus :size="13" />
            <span>新对话</span>
          </button>

          <div class="fl-sidebar-list">
            <template v-if="pinnedConvs().length">
              <div class="fl-sidebar-group">
                <Pin :size="10" />
                <span>已固定</span>
              </div>
              <button
                v-for="c in pinnedConvs()"
                :key="c.id"
                class="fl-sidebar-item"
                :class="{ 'is-active': c.id === chat.currentId }"
                @click="selectConv(c.id)"
              >
                <span class="fl-sidebar-item-main">
                  <span class="fl-sidebar-item-text">{{ c.title }}</span>
                  <span class="fl-sidebar-item-time">{{ formatMsgTime(c.updatedAt) }}</span>
                </span>
                <span class="fl-conv-menu-wrap" @click.stop>
                  <button
                    class="fl-conv-menu-trigger"
                    aria-label="会话操作"
                    @click.stop="showConvMenu = showConvMenu === c.id ? null : c.id"
                  >
                    <MoreHorizontal :size="13" />
                  </button>
                  <div
                    v-if="showConvMenu === c.id"
                    class="fl-conv-context-menu"
                    @click.stop
                  >
                    <button @click="startRename(c)">
                      <Pencil :size="12" /><span>重命名</span>
                    </button>
                    <button @click="handlePin(c.id, false)">
                      <PinOff :size="12" /><span>取消固定</span>
                    </button>
                    <button @click="handleArchive(c.id, true)">
                      <Archive :size="12" /><span>归档</span>
                    </button>
                    <button class="fl-danger" @click="handleDelete(c.id)">
                      <Trash2 :size="12" /><span>删除</span>
                    </button>
                  </div>
                </span>
              </button>
            </template>

            <template v-if="recentConvs().length">
              <div class="fl-sidebar-group"><span>最近</span></div>
              <button
                v-for="c in recentConvs()"
                :key="c.id"
                class="fl-sidebar-item"
                :class="{ 'is-active': c.id === chat.currentId }"
                @click="selectConv(c.id)"
              >
                <span class="fl-sidebar-item-main">
                  <span class="fl-sidebar-item-text">{{ c.title }}</span>
                  <span class="fl-sidebar-item-time">{{ formatMsgTime(c.updatedAt) }}</span>
                </span>
                <span class="fl-conv-menu-wrap" @click.stop>
                  <button
                    class="fl-conv-menu-trigger"
                    aria-label="会话操作"
                    @click.stop="showConvMenu = showConvMenu === c.id ? null : c.id"
                  >
                    <MoreHorizontal :size="13" />
                  </button>
                  <div
                    v-if="showConvMenu === c.id"
                    class="fl-conv-context-menu"
                    @click.stop
                  >
                    <button @click="startRename(c)">
                      <Pencil :size="12" /><span>重命名</span>
                    </button>
                    <button @click="handlePin(c.id, true)">
                      <Pin :size="12" /><span>固定</span>
                    </button>
                    <button @click="handleArchive(c.id, true)">
                      <Archive :size="12" /><span>归档</span>
                    </button>
                    <button class="fl-danger" @click="handleDelete(c.id)">
                      <Trash2 :size="12" /><span>删除</span>
                    </button>
                  </div>
                </span>
              </button>
            </template>

            <template v-if="archivedConvs().length">
              <details class="fl-sidebar-archived">
                <summary class="fl-sidebar-group">
                  <Archive :size="10" />
                  <span>已归档 ({{ archivedConvs().length }})</span>
                </summary>
                <button
                  v-for="c in archivedConvs()"
                  :key="c.id"
                  class="fl-sidebar-item is-archived"
                  :class="{ 'is-active': c.id === chat.currentId }"
                  @click="selectConv(c.id)"
                >
                  <span class="fl-sidebar-item-main">
                    <span class="fl-sidebar-item-text">{{ c.title }}</span>
                    <span class="fl-sidebar-item-time">{{ formatMsgTime(c.updatedAt) }}</span>
                  </span>
                  <span class="fl-conv-menu-wrap" @click.stop>
                    <button
                      class="fl-conv-menu-trigger"
                      aria-label="会话操作"
                      @click.stop="showConvMenu = showConvMenu === c.id ? null : c.id"
                    >
                      <MoreHorizontal :size="13" />
                    </button>
                    <div
                      v-if="showConvMenu === c.id"
                      class="fl-conv-context-menu"
                      @click.stop
                    >
                      <button @click="handleArchive(c.id, false)">
                        <Archive :size="12" /><span>取消归档</span>
                      </button>
                      <button class="fl-danger" @click="handleDelete(c.id)">
                        <Trash2 :size="12" /><span>删除</span>
                      </button>
                    </div>
                  </span>
                </button>
              </details>
            </template>

            <div
              v-if="!pinnedConvs().length && !recentConvs().length && !archivedConvs().length"
              class="fl-sidebar-empty"
            >
              暂无会话
            </div>
          </div>
        </aside>
      </Transition>

      <div class="fl-chat-main">

    <!-- ====== 会话下拉 ====== -->
    <Transition name="fl-dropdown">
      <div
        v-if="showConvDropdown"
        class="fl-chat-dropdown fl-chat-dropdown-trigger"
        role="listbox"
      >
        <template v-if="pinnedConvs().length">
          <div class="fl-dropdown-label">
            <Pin :size="10" />
            <span>已固定</span>
          </div>
          <button
            v-for="c in pinnedConvs()"
            :key="c.id"
            class="fl-dropdown-item"
            :class="{ 'is-active': c.id === chat.currentId }"
            role="option"
            :aria-selected="c.id === chat.currentId"
            @click="selectConv(c.id)"
          >
            <span class="fl-dropdown-item-main">
              <span class="fl-dropdown-item-text">{{ c.title }}</span>
              <span class="fl-dropdown-item-time">{{ formatMsgTime(c.updatedAt) }}</span>
            </span>
            <span class="fl-conv-menu-wrap">
              <button
                class="fl-conv-menu-trigger"
                aria-label="会话操作菜单"
                @click.stop="showConvMenu = c.id"
              >
                <MoreHorizontal :size="13" />
              </button>
              <div
                v-if="showConvMenu === c.id"
                class="fl-conv-context-menu"
                @click.stop
              >
                <button @click="startRename(c)">
                  <Pencil :size="12" /><span>重命名</span>
                </button>
                <button @click="handlePin(c.id, false)">
                  <PinOff :size="12" /><span>取消固定</span>
                </button>
                <button @click="handleArchive(c.id, true)">
                  <Archive :size="12" /><span>归档</span>
                </button>
                <button class="fl-danger" @click="handleDelete(c.id)">
                  <Trash2 :size="12" /><span>删除</span>
                </button>
              </div>
            </span>
          </button>
        </template>

        <template v-if="recentConvs().length">
          <div class="fl-dropdown-label"><span>最近</span></div>
          <button
            v-for="c in recentConvs()"
            :key="c.id"
            class="fl-dropdown-item"
            :class="{ 'is-active': c.id === chat.currentId }"
            role="option"
            :aria-selected="c.id === chat.currentId"
            @click="selectConv(c.id)"
          >
            <span class="fl-dropdown-item-main">
              <span class="fl-dropdown-item-text">{{ c.title }}</span>
              <span class="fl-dropdown-item-time">{{ formatMsgTime(c.updatedAt) }}</span>
            </span>
            <span class="fl-conv-menu-wrap">
              <button
                class="fl-conv-menu-trigger"
                aria-label="会话操作菜单"
                @click.stop="showConvMenu = c.id"
              >
                <MoreHorizontal :size="13" />
              </button>
              <div
                v-if="showConvMenu === c.id"
                class="fl-conv-context-menu"
                @click.stop
              >
                <button @click="startRename(c)">
                  <Pencil :size="12" /><span>重命名</span>
                </button>
                <button @click="handlePin(c.id, true)">
                  <Pin :size="12" /><span>固定</span>
                </button>
                <button @click="handleArchive(c.id, true)">
                  <Archive :size="12" /><span>归档</span>
                </button>
                <button class="fl-danger" @click="handleDelete(c.id)">
                  <Trash2 :size="12" /><span>删除</span>
                </button>
              </div>
            </span>
          </button>
        </template>

        <template v-if="archivedConvs().length">
          <div class="fl-dropdown-label">
            <Archive :size="10" />
            <span>已归档</span>
          </div>
          <button
            v-for="c in archivedConvs()"
            :key="c.id"
            class="fl-dropdown-item is-archived"
            :class="{ 'is-active': c.id === chat.currentId }"
            role="option"
            :aria-selected="c.id === chat.currentId"
            @click="selectConv(c.id)"
          >
            <span class="fl-dropdown-item-main">
              <span class="fl-dropdown-item-text">{{ c.title }}</span>
              <span class="fl-dropdown-item-time">{{ formatMsgTime(c.updatedAt) }}</span>
            </span>
            <span class="fl-conv-menu-wrap">
              <button
                class="fl-conv-menu-trigger"
                aria-label="会话操作菜单"
                @click.stop="showConvMenu = c.id"
              >
                <MoreHorizontal :size="13" />
              </button>
              <div
                v-if="showConvMenu === c.id"
                class="fl-conv-context-menu"
                @click.stop
              >
                <button @click="handleArchive(c.id, false)">
                  <Archive :size="12" /><span>取消归档</span>
                </button>
                <button class="fl-danger" @click="handleDelete(c.id)">
                  <Trash2 :size="12" /><span>删除</span>
                </button>
              </div>
            </span>
          </button>
        </template>

        <button class="fl-dropdown-new" @click="newConversation">
          <MessageSquarePlus :size="14" />
          <span>新对话</span>
        </button>
      </div>
    </Transition>

    <!-- ====== 重命名弹层 ====== -->
    <Transition name="fl-modal">
      <div
        v-if="renamingId"
        class="fl-rename-overlay"
        @click.self="renamingId = null"
      >
        <div class="fl-rename-box" role="dialog" aria-modal="true">
          <label class="fl-rename-label">重命名会话</label>
          <input
            v-model="renameInput"
            class="fl-rename-input"
            placeholder="输入新名称"
            autofocus
            @keydown.enter="confirmRename(renamingId!)"
            @keydown.escape="renamingId = null"
          />
          <div class="fl-rename-actions">
            <button class="fl-rename-cancel" @click="renamingId = null">
              取消
            </button>
            <button class="fl-rename-confirm" @click="confirmRename(renamingId!)">
              确认
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- ====== 消息列表 ====== -->
    <div ref="messageListEl" class="fl-chat-messages">
      <!-- 未配置 -->
      <div v-if="!chat.aiConfigured" class="fl-chat-empty">
        <div class="fl-empty-icon-wrap">
          <Settings :size="24" />
        </div>
        <p class="fl-empty-title">AI 尚未配置</p>
        <p class="fl-empty-desc">前往设置添加 API Key 与模型</p>
        <router-link to="/settings" class="fl-empty-link">
          打开设置 →
        </router-link>
      </div>

      <!-- 无会话/空会话 -->
      <div
        v-else-if="!chat.currentId || chat.messages.length === 0"
        class="fl-chat-empty"
      >
        <div class="fl-empty-icon-wrap">
          <Sparkles :size="22" />
        </div>
        <p class="fl-empty-title">
          {{ chat.currentId ? "开始一段新对话" : "选择或创建一个对话" }}
        </p>
        <p class="fl-empty-desc">
          {{
            chat.currentId
              ? "在下方输入想聊的内容,按 Enter 发送"
              : "顶部点击会话标题切换,或新建对话"
          }}
        </p>
      </div>

      <!-- 消息 -->
      <template v-else>
        <div v-for="msg in chat.messages" :key="msg.id" class="fl-msg-wrap">
          <div class="fl-msg-time">{{ formatMsgTime(msg.createdAt) }}</div>
          <div
            class="fl-msg fl-msg-fadein"
            :class="[
              'is-' + msg.role,
              {
                'is-streaming': chat.streamingMessageId === msg.id,
                'is-error': msg.status === 'error',
                'is-aborted': msg.status === 'aborted',
              },
            ]"
          >
          <!-- AI 消息靠左：头像在左 -->
          <template v-if="msg.role === 'assistant'">
            <div class="fl-msg-avatar" aria-hidden="true">
              <Bot :size="13" />
            </div>
            <div class="fl-msg-body">
              <div class="fl-msg-meta">
                <span class="fl-msg-role">助手</span>
                <span v-if="msg.model" class="fl-msg-model">{{ msg.model }}</span>
              </div>
              <div class="fl-msg-content">
                <template v-if="chat.streamingMessageId === msg.id">
                  <span
                    v-if="!chat.streamingContent"
                    class="fl-thinking"
                    aria-label="正在思考"
                  >
                    <span></span><span></span><span></span>
                  </span>
                  <template v-else>
                    <div
                      class="fl-msg-md"
                      v-html="renderMarkdown(chat.streamingContent)"
                    />
                    <span class="fl-cursor" aria-hidden="true" />
                  </template>
                </template>
                <template v-else>
                  <div
                    class="fl-msg-md"
                    v-html="renderMarkdown(msg.content)"
                  />
                  <span
                    v-if="msg.status === 'aborted'"
                    class="fl-aborted-tag"
                    title="已中止"
                  >
                    · 已中止
                  </span>
                </template>
              </div>

              <div v-if="msg.status === 'error'" class="fl-msg-error">
                <span class="fl-msg-error-text">
                  {{ msg.errorMessage || "发送失败" }}
                </span>
                <button class="fl-retry-btn" @click="retryMessage(msg)">
                  重试
                </button>
              </div>
            </div>
            <button
              class="fl-msg-action"
              :class="{ 'is-copied': copiedId === msg.id }"
              :aria-label="copiedId === msg.id ? '已复制' : '复制消息'"
              :title="copiedId === msg.id ? '已复制' : '复制'"
              @click="copyMessage(msg)"
            >
              <component :is="copiedId === msg.id ? Check : Copy" :size="12" />
            </button>
          </template>

          <!-- 用户消息靠右：内容在右，头像在最右 -->
          <template v-else>
            <div class="fl-msg-body is-user-body">
              <div class="fl-msg-meta">
                <span class="fl-msg-role">你</span>
                <span v-if="msg.model" class="fl-msg-model">{{ msg.model }}</span>
              </div>
              <div class="fl-msg-content">
                <span class="fl-msg-text">{{ msg.content }}</span>
                <span
                  v-if="msg.status === 'aborted'"
                  class="fl-aborted-tag"
                  title="已中止"
                >
                  · 已中止
                </span>
              </div>

              <div v-if="msg.status === 'error'" class="fl-msg-error">
                <span class="fl-msg-error-text">
                  {{ msg.errorMessage || "发送失败" }}
                </span>
                <button class="fl-retry-btn" @click="retryMessage(msg)">
                  重试
                </button>
              </div>
            </div>
            <div class="fl-msg-avatar" aria-hidden="true">
              <User :size="13" />
            </div>
          </template>
          </div>
        </div>
      </template>
    </div>

    <!-- ====== 输入区 ====== -->
    <div class="fl-chat-input-wrap">
      <div
        class="fl-chat-input"
        :class="{ 'is-disabled': !chat.aiConfigured }"
      >
        <textarea
          ref="textareaEl"
          v-model="inputText"
          class="fl-chat-textarea"
          :placeholder="chat.aiConfigured ? '输入消息…' : 'AI 未配置'"
          :disabled="!chat.aiConfigured"
          rows="1"
          @input="autoResize"
          @keydown.enter.exact.prevent="handleSend"
          @keydown.shift.enter.exact="handleNewline"
        />
        <div class="fl-input-actions">
          <div class="fl-model-dropdown-wrap">
            <button
              class="fl-model-pill"
              :title="selectedModel || '选择模型'"
              @click.stop="showModelDropdown = !showModelDropdown"
            >
              <span class="fl-model-pill-text">{{ selectedModel || '选择模型' }}</span>
              <ChevronDown :size="10" class="fl-model-pill-arrow" :class="{ 'is-open': showModelDropdown }" />
            </button>
            <Transition name="fl-dropdown-up">
              <div v-if="showModelDropdown" class="fl-model-dropdown-input" @click.stop>
                <template v-for="group in allProviderModels" :key="group.profileId">
                  <div class="fl-model-group-label">{{ group.profileName }}</div>
                  <button
                    v-for="m in group.models"
                    :key="group.profileId + ':' + m"
                    class="fl-model-dropdown-item"
                    :class="{ 'is-active': selectedModel === m }"
                    @click="pickModel(m)"
                  >
                    <span>{{ m }}</span>
                    <Check v-if="selectedModel === m" :size="12" class="fl-model-check" />
                  </button>
                </template>
                <div v-if="!allProviderModels.length" class="fl-model-empty">未获取到模型列表</div>
              </div>
            </Transition>
          </div>
          <button
            v-if="chat.isSending"
            class="fl-send-btn is-abort"
            aria-label="停止生成"
            title="停止生成"
            @click="handleAbort"
          >
            <Square :size="13" />
          </button>
          <button
            v-else
            class="fl-send-btn"
            :disabled="!inputText.trim() || !chat.aiConfigured"
            aria-label="发送消息"
            title="发送 (Enter)"
            @click="handleSend"
          >
            <ArrowUp :size="14" />
          </button>
        </div>
      </div>
      <p v-if="chat.currentConversation?.originType === 'inspiration'" class="fl-chat-hint">
        来自灵感「研究问题版」· AI 作为研究伙伴回答
      </p>
    </div>
      </div><!-- /fl-chat-main -->
    </div><!-- /fl-chat-body -->
  </aside>
  </Transition>
</template>

<style scoped>
/* ====== 设计 token (面板内私有) ====== */
.fl-chat-panel {
  --chat-radius: 14px;
  --chat-radius-sm: 10px;
  --chat-bubble-bg: color-mix(in srgb, var(--color-primary) 8%, transparent);
  --chat-bubble-border: color-mix(in srgb, var(--color-primary) 18%, transparent);
  --chat-glass: color-mix(in srgb, var(--color-bg-elevated) 92%, transparent);
  --chat-divider: color-mix(in srgb, var(--color-border) 70%, transparent);
  --chat-shadow-sm: 0 1px 2px color-mix(in srgb, black 4%, transparent),
    0 2px 8px color-mix(in srgb, black 6%, transparent);
  --chat-shadow-md: 0 4px 16px color-mix(in srgb, black 8%, transparent),
    0 12px 32px color-mix(in srgb, black 6%, transparent);

  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  z-index: 40;
  display: flex;
  flex-direction: column;
  background: var(--chat-glass);
  backdrop-filter: blur(12px) saturate(120%);
  -webkit-backdrop-filter: blur(12px) saturate(120%);
  border-left: 1px solid var(--color-border);
  box-shadow: -4px 0 24px color-mix(in srgb, black 10%, transparent);
  transition: left 280ms cubic-bezier(0.16, 1, 0.3, 1),
    width 280ms cubic-bezier(0.16, 1, 0.3, 1),
    box-shadow 280ms var(--ease-smooth);
  font-family: var(--font-sans);
  font-feature-settings: "ss01", "cv11", "tnum";
  user-select: none;
}

.fl-chat-panel.is-resizing {
  cursor: ew-resize;
  user-select: none;
}

/* ====== 全屏展开态 ====== */
.fl-chat-panel.is-expanded {
  left: 0;
  width: 100% !important;
  border-left: none;
  box-shadow: none;
  border-radius: 0;
}

.fl-chat-panel.is-expanded .fl-chat-resize {
  display: none;
}

/* ====== 拖拽手柄 (左边缘) ====== */
.fl-chat-resize {
  position: absolute;
  top: 0;
  bottom: 0;
  left: -3px;
  width: 6px;
  cursor: ew-resize;
  z-index: 5;
  display: grid;
  place-items: center;
  opacity: 0;
  transition: opacity 100ms var(--ease-smooth);
}

.fl-chat-resize:hover,
.fl-chat-panel.is-resizing .fl-chat-resize {
  opacity: 1;
}

.fl-chat-resize::before {
  content: "";
  position: absolute;
  inset: 0 2px;
  background: color-mix(in srgb, var(--color-primary) 30%, transparent);
  border-radius: var(--r-pill);
}

.fl-chat-resize-icon {
  position: relative;
  color: var(--color-primary);
  z-index: 1;
}

/* ====== 顶栏 ====== */
.fl-chat-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sp-2);
  height: 56px;
  padding: 0 var(--sp-3) 0 var(--sp-4);
  border-bottom: 1px solid var(--chat-divider);
  flex-shrink: 0;
  user-select: none;
}

.fl-chat-topbar-title {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
  flex: 1;
  min-width: 0;
  background: none;
  border: 1px solid transparent;
  color: var(--color-text-primary);
  font-size: var(--fs-14);
  font-weight: var(--fw-medium);
  letter-spacing: -0.01em;
  cursor: pointer;
  padding: 6px 10px 6px 6px;
  border-radius: var(--chat-radius-sm);
  transition:
    background 100ms var(--ease-smooth),
    border-color 100ms var(--ease-smooth);
}

.fl-chat-topbar-title:hover {
  background: var(--color-bg-hover);
  border-color: var(--chat-divider);
}

.fl-topbar-title-icon {
  display: grid;
  place-items: center;
  width: 22px;
  height: 22px;
  border-radius: 7px;
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--color-primary) 24%, transparent),
    color-mix(in srgb, var(--color-primary) 6%, transparent)
  );
  color: var(--color-primary);
  flex-shrink: 0;
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-primary) 14%, transparent);
}

.fl-topbar-title-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fl-topbar-chevron {
  flex-shrink: 0;
  color: var(--color-text-muted);
  transition: transform 150ms var(--ease-smooth);
}

.fl-topbar-chevron.is-open {
  transform: rotate(180deg);
}

/* ====== 模型切换 (segmented) ====== */
.fl-topbar-actions {
  display: flex;
  align-items: center;
  gap: var(--sp-1);
  flex-shrink: 0;
}

.fl-topbar-model-label {
  font-size: 11px;
  color: var(--color-text-muted);
  max-width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fl-model-dropdown-wrap {
  position: relative;
}

.fl-model-pill {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  border: 1px solid transparent;
  border-radius: var(--r-pill);
  background: transparent;
  color: var(--color-text-muted);
  font-size: 11px;
  cursor: pointer;
  transition: all 120ms var(--ease-smooth);
  max-width: 180px;
}

.fl-model-pill:hover {
  background: color-mix(in srgb, var(--color-primary) 8%, transparent);
  color: var(--color-text-secondary);
}

.fl-model-pill-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 120px;
}

.fl-model-pill-arrow {
  transition: transform 150ms var(--ease-smooth);
  opacity: 0.6;
}
.fl-model-pill-arrow.is-open { transform: rotate(180deg); }

.fl-model-dropdown-input {
  position: absolute;
  bottom: calc(100% + 6px);
  right: 0;
  width: 260px;
  max-height: 320px;
  overflow-y: auto;
  background: var(--color-bg-elevated);
  border: 1px solid var(--chat-divider);
  border-radius: var(--r-md);
  box-shadow: 0 8px 24px color-mix(in srgb, black 16%, transparent);
  z-index: 50;
  padding: var(--sp-1);
}

.fl-model-group-label {
  font-size: 10px;
  font-weight: var(--fw-semibold);
  color: var(--color-text-muted);
  padding: 6px 10px 3px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.fl-model-dropdown-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 6px 10px;
  border: none;
  border-radius: var(--r-sm);
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 12px;
  text-align: left;
  cursor: pointer;
  transition: background 100ms;
}

.fl-model-dropdown-item:hover {
  background: color-mix(in srgb, var(--color-primary) 8%, transparent);
  color: var(--color-text-primary);
}

.fl-model-dropdown-item.is-active {
  background: color-mix(in srgb, var(--color-primary) 12%, transparent);
  color: var(--color-primary);
  font-weight: var(--fw-medium);
}

.fl-model-check { color: var(--color-primary); }

.fl-model-empty {
  font-size: 11px;
  color: var(--color-text-muted);
  padding: var(--sp-2);
  text-align: center;
}

/* ====== 顶栏图标按钮 ====== */
.fl-topbar-icon {
  display: grid;
  place-items: center;
  width: 30px;
  height: 30px;
  border: none;
  border-radius: var(--chat-radius-sm);
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  transition:
    background 100ms var(--ease-smooth),
    color 100ms var(--ease-smooth),
    transform 80ms var(--ease-smooth);
}

.fl-topbar-icon:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.fl-topbar-icon:active {
  transform: scale(0.94);
}

/* ====== 会话下拉 ====== */
.fl-chat-dropdown {
  position: absolute;
  top: 60px;
  left: var(--sp-3);
  right: var(--sp-3);
  max-height: 380px;
  overflow-y: auto;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--chat-radius);
  box-shadow: var(--chat-shadow-md);
  z-index: 50;
  padding: var(--sp-1) 0;
}

.fl-dropdown-label {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 10px var(--sp-3) 4px;
  font-size: 10.5px;
  font-weight: var(--fw-semibold);
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.fl-dropdown-label > svg {
  opacity: 0.7;
}

.fl-dropdown-item {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
  width: 100%;
  padding: 8px var(--sp-2) 8px var(--sp-3);
  border: none;
  background: transparent;
  color: var(--color-text-primary);
  font-size: var(--fs-14);
  text-align: left;
  cursor: pointer;
  border-radius: var(--r-sm);
  margin: 1px var(--sp-1);
  width: calc(100% - var(--sp-2));
  position: relative;
  transition: background 100ms var(--ease-smooth);
}

.fl-dropdown-item:hover {
  background: var(--color-bg-hover);
}

.fl-dropdown-item.is-active {
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
  font-weight: var(--fw-medium);
}

.fl-dropdown-item.is-archived {
  opacity: 0.7;
}

.fl-dropdown-item-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.fl-dropdown-item-text {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fl-dropdown-item-time {
  font-size: 10.5px;
  color: var(--color-text-muted);
  opacity: 0.75;
  letter-spacing: 0.02em;
}

.fl-conv-menu-wrap {
  position: relative;
  flex-shrink: 0;
}

.fl-conv-menu-trigger {
  display: grid;
  place-items: center;
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  color: var(--color-text-muted);
  border-radius: var(--r-xs);
  cursor: pointer;
  opacity: 0;
  transition:
    opacity 100ms var(--ease-smooth),
    background 100ms var(--ease-smooth),
    color 100ms var(--ease-smooth);
}

.fl-dropdown-item:hover .fl-conv-menu-trigger {
  opacity: 1;
}

.fl-conv-menu-trigger:hover {
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
}

.fl-conv-context-menu {
  position: absolute;
  right: 0;
  top: 100%;
  margin-top: 4px;
  min-width: 142px;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--chat-radius-sm);
  box-shadow: var(--chat-shadow-md);
  z-index: 60;
  padding: 4px;
}

.fl-conv-context-menu button {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
  width: 100%;
  padding: 6px 10px;
  border: none;
  background: transparent;
  color: var(--color-text-primary);
  font-size: 13px;
  cursor: pointer;
  border-radius: var(--r-xs);
  transition: background 100ms var(--ease-smooth);
}

.fl-conv-context-menu button:hover {
  background: var(--color-bg-hover);
}

.fl-conv-context-menu button.fl-danger {
  color: var(--color-q1, #ef4444);
}

.fl-conv-context-menu button.fl-danger:hover {
  background: color-mix(in srgb, var(--color-q1, #ef4444) 12%, transparent);
}

.fl-dropdown-new {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
  width: calc(100% - var(--sp-2));
  margin: 4px var(--sp-1) 2px;
  padding: 9px var(--sp-3);
  border: 1px dashed var(--chat-bubble-border);
  border-radius: var(--r-sm);
  background: transparent;
  color: var(--color-primary);
  font-size: var(--fs-14);
  font-weight: var(--fw-medium);
  cursor: pointer;
  transition:
    background 120ms var(--ease-smooth),
    border-color 120ms var(--ease-smooth);
}

.fl-dropdown-new:hover {
  background: var(--color-primary-soft);
  border-style: solid;
}

/* ====== 重命名弹层 ====== */
.fl-rename-overlay {
  position: absolute;
  inset: 0;
  background: color-mix(in srgb, black 35%, transparent);
  z-index: 100;
  display: grid;
  place-items: center;
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
}

.fl-rename-box {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--chat-radius);
  padding: var(--sp-4);
  width: 280px;
  box-shadow: var(--chat-shadow-md);
}

.fl-rename-label {
  display: block;
  font-size: 12px;
  font-weight: var(--fw-semibold);
  color: var(--color-text-secondary);
  margin-bottom: var(--sp-2);
  letter-spacing: 0.02em;
}

.fl-rename-input {
  width: 100%;
  padding: 9px 12px;
  border: 1px solid var(--color-border);
  border-radius: var(--r-sm);
  background: var(--color-bg);
  color: var(--color-text-primary);
  font-size: var(--fs-14);
  font-family: inherit;
  outline: none;
  transition:
    border-color 120ms var(--ease-smooth),
    box-shadow 120ms var(--ease-smooth);
}

.fl-rename-input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-primary) 18%, transparent);
}

.fl-rename-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--sp-2);
  margin-top: var(--sp-3);
}

.fl-rename-cancel,
.fl-rename-confirm {
  padding: 7px 14px;
  border: 1px solid transparent;
  border-radius: var(--r-sm);
  font-size: 13px;
  font-weight: var(--fw-medium);
  cursor: pointer;
  transition:
    background 100ms var(--ease-smooth),
    color 100ms var(--ease-smooth),
    border-color 100ms var(--ease-smooth);
}

.fl-rename-cancel {
  background: transparent;
  border-color: var(--color-border);
  color: var(--color-text-secondary);
}

.fl-rename-cancel:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.fl-rename-confirm {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
}

.fl-rename-confirm:hover {
  background: var(--color-primary-dark);
}

/* ====== Body 容器 (sidebar + main 横向) ====== */
.fl-chat-body {
  flex: 1;
  display: flex;
  min-height: 0;
  overflow: hidden;
}

.fl-chat-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}

/* ====== 侧边会话栏 ====== */
.fl-chat-sidebar {
  width: 220px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  min-height: 0;
  border-right: 1px solid var(--chat-divider);
  background: color-mix(in srgb, var(--color-bg) 60%, transparent);
}

.fl-sidebar-new {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  margin: 10px 10px 6px;
  padding: 7px 12px;
  border: 1px dashed var(--chat-bubble-border);
  border-radius: 9px;
  background: transparent;
  color: var(--color-primary);
  font-size: 13px;
  font-weight: var(--fw-medium);
  cursor: pointer;
  transition:
    background 120ms var(--ease-smooth),
    border-style 120ms var(--ease-smooth);
}

.fl-sidebar-new:hover {
  background: var(--color-primary-soft);
  border-style: solid;
}

.fl-sidebar-list {
  flex: 1;
  overflow-y: auto;
  padding: 2px 4px 8px;
}

.fl-sidebar-list::-webkit-scrollbar {
  width: 6px;
}
.fl-sidebar-list::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--color-text-muted) 26%, transparent);
  border-radius: var(--r-pill);
}

.fl-sidebar-group {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 10px 4px;
  font-size: 10.5px;
  font-weight: var(--fw-semibold);
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  cursor: default;
}

.fl-sidebar-group > svg {
  opacity: 0.6;
}

.fl-sidebar-item {
  display: flex;
  align-items: center;
  gap: 6px;
  width: calc(100% - 8px);
  margin: 1px 4px;
  padding: 7px 6px 7px 10px;
  border: none;
  background: transparent;
  color: var(--color-text-primary);
  font-size: 13px;
  text-align: left;
  cursor: pointer;
  border-radius: 7px;
  position: relative;
  transition: background 100ms var(--ease-smooth);
}

.fl-sidebar-item:hover {
  background: var(--color-bg-hover);
}

.fl-sidebar-item.is-active {
  background: var(--color-primary-soft);
  color: var(--color-primary-dark);
  font-weight: var(--fw-medium);
}

.fl-sidebar-item.is-archived {
  opacity: 0.7;
}

.fl-sidebar-item-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.fl-sidebar-item-text {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fl-sidebar-item-time {
  font-size: 10.5px;
  color: var(--color-text-muted);
  opacity: 0.75;
  letter-spacing: 0.02em;
}

.fl-sidebar-item .fl-conv-menu-trigger {
  opacity: 0;
}

.fl-sidebar-item:hover .fl-conv-menu-trigger,
.fl-sidebar-item.is-active .fl-conv-menu-trigger {
  opacity: 1;
}

.fl-sidebar-archived {
  margin-top: 4px;
}

.fl-sidebar-archived > summary {
  cursor: pointer;
  list-style: none;
  user-select: none;
}
.fl-sidebar-archived > summary::-webkit-details-marker {
  display: none;
}
.fl-sidebar-archived > summary::before {
  content: "›";
  display: inline-block;
  margin-right: 2px;
  transition: transform 120ms var(--ease-smooth);
}
.fl-sidebar-archived[open] > summary::before {
  transform: rotate(90deg);
}

.fl-sidebar-empty {
  padding: 24px 10px;
  text-align: center;
  font-size: 12px;
  color: var(--color-text-muted);
}

/* sidebar slide-in */
.fl-sidebar-enter-active,
.fl-sidebar-leave-active {
  transition: width 220ms cubic-bezier(0.16, 1, 0.3, 1),
    opacity 180ms var(--ease-smooth),
    transform 220ms cubic-bezier(0.16, 1, 0.3, 1);
  overflow: hidden;
}
.fl-sidebar-enter-from,
.fl-sidebar-leave-to {
  width: 0 !important;
  opacity: 0;
  transform: translateX(-12px);
}

/* sidebar 切换按钮 active 态 */
.fl-topbar-sidebar-toggle.is-active {
  background: var(--color-bg-hover);
  color: var(--color-primary);
}

/* ====== 消息列表 ====== */
.fl-chat-messages {
  flex: 1;
  overflow-y: auto;
  padding: var(--sp-4) var(--sp-3);
  display: flex;
  flex-direction: column;
  gap: 14px;
  scroll-behavior: smooth;
  user-select: text;
}

.fl-chat-messages::-webkit-scrollbar {
  width: 8px;
}

.fl-chat-messages::-webkit-scrollbar-track {
  background: transparent;
}

.fl-chat-messages::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--color-text-muted) 30%, transparent);
  border-radius: var(--r-pill);
  border: 2px solid transparent;
  background-clip: padding-box;
}

.fl-chat-messages::-webkit-scrollbar-thumb:hover {
  background: color-mix(in srgb, var(--color-text-muted) 50%, transparent);
  background-clip: padding-box;
}

/* ====== 空状态 ====== */
.fl-chat-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  height: 100%;
  text-align: center;
  padding: var(--sp-4);
  color: var(--color-text-muted);
}

.fl-empty-icon-wrap {
  display: grid;
  place-items: center;
  width: 56px;
  height: 56px;
  border-radius: 16px;
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--color-primary) 14%, transparent),
    color-mix(in srgb, var(--color-primary) 4%, transparent)
  );
  color: var(--color-primary);
  margin-bottom: 4px;
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-primary) 12%, transparent);
}

.fl-empty-title {
  font-size: var(--fs-14);
  font-weight: var(--fw-medium);
  color: var(--color-text-primary);
  margin: 0;
}

.fl-empty-desc {
  font-size: 12.5px;
  color: var(--color-text-muted);
  margin: 0;
  max-width: 240px;
  line-height: 1.5;
}

.fl-empty-link {
  margin-top: 8px;
  padding: 6px 14px;
  border-radius: var(--r-pill);
  background: var(--color-primary);
  color: var(--color-text-on-primary);
  font-size: 13px;
  font-weight: var(--fw-medium);
  text-decoration: none;
  transition:
    background 120ms var(--ease-smooth),
    transform 80ms var(--ease-smooth);
}

.fl-empty-link:hover {
  background: var(--color-primary-dark);
}

.fl-empty-link:active {
  transform: scale(0.97);
}

/* ====== 单条消息 ====== */
.fl-msg-wrap {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin: 6px 0;
}

.fl-msg-time {
  text-align: center;
  font-size: 11px;
  color: var(--color-text-muted);
  letter-spacing: 0.02em;
  user-select: none;
  opacity: 0.75;
}

.fl-msg {
  display: flex;
  gap: var(--sp-2);
  position: relative;
  padding: 2px 0;
}

/* AI 消息靠左，用户消息靠右 */
.fl-msg.is-assistant {
  justify-content: flex-start;
}

.fl-msg.is-user {
  justify-content: flex-end;
}

.fl-msg-fadein {
  animation: fl-msg-in 200ms var(--ease-smooth);
}

@keyframes fl-msg-in {
  from {
    opacity: 0;
    transform: translateY(4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.fl-msg-avatar {
  flex-shrink: 0;
  width: 26px;
  height: 26px;
  border-radius: 8px;
  display: grid;
  place-items: center;
  margin-top: 18px;
  border: 1px solid var(--chat-divider);
}

.fl-msg.is-user .fl-msg-avatar {
  background: var(--chat-bubble-bg);
  color: var(--color-primary);
  border-color: var(--chat-bubble-border);
  margin-top: 4px;
  order: 1;
}

.fl-msg.is-assistant .fl-msg-avatar {
  background: var(--color-bg);
  color: var(--color-text-secondary);
}

.fl-msg-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.fl-msg-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11.5px;
  color: var(--color-text-muted);
  letter-spacing: 0.01em;
}

.fl-msg-role {
  font-weight: var(--fw-medium);
  color: var(--color-text-secondary);
}

.fl-msg-model {
  padding: 1px 7px;
  border-radius: var(--r-pill);
  background: var(--color-bg);
  border: 1px solid var(--chat-divider);
  font-family: var(--font-mono, ui-monospace, "SF Mono", monospace);
  font-size: 10.5px;
  color: var(--color-text-muted);
}

.fl-msg-content {
  font-size: var(--fs-14);
  line-height: 1.5;
  color: var(--color-text-primary);
  word-break: break-word;
}

/* 用户纯文本消息保留换行;助手 markdown 容器不能用 pre-wrap,
   否则 marked 输出的 HTML 字符串里的 \n 会被当作字面换行渲染,
   导致每段间多出一行空白。 */
.fl-msg.is-user .fl-msg-text {
  white-space: pre-wrap;
}

.fl-msg.is-user .fl-msg-meta {
  justify-content: flex-end;
}

.fl-msg.is-user .fl-msg-body.is-user-body {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
  min-width: 0;
}

.fl-msg.is-user .fl-msg-content {
  background: var(--chat-bubble-bg);
  border: 1px solid var(--chat-bubble-border);
  padding: 8px 12px;
  border-radius: var(--chat-radius);
  border-top-right-radius: 4px;
  max-width: 85%;
}

.fl-msg.is-assistant .fl-msg-content {
  padding: 0;
}

.fl-msg.is-error .fl-msg-content {
  border-color: color-mix(in srgb, var(--color-q1, #ef4444) 30%, transparent);
}

.fl-msg.is-aborted .fl-msg-content {
  opacity: 0.85;
}

/* ====== Markdown 渲染样式 (v-html 注入,需 :deep) ====== */
.fl-msg-md {
  font-size: var(--fs-14);
  line-height: 1.6;
  color: var(--color-text-primary);
  word-break: break-word;
}

/* 所有块级元素统一垂直节奏 0.8em */
.fl-msg-md :deep(p),
.fl-msg-md :deep(ul),
.fl-msg-md :deep(ol),
.fl-msg-md :deep(h1),
.fl-msg-md :deep(h2),
.fl-msg-md :deep(h3),
.fl-msg-md :deep(h4),
.fl-msg-md :deep(pre),
.fl-msg-md :deep(blockquote),
.fl-msg-md :deep(table),
.fl-msg-md :deep(hr) {
  margin: 0.8em 0;
}

.fl-msg-md :deep(:first-child) {
  margin-top: 0 !important;
}
.fl-msg-md :deep(:last-child) {
  margin-bottom: 0 !important;
}

.fl-msg-md :deep(h1),
.fl-msg-md :deep(h2),
.fl-msg-md :deep(h3),
.fl-msg-md :deep(h4) {
  margin-bottom: 0.4em;
  font-weight: var(--fw-semibold);
  letter-spacing: -0.01em;
  color: var(--color-text-primary);
}
.fl-msg-md :deep(h1) { font-size: 17px; }
.fl-msg-md :deep(h2) { font-size: 15.5px; }
.fl-msg-md :deep(h3) { font-size: 14.5px; }
.fl-msg-md :deep(h4) { font-size: 13.5px; }

.fl-msg-md :deep(ul),
.fl-msg-md :deep(ol) {
  padding-left: 1.5em;
}

.fl-msg-md :deep(li) {
  margin: 0;
}

.fl-msg-md :deep(li > p) {
  margin: 0;
}

.fl-msg-md :deep(br) {
  line-height: 1;
}

.fl-msg-md :deep(strong) {
  font-weight: var(--fw-semibold);
  color: var(--color-text-primary);
}
.fl-msg-md :deep(em) {
  font-style: italic;
}
.fl-msg-md :deep(del) {
  color: var(--color-text-muted);
}

.fl-msg-md :deep(a) {
  color: var(--color-primary);
  text-decoration: none;
  border-bottom: 1px solid color-mix(in srgb, var(--color-primary) 30%, transparent);
  transition: border-color 100ms var(--ease-smooth);
}
.fl-msg-md :deep(a:hover) {
  border-bottom-color: var(--color-primary);
}

.fl-msg-md :deep(blockquote) {
  padding: 1rem 1.5rem;
  background-color: color-mix(in srgb, var(--color-primary) 6%, transparent);
  font-style: italic;
  color: var(--color-text-secondary);
  border-left: 4px solid color-mix(in srgb, var(--color-primary) 40%, transparent);
  border-radius: var(--r-sm);
}

.fl-msg-md :deep(code) {
  padding: 1px 6px;
  border-radius: 5px;
  background: color-mix(in srgb, var(--color-text-muted) 14%, transparent);
  font-family: var(--font-mono, ui-monospace, "SF Mono", "Cascadia Mono", monospace);
  font-size: 0.92em;
  color: color-mix(in srgb, var(--color-primary) 80%, var(--color-text-primary));
}

.fl-msg-md :deep(pre) {
  padding: 12px 14px;
  background: var(--color-bg);
  border: 1px solid var(--chat-divider);
  border-radius: 10px;
  overflow-x: auto;
  font-size: 12.5px;
  line-height: 1.55;
}

.fl-msg-md :deep(pre code) {
  padding: 0;
  background: transparent;
  color: var(--color-text-primary);
  font-size: inherit;
}

.fl-msg-md :deep(hr) {
  border: 0;
  border-top: 1px solid var(--chat-divider);
}

.fl-msg-md :deep(table) {
  display: block;
  max-width: 100%;
  overflow-x: auto;
  border-collapse: collapse;
  font-size: 13px;
}
.fl-msg-md :deep(th),
.fl-msg-md :deep(td) {
  padding: 4px 8px;
  border: 1px solid var(--chat-divider);
  text-align: left;
}
.fl-msg-md :deep(th) {
  background: color-mix(in srgb, var(--color-text-muted) 8%, transparent);
  font-weight: var(--fw-medium);
}

.fl-msg-md :deep(img) {
  max-width: 100%;
  border-radius: 8px;
}

.fl-msg-md :deep(input[type="checkbox"]) {
  margin-right: 6px;
}

.fl-aborted-tag {
  margin-left: 6px;
  font-size: 11.5px;
  color: var(--color-text-muted);
  font-style: italic;
}

/* 流式光标 */
.fl-cursor {
  display: inline-block;
  width: 2px;
  height: 1em;
  background: var(--color-primary);
  margin-left: 2px;
  vertical-align: text-bottom;
  border-radius: 1px;
  animation: fl-blink 1s ease-in-out infinite;
}

@keyframes fl-blink {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.2;
  }
}

/* 思考点 */
.fl-thinking {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 0;
}

.fl-thinking > span {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--color-text-muted);
  animation: fl-bounce 1.2s ease-in-out infinite;
}

.fl-thinking > span:nth-child(2) {
  animation-delay: 150ms;
}

.fl-thinking > span:nth-child(3) {
  animation-delay: 300ms;
}

@keyframes fl-bounce {
  0%,
  60%,
  100% {
    transform: translateY(0);
    opacity: 0.4;
  }
  30% {
    transform: translateY(-3px);
    opacity: 1;
  }
}

/* 错误 */
.fl-msg-error {
  display: flex;
  align-items: center;
  gap: var(--sp-2);
  margin-top: 4px;
  padding: 8px 10px;
  border-radius: var(--r-sm);
  background: color-mix(in srgb, var(--color-q1, #ef4444) 8%, transparent);
  border: 1px solid color-mix(in srgb, var(--color-q1, #ef4444) 22%, transparent);
}

.fl-msg-error-text {
  flex: 1;
  font-size: 12.5px;
  color: var(--color-q1, #ef4444);
  word-break: break-word;
}

.fl-retry-btn {
  flex-shrink: 0;
  padding: 3px 10px;
  border: 1px solid color-mix(in srgb, var(--color-q1, #ef4444) 30%, transparent);
  border-radius: var(--r-sm);
  background: transparent;
  color: var(--color-q1, #ef4444);
  font-size: 12px;
  font-weight: var(--fw-medium);
  cursor: pointer;
  transition: background 100ms var(--ease-smooth);
}

.fl-retry-btn:hover {
  background: color-mix(in srgb, var(--color-q1, #ef4444) 14%, transparent);
}

/* 悬浮操作 */
.fl-msg-action {
  position: absolute;
  top: 16px;
  right: 0;
  width: 24px;
  height: 24px;
  border: 1px solid var(--chat-divider);
  border-radius: var(--r-xs);
  background: var(--color-bg-elevated);
  color: var(--color-text-muted);
  cursor: pointer;
  display: grid;
  place-items: center;
  opacity: 0;
  transform: scale(0.92);
  transition:
    opacity 120ms var(--ease-smooth),
    transform 120ms var(--ease-smooth),
    color 100ms var(--ease-smooth),
    border-color 100ms var(--ease-smooth);
}

.fl-msg:hover .fl-msg-action {
  opacity: 1;
  transform: scale(1);
}

.fl-msg-action:hover {
  color: var(--color-text-primary);
  border-color: var(--color-text-muted);
}

.fl-msg-action.is-copied {
  color: var(--color-success, #10b981);
  border-color: color-mix(in srgb, var(--color-success, #10b981) 30%, transparent);
  opacity: 1;
}

/* ====== 输入区 ====== */
.fl-chat-input-wrap {
  flex-shrink: 0;
  padding: var(--sp-2) var(--sp-3) var(--sp-3);
  border-top: 1px solid var(--chat-divider);
  background: linear-gradient(
    180deg,
    color-mix(in srgb, var(--color-bg-elevated) 60%, transparent),
    var(--color-bg-elevated)
  );
}

.fl-chat-input {
  display: flex;
  flex-direction: column;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: var(--chat-radius);
  transition:
    border-color 120ms var(--ease-smooth),
    box-shadow 120ms var(--ease-smooth);
}

.fl-chat-input:focus-within {
  border-color: color-mix(in srgb, var(--color-primary) 50%, var(--color-border));
}

.fl-chat-input.is-disabled {
  opacity: 0.6;
}

.fl-chat-textarea {
  padding: 10px 12px 4px;
  border: none;
  background: transparent;
  color: var(--color-text-primary);
  font-size: var(--fs-14);
  font-family: var(--font-sans);
  line-height: 1.55;
  resize: none;
  outline: none;
  max-height: 160px;
  user-select: text;
}

.fl-chat-textarea::placeholder {
  color: var(--color-text-muted);
}

.fl-chat-textarea:disabled {
  cursor: not-allowed;
}

.fl-input-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 6px 6px;
  gap: 6px;
}

.fl-send-btn {
  flex-shrink: 0;
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 50%;
  background: var(--color-primary);
  color: var(--color-text-on-primary);
  cursor: pointer;
  transition:
    background 120ms var(--ease-smooth),
    opacity 100ms var(--ease-smooth),
    transform 80ms var(--ease-smooth);
}

.fl-send-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
  box-shadow: none;
}

.fl-send-btn:not(:disabled):hover {
  background: var(--color-primary-dark);
}

.fl-send-btn:not(:disabled):active {
  transform: scale(0.92);
}

.fl-send-btn.is-abort {
  background: var(--color-q1, #ef4444);
  box-shadow: 0 1px 2px color-mix(in srgb, var(--color-q1, #ef4444) 30%, transparent);
  animation: fl-pulse-abort 1.5s ease-in-out infinite;
}

.fl-send-btn.is-abort:hover {
  background: color-mix(in srgb, var(--color-q1, #ef4444) 90%, black);
}

@keyframes fl-pulse-abort {
  0%,
  100% {
    box-shadow: 0 0 0 0 color-mix(in srgb, var(--color-q1, #ef4444) 40%, transparent);
  }
  50% {
    box-shadow: 0 0 0 6px color-mix(in srgb, var(--color-q1, #ef4444) 0%, transparent);
  }
}

.fl-chat-hint {
  margin: 6px 4px 0;
  font-size: 11px;
  color: var(--color-text-muted);
  letter-spacing: 0.01em;
}

/* ====== 过渡动画 ====== */
.fl-dropdown-up-enter-active,
.fl-dropdown-up-leave-active {
  transition:
    opacity 150ms var(--ease-smooth),
    transform 150ms var(--ease-smooth);
}

.fl-dropdown-up-enter-from,
.fl-dropdown-up-leave-to {
  opacity: 0;
  transform: translateY(6px) scale(0.98);
}

.fl-dropdown-enter-active,
.fl-dropdown-leave-active {
  transition:
    opacity 150ms var(--ease-smooth),
    transform 150ms var(--ease-smooth);
}

.fl-dropdown-enter-from,
.fl-dropdown-leave-to {
  opacity: 0;
  transform: translateY(-6px) scale(0.98);
}

.fl-modal-enter-active,
.fl-modal-leave-active {
  transition: opacity 180ms var(--ease-smooth);
}

.fl-modal-enter-from,
.fl-modal-leave-to {
  opacity: 0;
}

.fl-modal-enter-active .fl-rename-box,
.fl-modal-leave-active .fl-rename-box {
  transition: transform 200ms var(--ease-smooth);
}

.fl-modal-enter-from .fl-rename-box,
.fl-modal-leave-to .fl-rename-box {
  transform: scale(0.96) translateY(4px);
}

/* ====== Reduced motion ====== */
@media (prefers-reduced-motion: reduce) {
  .fl-msg-fadein,
  .fl-cursor,
  .fl-thinking > span,
  .fl-send-btn.is-abort {
    animation: none !important;
  }
  *,
  *::before,
  *::after {
    transition-duration: 0.01ms !important;
  }
  .fl-cursor {
    opacity: 0.6;
  }
}

/* ====== 浮动触发按钮 (FAB) ====== */
.fl-chat-fab {
  position: fixed;
  bottom: 28px;
  right: 28px;
  z-index: 39;
  width: 48px;
  height: 48px;
  border: 1px solid color-mix(in srgb, var(--color-primary) 25%, transparent);
  border-radius: 15px;
  background: linear-gradient(
    135deg,
    color-mix(in srgb, var(--color-primary) 16%, var(--color-bg-elevated)),
    color-mix(in srgb, var(--color-primary) 6%, var(--color-bg-elevated))
  );
  backdrop-filter: blur(12px) saturate(120%);
  -webkit-backdrop-filter: blur(12px) saturate(120%);
  color: var(--color-primary);
  cursor: pointer;
  display: grid;
  place-items: center;
  box-shadow:
    0 2px 8px color-mix(in srgb, var(--color-primary) 20%, transparent),
    0 8px 24px color-mix(in srgb, black 12%, transparent);
  transition:
    transform 150ms var(--ease-smooth),
    box-shadow 150ms var(--ease-smooth),
    background 120ms var(--ease-smooth);
}

.fl-chat-fab:hover {
  transform: scale(1.06);
  box-shadow:
    0 2px 8px color-mix(in srgb, var(--color-primary) 30%, transparent),
    0 12px 32px color-mix(in srgb, black 16%, transparent);
}

.fl-chat-fab:active {
  transform: scale(0.96);
}

/* FAB 渐入 */
.fl-fab-enter-active {
  transition: opacity 200ms var(--ease-smooth), transform 200ms var(--ease-smooth);
}
.fl-fab-leave-active {
  transition: opacity 120ms var(--ease-smooth), transform 120ms var(--ease-smooth);
}
.fl-fab-enter-from,
.fl-fab-leave-to {
  opacity: 0;
  transform: scale(0.8) translateY(8px);
}

/* ====== 抽屉滑入/滑出 ====== */
.fl-drawer-enter-active {
  transition: transform 260ms cubic-bezier(0.16, 1, 0.3, 1),
    opacity 200ms var(--ease-smooth);
}
.fl-drawer-leave-active {
  transition: transform 180ms var(--ease-smooth),
    opacity 140ms var(--ease-smooth);
}
.fl-drawer-enter-from,
.fl-drawer-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>

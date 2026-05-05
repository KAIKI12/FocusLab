/// <reference types="vite/client" />
import { invoke, Channel } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { ref, computed } from "vue";

// ---------- 类型 ----------

export interface Conversation {
  id: string;
  title: string;
  originType: string;
  originId: string | null;
  provider: string;
  apiFormat: string;
  model: string;
  systemPrompt: string;
  messageCount: number;
  pinned: boolean;
  archived: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface ChatMessage {
  id: string;
  conversationId: string;
  role: string;
  content: string;
  model: string | null;
  status: string;
  errorMessage: string | null;
  toolCalls: string | null;
  toolResults: string | null;
  tokensIn: number | null;
  tokensOut: number | null;
  createdAt: string;
}

interface StreamChunk {
  delta: string;
  done: boolean;
  error: string | null;
}

interface SendMessageResult {
  userMessageId: string;
  assistantMessageId: string;
  status: string;
  error: string | null;
}

// ---------- store ----------

export const useChatStore = defineStore("chat", () => {
  const conversations = ref<Conversation[]>([]);
  const currentId = ref<string | null>(null);
  const messages = ref<ChatMessage[]>([]);
  const streamingMessageId = ref<string | null>(null);
  const streamingContent = ref("");
  const isSending = ref(false);
  const aiConfigured = ref(false);

  const currentConversation = computed(() =>
    conversations.value.find((c) => c.id === currentId.value) ?? null,
  );

  // ---- 会话 CRUD ----

  async function loadConversations() {
    conversations.value = await invoke<Conversation[]>("list_conversations", {
      includeArchived: false,
    });
  }

  async function selectConversation(id: string) {
    currentId.value = id;
    messages.value = await invoke<ChatMessage[]>("list_chat_messages", {
      conversationId: id,
    });
  }

  async function createConversation(opts?: {
    title?: string;
    originType?: string;
    originId?: string;
    systemPrompt?: string;
    model?: string;
  }) {
    const id = await invoke<string>("create_conversation", {
      input: {
        title: opts?.title,
        originType: opts?.originType,
        originId: opts?.originId,
        systemPrompt: opts?.systemPrompt,
        model: opts?.model,
      },
    });
    await loadConversations();
    await selectConversation(id);
    return id;
  }

  async function renameConversation(id: string, title: string) {
    await invoke("rename_conversation", { id, title });
    await loadConversations();
  }

  async function setConversationModel(id: string, model: string) {
    await invoke("set_conversation_model", { id, model });
    await loadConversations();
  }

  async function pinConversation(id: string, pinned: boolean) {
    await invoke("pin_conversation", { id, pinned });
    await loadConversations();
  }

  async function archiveConversation(id: string, archived: boolean) {
    await invoke("archive_conversation", { id, archived });
    await loadConversations();
    if (currentId.value === id && archived) {
      currentId.value = null;
      messages.value = [];
    }
  }

  async function deleteConversation(id: string) {
    await invoke("delete_conversation", { id });
    if (currentId.value === id) {
      currentId.value = null;
      messages.value = [];
    }
    await loadConversations();
  }

  async function clearAllConversations() {
    await invoke("delete_all_conversations");
    currentId.value = null;
    messages.value = [];
    await loadConversations();
  }

  // ---- 流式发送 ----

  async function sendMessage(content: string) {
    if (!currentId.value || isSending.value) return;
    isSending.value = true;

    const onChunk = new Channel<StreamChunk>();
    onChunk.onmessage = (chunk: StreamChunk) => {
      if (chunk.delta) {
        streamingContent.value += chunk.delta;
      }
      if (chunk.done) {
        // done 会在 send_chat_message 返回后处理
      }
    };

    streamingContent.value = "";
    streamingMessageId.value = "pending";

    try {
      const result = await invoke<SendMessageResult>("send_chat_message", {
        conversationId: currentId.value,
        content,
        onChunk,
      });

      streamingMessageId.value = result.assistantMessageId;

      // 刷新消息列表
      messages.value = await invoke<ChatMessage[]>("list_chat_messages", {
        conversationId: currentId.value!,
      });
      await loadConversations();
    } catch (err) {
      console.error("[chat] send failed:", err);
    } finally {
      streamingMessageId.value = null;
      streamingContent.value = "";
      isSending.value = false;
    }
  }

  async function abortStreaming() {
    if (!currentId.value) return;
    await invoke("abort_chat_message", { conversationId: currentId.value });
  }

  // ---- 自检 ----

  async function checkConfigured() {
    aiConfigured.value = await invoke<boolean>("ai_chat_is_configured");
  }

  // ---- 从灵感创建 ----

  async function createFromInspiration(
    noteId: string,
    raw: string,
    question: string,
  ) {
    const id = await createConversation({
      title: question.slice(0, 30),
      originType: "inspiration",
      originId: noteId,
      systemPrompt: `【用户的原始速记】\n${raw}\n\n【AI 优化后的研究问题】\n${question}\n\n你是用户的研究伙伴。帮助用户把模糊的研究想法打磨成可探索的子问题、理清思路、找到切入方向。保持温和鼓励，追问时给出具体选项而非泛泛而谈。`,
    });
    // 不自动发送,预填到输入框(由 UI 层读取 prefill)
    prefillMessage.value = question;
    return id;
  }

  async function createFromInspirations(items: Array<{ id: string; content: string }>) {
    const bulletList = items.map((it, i) => `${i + 1}. ${it.content}`).join("\n");
    const idList = items.map((it) => it.id).join(",");
    const systemPrompt = `【用户选中的 ${items.length} 条灵感】\n${bulletList}\n\n你是用户的研究反思伙伴。请从以下角度帮助用户：\n1. **关联发现** — 这些灵感之间有什么隐含联系？是否存在共同主题、递进关系或矛盾？\n2. **反思引导** — 基于这些灵感的交集，提出 2-3 个值得深挖的研究方向或问题。\n3. **整合建议** — 如果将这些灵感合并思考，可能产生什么新的研究思路？\n\n保持温和鼓励，回答具体而非泛泛而谈。用编号引用具体灵感条目。`;
    const id = await createConversation({
      title: `${items.length} 条灵感关联探索`,
      originType: "inspiration_batch",
      originId: idList,
      systemPrompt,
    });
    // 自动发送，消息直接显示在聊天中(把选中的灵感原文一并展示)
    const userMessage = `请帮我探索以下 ${items.length} 条灵感之间的关联和反思方向：\n\n${bulletList}`;
    await sendMessage(userMessage);
    return id;
  }

  const prefillMessage = ref("");

  return {
    conversations,
    currentId,
    messages,
    streamingMessageId,
    streamingContent,
    isSending,
    aiConfigured,
    currentConversation,
    prefillMessage,

    loadConversations,
    selectConversation,
    createConversation,
    renameConversation,
    setConversationModel,
    pinConversation,
    archiveConversation,
    deleteConversation,
    clearAllConversations,
    sendMessage,
    abortStreaming,
    checkConfigured,
    createFromInspiration,
    createFromInspirations,
  };
});

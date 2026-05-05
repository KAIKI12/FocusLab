/**
 * useAIProfileStore · AI Provider Profile - chat / embedding 双独立池子。
 *
 * 设计:
 * - chat / embedding 各自一份列表 + activeId
 * - 修改激活的 profile 时,后端会自动同步运行态 AIService
 * - 删除激活的 profile 时,后端会自动 fallback 到剩余里最早的一条
 */

import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";

export interface ChatProfile {
  id: string;
  name: string;
  provider: string;
  apiFormat: string;
  baseUrl: string;
  apiKey: string;
  modelFast: string;
  modelStrong: string;
  selectedModels: string[];
  createdAt: string;
  updatedAt: string;
}

export interface EmbeddingProfile {
  id: string;
  name: string;
  baseUrl: string;
  apiKey: string;
  model: string;
  createdAt: string;
  updatedAt: string;
}

interface RustChatProfile {
  id: string;
  name: string;
  provider: string;
  api_format: string;
  base_url: string;
  api_key: string;
  model_fast: string;
  model_strong: string;
  selected_models: string;
  created_at: string;
  updated_at: string;
}

interface RustEmbeddingProfile {
  id: string;
  name: string;
  base_url: string;
  api_key: string;
  model: string;
  created_at: string;
  updated_at: string;
}

function fromRustChat(r: RustChatProfile): ChatProfile {
  let selectedModels: string[] = [];
  try {
    selectedModels = JSON.parse(r.selected_models || "[]");
  } catch { /* ignore */ }
  return {
    id: r.id,
    name: r.name,
    provider: r.provider,
    apiFormat: r.api_format,
    baseUrl: r.base_url,
    apiKey: r.api_key,
    modelFast: r.model_fast,
    modelStrong: r.model_strong,
    selectedModels,
    createdAt: r.created_at,
    updatedAt: r.updated_at,
  };
}

function fromRustEmbedding(r: RustEmbeddingProfile): EmbeddingProfile {
  return {
    id: r.id,
    name: r.name,
    baseUrl: r.base_url,
    apiKey: r.api_key,
    model: r.model,
    createdAt: r.created_at,
    updatedAt: r.updated_at,
  };
}

export const useAIProfileStore = defineStore("aiProfile", () => {
  const chatProfiles = ref<ChatProfile[]>([]);
  const embeddingProfiles = ref<EmbeddingProfile[]>([]);
  const activeChatId = ref<string>("");
  const activeEmbeddingId = ref<string>("");

  async function loadAll() {
    const [chats, embs, ac, ae] = await Promise.all([
      invokeCmd<RustChatProfile[]>("list_chat_profiles"),
      invokeCmd<RustEmbeddingProfile[]>("list_embedding_profiles"),
      invokeCmd<string | null>("get_active_chat_profile_id"),
      invokeCmd<string | null>("get_active_embedding_profile_id"),
    ]);
    chatProfiles.value = chats.map(fromRustChat);
    embeddingProfiles.value = embs.map(fromRustEmbedding);
    activeChatId.value = ac ?? "";
    activeEmbeddingId.value = ae ?? "";
  }

  async function createChat(input: Omit<ChatProfile, "id" | "createdAt" | "updatedAt">) {
    const id = await invokeCmd<string>("create_chat_profile", {
      input: {
        name: input.name,
        provider: input.provider,
        apiFormat: input.apiFormat,
        baseUrl: input.baseUrl,
        apiKey: input.apiKey,
        modelFast: input.modelFast,
        modelStrong: input.modelStrong,
        selectedModels: JSON.stringify(input.selectedModels),
      },
    });
    await loadAll();
    return id;
  }

  async function updateChat(p: ChatProfile) {
    await invokeCmd<void>("update_chat_profile", {
      input: {
        id: p.id,
        name: p.name,
        provider: p.provider,
        apiFormat: p.apiFormat,
        baseUrl: p.baseUrl,
        apiKey: p.apiKey,
        modelFast: p.modelFast,
        modelStrong: p.modelStrong,
        selectedModels: JSON.stringify(p.selectedModels),
      },
    });
    await loadAll();
  }

  async function deleteChat(id: string) {
    await invokeCmd<void>("delete_chat_profile", { id });
    await loadAll();
  }

  async function activateChat(id: string) {
    await invokeCmd<void>("activate_chat_profile", { id });
    activeChatId.value = id;
  }

  async function createEmbedding(input: Omit<EmbeddingProfile, "id" | "createdAt" | "updatedAt">) {
    const id = await invokeCmd<string>("create_embedding_profile", {
      input: {
        name: input.name,
        baseUrl: input.baseUrl,
        apiKey: input.apiKey,
        model: input.model,
      },
    });
    await loadAll();
    return id;
  }

  async function updateEmbedding(p: EmbeddingProfile) {
    await invokeCmd<void>("update_embedding_profile", {
      input: {
        id: p.id,
        name: p.name,
        baseUrl: p.baseUrl,
        apiKey: p.apiKey,
        model: p.model,
      },
    });
    await loadAll();
  }

  async function deleteEmbedding(id: string) {
    await invokeCmd<void>("delete_embedding_profile", { id });
    await loadAll();
  }

  async function activateEmbedding(id: string) {
    await invokeCmd<void>("activate_embedding_profile", { id });
    activeEmbeddingId.value = id;
  }

  return {
    chatProfiles,
    embeddingProfiles,
    activeChatId,
    activeEmbeddingId,
    loadAll,
    createChat,
    updateChat,
    deleteChat,
    activateChat,
    createEmbedding,
    updateEmbedding,
    deleteEmbedding,
    activateEmbedding,
  };
});

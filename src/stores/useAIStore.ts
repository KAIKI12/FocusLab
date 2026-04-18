/**
 * useAIStore · AI 操作 store。
 */

import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";

export interface SubTask {
  name: string;
  estimatedMinutes: number;
  quadrant: string;
}

export const useAIStore = defineStore("ai", () => {
  const loading = ref(false);
  const configured = ref(false);

  async function configure(
    provider: string,
    baseUrl: string,
    apiKey: string,
    model: string,
  ) {
    await invokeCmd<void>("configure_ai", { provider, baseUrl, apiKey, model });
    configured.value = true;
  }

  async function testConnection(): Promise<string> {
    return invokeCmd<string>("test_ai_connection");
  }

  async function decomposeTask(taskName: string, description?: string): Promise<SubTask[]> {
    loading.value = true;
    try {
      const raw = await invokeCmd<string>("ai_decompose_task", {
        input: { taskName, description: description ?? "" },
      });
      // 解析 JSON 数组
      const parsed = JSON.parse(raw);
      if (!Array.isArray(parsed)) throw new Error("AI 返回非数组");
      return parsed.slice(0, 7).map((item: Record<string, unknown>) => ({
        name: String(item.name ?? ""),
        estimatedMinutes: Number(item.estimatedMinutes ?? 30),
        quadrant: String(item.quadrant ?? "important_not_urgent"),
      }));
    } catch (e) {
      console.error("[ai] decompose failed", e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function generateNarrative(
    grade: string,
    completed: number,
    total: number,
    focusMinutes: number,
    tone?: string,
  ): Promise<string> {
    loading.value = true;
    try {
      return await invokeCmd<string>("ai_settlement_narrative", {
        input: { grade, completed, total, focusMinutes, tone: tone ?? "academic" },
      });
    } finally {
      loading.value = false;
    }
  }

  return { loading, configured, configure, testConnection, decomposeTask, generateNarrative };
});

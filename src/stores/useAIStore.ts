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

export interface QuickNoteCandidate {
  label: string;
  style: string;
  styleName: string;
  text: string;
  quadrant?: string;
}

export const useAIStore = defineStore("ai", () => {
  const loading = ref(false);
  const configured = ref(false);

  /**
   * 配置 AI provider。
   * 所有参数持久化到 settings KV；apiFormat 仅在 provider === "custom" 时生效。
   */
  async function configure(
    provider: string,
    apiFormat: string,
    baseUrl: string,
    apiKey: string,
    model: string,
    enabled?: string,
    tone?: string,
    toneCustom?: string,
    intensity?: string,
  ) {
    await invokeCmd<void>("configure_ai", {
      provider,
      apiFormat,
      baseUrl,
      apiKey,
      model,
      enabled,
      tone,
      toneCustom,
      intensity,
    });
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

  async function dailySuggestions(energyLevel?: string): Promise<string> {
    loading.value = true;
    try {
      return await invokeCmd<string>("ai_daily_suggestions", {
        input: { energyLevel: energyLevel ?? "正常" },
      });
    } finally {
      loading.value = false;
    }
  }

  async function classifyQuadrant(taskName: string, description?: string): Promise<string> {
    return invokeCmd<string>("ai_classify_quadrant", {
      input: { taskName, description: description ?? "" },
    });
  }

  async function weeklySummary(): Promise<string> {
    loading.value = true;
    try {
      return await invokeCmd<string>("ai_weekly_summary");
    } finally {
      loading.value = false;
    }
  }

  async function optimizeQuickNote(rawText: string): Promise<QuickNoteCandidate[]> {
    loading.value = true;
    try {
      const raw = await invokeCmd<string>("ai_optimize_quick_note", {
        input: { rawText },
      });
      const parsed = JSON.parse(raw);
      const candidates: QuickNoteCandidate[] = (parsed.candidates ?? [])
        .slice(0, 3)
        .map((c: Record<string, unknown>) => ({
          label: String(c.label ?? ""),
          style: String(c.style ?? "note"),
          styleName: String(c.styleName ?? ""),
          text: String(c.text ?? ""),
          quadrant: c.quadrant ? String(c.quadrant) : undefined,
        }));
      if (candidates.length < 3) throw new Error("AI 返回候选不足 3 个");
      return candidates;
    } catch (e) {
      console.error("[ai] optimizeQuickNote failed", e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  return {
    loading, configured, configure, testConnection,
    decomposeTask, generateNarrative, dailySuggestions, classifyQuadrant, weeklySummary,
    optimizeQuickNote,
  };
});

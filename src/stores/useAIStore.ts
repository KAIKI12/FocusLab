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

interface OptimizeCacheEntry {
  candidates: QuickNoteCandidate[];
  error: string;
  loading: boolean;
  ts: number;
}

/** 当前会话的梳理状态 — 路由切换后仍可恢复 */
interface OptimizeSession {
  draftText: string;
  candidates: QuickNoteCandidate[];
  error: string;
  loading: boolean;
  show: boolean;
}

export interface MilestoneBreakdownResult {
  goal_understanding: string;
  milestones: Array<{
    name: string;
    order: number;
    deadline_hint: string;
    priority: "high" | "medium" | "low";
    deliverable: string;
  }>;
}

export interface TaskDurationEstimationResult {
  estimated_minutes: number;
  confidence: "high" | "medium" | "low";
  reasoning: string;
  range: {
    min: number;
    max: number;
  };
}

export interface MilestoneRiskResult {
  risk_level: "high" | "medium" | "low";
  summary: string;
  actions: string[];
}

export const useAIStore = defineStore("ai", () => {
  const loading = ref(false);
  const configured = ref(false);

  // AI 梳理结果缓存 — key 为原文, 路由切换后仍可恢复
  const optimizeCache = ref<Map<string, OptimizeCacheEntry>>(new Map());
  // 当前会话梳理状态（含原文，不依赖组件内 draft ref）
  const optimizeSession = ref<OptimizeSession | null>(null);

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

  async function configureEmbedding(
    baseUrl: string,
    apiKey: string,
    model: string,
    enabled?: string,
  ) {
    await invokeCmd<void>("configure_embedding", {
      baseUrl,
      apiKey,
      model,
      enabled,
    });
  }

  async function testConnection(): Promise<string> {
    return invokeCmd<string>("test_ai_connection");
  }

  async function fetchModels(baseUrl: string, apiKey: string, apiFormat: string): Promise<string[]> {
    return invokeCmd<string[]>("fetch_ai_models", { baseUrl, apiKey, apiFormat });
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
    // 先查缓存
    const cached = optimizeCache.value.get(rawText);
    if (cached && !cached.loading && !cached.error) return cached.candidates;

    loading.value = true;
    // 写入 session 供路由恢复
    optimizeSession.value = { draftText: rawText, candidates: [], error: "", loading: true, show: true };
    optimizeCache.value.set(rawText, { candidates: [], error: "", loading: true, ts: Date.now() });
    try {
      const raw = await invokeCmd<string>("ai_optimize_quick_note", {
        input: { rawText },
      });
      console.debug("[ai] optimizeQuickNote raw:", raw);
      const parsed = JSON.parse(raw) as Record<string, unknown>;
      const candidates: QuickNoteCandidate[] = (
        (parsed.candidates as Record<string, unknown>[]) ?? []
      )
        .slice(0, 3)
        .map((c) => ({
          label: String(c.label ?? ""),
          style: String(c.style ?? "note"),
          styleName: String(c.styleName ?? ""),
          text: String(c.text ?? ""),
          quadrant: c.quadrant ? String(c.quadrant) : undefined,
        }));
      if (candidates.length < 3) throw new Error("AI 返回候选不足 3 个");
      optimizeCache.value.set(rawText, { candidates, error: "", loading: false, ts: Date.now() });
      optimizeSession.value = { draftText: rawText, candidates, error: "", loading: false, show: true };
      return candidates;
    } catch (e) {
      console.error("[ai] optimizeQuickNote failed", e);
      const msg = e instanceof Error ? e.message : String(e);
      optimizeCache.value.set(rawText, { candidates: [], error: msg, loading: false, ts: Date.now() });
      optimizeSession.value = { draftText: rawText, candidates: [], error: msg, loading: false, show: true };
      throw e;
    } finally {
      loading.value = false;
    }
  }

  // ---------- 新 AI 功能 ----------

  /** 未完成任务温和提醒，返回 { message, next_step, tone } */
  async function unfinishedReminder(
    unfinishedTasks: string[],
    completedSummary: string,
    availableTime?: string,
  ): Promise<{ message: string; next_step: string; tone: string }> {
    loading.value = true;
    try {
      const raw = await invokeCmd<string>("ai_unfinished_reminder", {
        input: {
          unfinishedTasks: unfinishedTasks.join("、"),
          completedSummary,
          availableTime: availableTime ?? undefined,
        },
      });
      const parsed = JSON.parse(raw);
      return {
        message: String(parsed.message ?? "今天已有不少收获，明天继续加油！"),
        next_step: String(parsed.next_step ?? "选一项最小的任务先开始"),
        tone: String(parsed.tone ?? "gentle"),
      };
    } catch (e) {
      console.error("[ai] unfinishedReminder failed", e);
      return {
        message: "今天已有不少收获，未完成的任务明天继续加油！",
        next_step: "选一项最小的任务先开始",
        tone: "gentle",
      };
    } finally {
      loading.value = false;
    }
  }

  /** 任务完成正反馈，返回 { message, badge, tone } */
  async function taskFeedback(
    taskName: string,
    estimatedMinutes?: number,
    actualMinutes?: number,
    quadrant?: string,
  ): Promise<{ message: string; badge: string; tone: string }> {
    loading.value = true;
    try {
      const raw = await invokeCmd<string>("ai_task_feedback", {
        input: {
          taskName,
          estimatedMinutes: estimatedMinutes ?? 0,
          actualMinutes: actualMinutes ?? 0,
          quadrant: quadrant ?? "important_not_urgent",
        },
      });
      const parsed = JSON.parse(raw);
      return {
        message: String(parsed.message ?? `「${taskName}」完成了，继续保持！`),
        badge: String(parsed.badge ?? "✅"),
        tone: String(parsed.tone ?? "encouraging"),
      };
    } catch (e) {
      console.error("[ai] taskFeedback failed", e);
      return {
        message: `「${taskName}」完成了，继续保持！`,
        badge: "✅",
        tone: "encouraging",
      };
    } finally {
      loading.value = false;
    }
  }

  /** 里程碑 AI 拆解，返回结构化里程碑建议 */
  async function milestoneBreakdown(
    goalName: string,
    goalDescription?: string,
    totalDeadline?: string,
    currentProgress?: string,
  ): Promise<MilestoneBreakdownResult> {
    loading.value = true;
    try {
      const raw = await invokeCmd<string>("ai_milestone_breakdown", {
        input: {
          goalName,
          goalDescription: goalDescription ?? undefined,
          totalDeadline: totalDeadline ?? undefined,
          currentProgress: currentProgress ?? undefined,
        },
      });
      const parsed = JSON.parse(raw) as MilestoneBreakdownResult;
      if (!Array.isArray(parsed.milestones)) throw new Error("AI 返回格式异常");
      return parsed;
    } catch (e) {
      console.error("[ai] milestoneBreakdown failed", e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function estimateTaskDuration(
    taskName: string,
    description?: string,
  ): Promise<TaskDurationEstimationResult> {
    loading.value = true;
    try {
      const raw = await invokeCmd<string>("ai_estimate_task_duration", {
        input: {
          taskName,
          description: description ?? undefined,
        },
      });
      const parsed = JSON.parse(raw) as TaskDurationEstimationResult;
      if (typeof parsed.estimated_minutes !== "number") {
        throw new Error("AI 返回格式异常");
      }
      return parsed;
    } catch (e) {
      console.error("[ai] estimateTaskDuration failed", e);
      return {
        estimated_minutes: 30,
        confidence: "low",
        reasoning: "AI 返回异常，已使用默认预估",
        range: { min: 15, max: 60 },
      };
    } finally {
      loading.value = false;
    }
  }

  async function milestoneRisk(
    milestoneName: string,
    goalName: string,
    targetDate: string,
    remainingDays: number,
    doneSubtasks: number,
    totalSubtasks: number,
    milestoneId?: string,
  ): Promise<MilestoneRiskResult> {
    loading.value = true;
    try {
      const raw = await invokeCmd<string>("ai_milestone_risk", {
        input: {
          milestoneName,
          goalName,
          targetDate,
          remainingDays,
          doneSubtasks,
          totalSubtasks,
          milestoneId: milestoneId ?? undefined,
        },
      });
      const parsed = JSON.parse(raw) as MilestoneRiskResult;
      if (!parsed.risk_level || !Array.isArray(parsed.actions)) {
        throw new Error("AI 返回格式异常");
      }
      return parsed;
    } catch (e) {
      console.error("[ai] milestoneRisk failed", e);
      return {
        risk_level: "high",
        summary: "当前进度偏慢，存在延期风险",
        actions: ["优先完成最核心子任务", "评估是否可调整截止日期"],
      };
    } finally {
      loading.value = false;
    }
  }

  function clearOptimizeCache() {
    optimizeCache.value.clear();
  }

  // ---- 灵感: AI 推荐归属目标 ----
  // D2: 失败时显式抛出,调用方决定是否提示用户/降级。
  // 不在此处返回伪造的 "AI 推荐失败" 占位 — 那会让上游误以为成功调用。
  async function suggestGoalForInspiration(
    inspirationContent: string,
    goals: Array<{ id: string; name: string }>,
  ): Promise<{ goalId: string | null; reason: string }> {
    return await invokeCmd<{ goalId: string | null; reason: string }>(
      "ai_suggest_goal_for_inspiration",
      {
        input: {
          inspirationContent,
          goals: goals.map((g) => [g.id, g.name]),
        },
      },
    );
  }

  // ---- 灵感: AI 起草后续实验 ----
  // D2: 失败时显式抛出,UI 决定是否提示并提供"用模板代替"选项。
  async function draftFollowupExperiment(parentContent: string): Promise<string> {
    return await invokeCmd<string>("ai_draft_followup_experiment", {
      input: { parentContent },
    });
  }

  // ---- 灵感: AI 纠偏分析 ----
  // D2: 失败时显式抛出。原本返回 null 的 silent fallback 让 UI 无法区分
  // "AI 没意见" 与 "AI 调用失败" 两种语义。
  async function analyzeCorrection(
    oldContent: string,
    newContent: string,
  ): Promise<{ summary: string; oldJudgment: string; newEvidence: string; suggestion: string }> {
    return await invokeCmd<{
      summary: string;
      oldJudgment: string;
      newEvidence: string;
      suggestion: string;
    }>("ai_analyze_correction", { input: { oldContent, newContent } });
  }

  return {
    loading, configured, configure, configureEmbedding, testConnection, fetchModels,
    decomposeTask, generateNarrative, dailySuggestions, classifyQuadrant, weeklySummary,
    optimizeQuickNote, optimizeCache, optimizeSession, clearOptimizeCache,
    unfinishedReminder, taskFeedback, milestoneBreakdown,
    estimateTaskDuration, milestoneRisk,
    suggestGoalForInspiration, draftFollowupExperiment, analyzeCorrection,
  };
});

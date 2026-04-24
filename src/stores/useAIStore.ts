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

  return {
    loading, configured, configure, testConnection,
    decomposeTask, generateNarrative, dailySuggestions, classifyQuadrant, weeklySummary,
    optimizeQuickNote, unfinishedReminder, taskFeedback, milestoneBreakdown,
    estimateTaskDuration, milestoneRisk,
  };
});

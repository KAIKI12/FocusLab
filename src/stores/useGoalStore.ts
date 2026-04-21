/**
 * useGoalStore · 长线目标 + 里程碑 store。
 *
 * v2 扩展(2026-04-21):
 *   - target_date 直改字段 + setTargetDate 动作
 *   - notesByMilestone:Map<milestoneId, MilestoneNote[]> · 按需加载
 *   - weeklyInvest:当前 selectedGoal 的本周 7 桶投入(切换 goal 时自动刷新)
 */

import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import type {
  CreateGoalInput,
  CreateMilestoneInput,
  Goal,
  Milestone,
  MilestoneNote,
  WeeklyInvest,
} from "@/types";

export const useGoalStore = defineStore("goal", () => {
  const goals = ref<Goal[]>([]);
  const milestones = ref<Milestone[]>([]);
  const selectedGoalId = ref<string | null>(null);
  const loading = ref(false);

  // v2: 备注按 milestoneId 分桶缓存
  const notesByMilestone = ref<Record<string, MilestoneNote[]>>({});
  // v2: 当前 goal 的本周投入(切换 goal 时重新拉)
  const weeklyInvest = ref<WeeklyInvest | null>(null);

  async function loadGoals(includeArchived = false) {
    loading.value = true;
    try {
      goals.value = await invokeCmd<Goal[]>("list_goals", { includeArchived });
    } finally {
      loading.value = false;
    }
  }

  async function loadMilestones(goalId: string) {
    milestones.value = await invokeCmd<Milestone[]>("list_milestones", { goalId });
  }

  async function loadWeeklyInvest(goalId: string) {
    try {
      weeklyInvest.value = await invokeCmd<WeeklyInvest>("get_goal_weekly_invest", { goalId });
    } catch (e) {
      console.error("[goal] loadWeeklyInvest failed", e);
      weeklyInvest.value = null;
    }
  }

  async function selectGoal(goalId: string) {
    selectedGoalId.value = goalId;
    notesByMilestone.value = {}; // 切换 goal 清空备注缓存
    await Promise.all([loadMilestones(goalId), loadWeeklyInvest(goalId)]);
  }

  async function createGoal(input: CreateGoalInput) {
    const created = await invokeCmd<Goal>("create_goal", { input });
    goals.value.unshift(created);
    return created;
  }

  async function archiveGoal(id: string) {
    await invokeCmd<void>("archive_goal", { id });
    goals.value = goals.value.filter((g) => g.id !== id);
    if (selectedGoalId.value === id) {
      selectedGoalId.value = null;
      milestones.value = [];
      weeklyInvest.value = null;
      notesByMilestone.value = {};
    }
  }

  async function updateGoal(input: { id: string; name?: string; description?: string; targetDate?: string }) {
    await invokeCmd<void>("update_goal", { input });
    const target = goals.value.find((g) => g.id === input.id);
    if (target) {
      if (input.name) target.name = input.name;
      if (input.description !== undefined) target.description = input.description || null;
      if (input.targetDate !== undefined) target.target_date = input.targetDate || null;
    }
  }

  async function createMilestone(input: CreateMilestoneInput) {
    const created = await invokeCmd<Milestone>("create_milestone", { input });
    milestones.value.push(created);
    return created;
  }

  async function completeMilestone(id: string) {
    await invokeCmd<void>("complete_milestone", { id });
    const target = milestones.value.find((m) => m.id === id);
    if (target) {
      target.status = "completed";
      target.completed_at = new Date().toISOString();
    }
    // 完成里程碑会影响周投入吗? 不会(session 归属不变),不刷新。
  }

  async function updateMilestone(input: { id: string; name?: string; description?: string; status?: string }) {
    await invokeCmd<void>("update_milestone", { input });
    const target = milestones.value.find((m) => m.id === input.id);
    if (target) {
      if (input.name) target.name = input.name;
      if (input.description !== undefined) target.description = input.description || null;
      if (input.status) target.status = input.status;
    }
  }

  // ---------- v2: target_date ----------

  async function setMilestoneTargetDate(milestoneId: string, targetDate: string | null) {
    await invokeCmd<void>("set_milestone_target_date", {
      input: { milestoneId, targetDate },
    });
    const target = milestones.value.find((m) => m.id === milestoneId);
    if (target) target.target_date = targetDate;
  }

  // ---------- v2: notes ----------

  async function loadNotes(milestoneId: string) {
    const list = await invokeCmd<MilestoneNote[]>("list_milestone_notes", { milestoneId });
    notesByMilestone.value = { ...notesByMilestone.value, [milestoneId]: list };
  }

  async function addNote(milestoneId: string, text: string) {
    const created = await invokeCmd<MilestoneNote>("add_milestone_note", {
      input: { milestoneId, text },
    });
    const existing = notesByMilestone.value[milestoneId] ?? [];
    notesByMilestone.value = {
      ...notesByMilestone.value,
      [milestoneId]: [created, ...existing], // 最新在前,和后端 DESC 一致
    };
    return created;
  }

  async function removeNote(milestoneId: string, noteId: string) {
    await invokeCmd<void>("delete_milestone_note", { id: noteId });
    const existing = notesByMilestone.value[milestoneId] ?? [];
    notesByMilestone.value = {
      ...notesByMilestone.value,
      [milestoneId]: existing.filter((n) => n.id !== noteId),
    };
  }

  return {
    goals,
    milestones,
    selectedGoalId,
    loading,
    notesByMilestone,
    weeklyInvest,
    loadGoals,
    loadMilestones,
    loadWeeklyInvest,
    selectGoal,
    createGoal,
    archiveGoal,
    updateGoal,
    createMilestone,
    completeMilestone,
    updateMilestone,
    setMilestoneTargetDate,
    loadNotes,
    addNote,
    removeNote,
  };
});

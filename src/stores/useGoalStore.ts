/**
 * useGoalStore · 长线目标 + 里程碑 store。
 */

import { defineStore } from "pinia";
import { ref } from "vue";

import { invokeCmd } from "@/composables/useTauriInvoke";
import type { CreateGoalInput, CreateMilestoneInput, Goal, Milestone } from "@/types";

export const useGoalStore = defineStore("goal", () => {
  const goals = ref<Goal[]>([]);
  const milestones = ref<Milestone[]>([]);
  const selectedGoalId = ref<string | null>(null);
  const loading = ref(false);

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

  async function selectGoal(goalId: string) {
    selectedGoalId.value = goalId;
    await loadMilestones(goalId);
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

  return {
    goals,
    milestones,
    selectedGoalId,
    loading,
    loadGoals,
    loadMilestones,
    selectGoal,
    createGoal,
    archiveGoal,
    updateGoal,
    createMilestone,
    completeMilestone,
    updateMilestone,
  };
});

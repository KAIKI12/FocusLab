<script setup lang="ts">
/**
 * GoalsView · 长线目标管理页面。
 *
 * 左栏: 目标列表 + 新建目标表单
 * 右栏: 选中目标的里程碑时间线
 */

import { Pencil, Plus } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";

import GoalCard from "@/components/goal/GoalCard.vue";
import MilestoneTimeline from "@/components/goal/MilestoneTimeline.vue";
import { useGoalStore } from "@/stores/useGoalStore";

const goals = useGoalStore();
const newGoalName = ref("");
const editingGoal = ref(false);
const editGoalName = ref("");
const editGoalDesc = ref("");
const editGoalDate = ref("");

const selectedGoal = computed(() =>
  goals.goals.find(g => g.id === goals.selectedGoalId) ?? null,
);

const daysSinceCreated = computed(() => {
  if (!selectedGoal.value) return 0;
  const created = new Date(selectedGoal.value.created_at);
  return Math.floor((Date.now() - created.getTime()) / (1000 * 60 * 60 * 24));
});

onMounted(async () => {
  await goals.loadGoals();
  // 自动选中第一个
  if (goals.goals.length > 0 && !goals.selectedGoalId) {
    await goals.selectGoal(goals.goals[0].id);
  }
});

async function onAddGoal() {
  const name = newGoalName.value.trim();
  if (!name) return;
  const created = await goals.createGoal({ name });
  newGoalName.value = "";
  await goals.selectGoal(created.id);
}

function startEditGoal() {
  if (!selectedGoal.value) return;
  editGoalName.value = selectedGoal.value.name;
  editGoalDesc.value = selectedGoal.value.description ?? "";
  editGoalDate.value = selectedGoal.value.target_date ?? "";
  editingGoal.value = true;
}

async function saveGoalEdit() {
  if (!selectedGoal.value || !editGoalName.value.trim()) return;
  try {
    await goals.updateGoal({
      id: selectedGoal.value.id,
      name: editGoalName.value.trim(),
      description: editGoalDesc.value || undefined,
      targetDate: editGoalDate.value || undefined,
    });
    editingGoal.value = false;
  } catch (e) { console.error(e); }
}
</script>

<template>
  <section class="fl-goals">
    <header class="fl-page-head">
      <h1>长线目标</h1>
      <p class="fl-page-sub">追踪长期目标与里程碑进展</p>
    </header>

    <div class="fl-goals-layout">
      <!-- 左栏:目标列表 -->
      <div class="fl-goals-left">
        <form class="fl-goal-form" @submit.prevent="onAddGoal">
          <input
            v-model="newGoalName"
            class="fl-goal-input"
            type="text"
            placeholder="新建目标…"
            maxlength="60"
          />
          <button class="fl-goal-add" type="submit" :disabled="!newGoalName.trim()">
            <Plus :size="14" />
          </button>
        </form>

        <div v-if="goals.loading" class="fl-goals-empty">载入中…</div>
        <div v-else-if="!goals.goals.length" class="fl-goals-empty">
          还没有长线目标 · 写下你的第一个 ↑
        </div>
        <div v-else class="fl-goals-list">
          <GoalCard
            v-for="g in goals.goals"
            :key="g.id"
            :goal="g"
            :selected="goals.selectedGoalId === g.id"
            @select="goals.selectGoal(g.id)"
            @archive="goals.archiveGoal(g.id)"
          />
        </div>
      </div>

      <!-- 右栏:Hero卡 + 里程碑时间线 -->
      <div class="fl-goals-right">
        <!-- 目标 Hero 卡(对齐原型 milestones.html) -->
        <div v-if="selectedGoal" class="fl-goal-hero">
          <div class="fl-hero-tag">🎯 当前焦点</div>
          <button class="fl-hero-edit-btn" title="编辑目标" @click="startEditGoal">
            <Pencil :size="12" />
          </button>

          <!-- 编辑模式 -->
          <template v-if="editingGoal">
            <input v-model="editGoalName" class="fl-hero-input" type="text" maxlength="60" placeholder="目标名称" />
            <textarea v-model="editGoalDesc" class="fl-hero-input fl-hero-textarea" rows="2" placeholder="目标描述(可选)" maxlength="300" />
            <div class="fl-hero-edit-row">
              <label class="fl-hero-date-label">
                目标日期
                <input v-model="editGoalDate" class="fl-hero-input fl-hero-date" type="date" />
              </label>
              <div class="fl-hero-edit-btns">
                <button class="fl-hero-save" @click="saveGoalEdit">保存</button>
                <button class="fl-hero-cancel" @click="editingGoal = false">取消</button>
              </div>
            </div>
          </template>

          <!-- 显示模式 -->
          <template v-else>
            <h2 class="fl-hero-name">{{ selectedGoal.name }}</h2>
            <p v-if="selectedGoal.description" class="fl-hero-desc">{{ selectedGoal.description }}</p>
            <div class="fl-hero-meta">
              <span>📅 已进行 {{ daysSinceCreated }} 天</span>
              <span>🏁 里程碑 {{ goals.milestones.filter(m => m.status === 'completed').length }} / {{ goals.milestones.length }}</span>
              <span v-if="selectedGoal.target_date">🎯 预计 {{ selectedGoal.target_date }}</span>
            </div>
          </template>
        </div>
        <MilestoneTimeline />
      </div>
    </div>
  </section>
</template>

<style scoped>
.fl-goals {
  max-width: 960px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--sp-5);
}

.fl-page-head h1 {
  font-size: var(--fs-24);
  font-weight: var(--fw-semibold);
  margin: 0;
  color: var(--color-text-primary);
}
.fl-page-sub {
  margin: var(--sp-1) 0 0;
  color: var(--color-text-secondary);
  font-size: var(--fs-12);
}

.fl-goals-layout {
  display: grid;
  grid-template-columns: 280px 1fr;
  gap: var(--sp-5);
  min-height: 400px;
}

.fl-goals-left {
  display: flex;
  flex-direction: column;
  gap: var(--sp-3);
}

.fl-goal-form {
  display: flex;
  gap: var(--sp-2);
}
.fl-goal-input {
  flex: 1;
  padding: var(--sp-2) var(--sp-3);
  border-radius: var(--r-md);
  border: 1px solid var(--color-border);
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
  font-size: var(--fs-13, var(--fs-14));
  outline: none;
}
.fl-goal-input:focus {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}
.fl-goal-add {
  padding: var(--sp-2);
  border-radius: var(--r-md);
  border: none;
  background: var(--color-primary);
  color: var(--color-text-on-primary);
  cursor: pointer;
  display: grid;
  place-items: center;
}
.fl-goal-add:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.fl-goals-list {
  display: flex;
  flex-direction: column;
  gap: var(--sp-2);
}

.fl-goals-right {
  padding: var(--sp-4);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--r-lg);
}

.fl-goals-empty {
  text-align: center;
  padding: var(--sp-6);
  color: var(--color-text-muted);
  font-size: var(--fs-12);
}

/* Hero 卡 */
.fl-goal-hero {
  background: linear-gradient(180deg, var(--color-primary-soft) 0%, var(--color-bg-elevated) 100%);
  border: 1px solid color-mix(in srgb, var(--color-primary) 20%, var(--color-border));
  border-radius: var(--r-lg); padding: var(--sp-5);
  position: relative; overflow: hidden; margin-bottom: var(--sp-4);
}
.fl-goal-hero::before {
  content: ""; position: absolute; top: -60px; right: -60px;
  width: 200px; height: 200px;
  background: radial-gradient(circle, var(--color-primary-light), transparent 60%);
  opacity: 0.2; pointer-events: none;
}
.fl-hero-tag {
  font-size: var(--fs-12); color: var(--color-primary-dark); font-weight: var(--fw-medium);
  padding: 3px 10px; background: rgba(255,255,255,0.6); border-radius: var(--r-pill);
  display: inline-block; margin-bottom: var(--sp-3); position: relative;
}
.fl-hero-name { font-size: var(--fs-20, 20px); font-weight: var(--fw-semibold); margin: 0 0 var(--sp-2); position: relative; }
.fl-hero-desc { font-size: var(--fs-14); color: var(--color-text-secondary); margin: 0 0 var(--sp-3); position: relative; line-height: 1.6; }
.fl-hero-meta {
  display: flex; gap: var(--sp-4); font-size: var(--fs-12); color: var(--color-text-muted);
  position: relative;
}

/* Hero 编辑 */
.fl-hero-edit-btn {
  position: absolute; top: var(--sp-3); right: var(--sp-3);
  background: rgba(255,255,255,0.6); border: none; border-radius: var(--r-sm);
  color: var(--color-text-muted); cursor: pointer; padding: 4px 6px;
  opacity: 0; transition: opacity var(--dur-fast);
}
.fl-goal-hero:hover .fl-hero-edit-btn { opacity: 1; }
.fl-hero-edit-btn:hover { color: var(--color-primary); }

.fl-hero-input {
  width: 100%; padding: var(--sp-2) var(--sp-3);
  border: 1px solid var(--color-border); border-radius: var(--r-md);
  background: var(--color-bg-elevated); color: var(--color-text-primary);
  font-size: var(--fs-14); outline: none; position: relative;
}
.fl-hero-input:focus { border-color: var(--color-primary); }
.fl-hero-textarea { font-family: inherit; resize: vertical; font-size: var(--fs-12); margin-top: var(--sp-2); }
.fl-hero-edit-row { display: flex; align-items: flex-end; gap: var(--sp-3); margin-top: var(--sp-2); position: relative; }
.fl-hero-date-label { font-size: 11px; color: var(--color-text-muted); display: flex; flex-direction: column; gap: 4px; }
.fl-hero-date { width: 160px; font-size: var(--fs-12); }
.fl-hero-edit-btns { display: flex; gap: 6px; margin-left: auto; }
.fl-hero-save {
  padding: 4px 14px; border-radius: var(--r-md); border: none;
  background: var(--color-primary); color: #fff; font-size: 11px; cursor: pointer;
}
.fl-hero-cancel {
  padding: 4px 14px; border-radius: var(--r-md);
  border: 1px solid var(--color-border); background: transparent;
  color: var(--color-text-secondary); font-size: 11px; cursor: pointer;
}
</style>

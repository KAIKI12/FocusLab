# Global Demo Gap Overview Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把 FocusLab 从“功能基本具备”推进到“全局 demo 主流程闭环成熟”。

**Architecture:** 不再按页面补零散功能，而按 4 个产品闭环工程包推进：日计划闭环、任务智能闭环、实验功能产品化、桌面工程化收尾。每个工程包先用纯函数/Store 测试锁定行为，再接入 Vue UI 和 Tauri command，避免只做展示态。

**Tech Stack:** Vue 3 + Pinia + Vitest + Tauri v2 + Rust command + SQLite repository。

---

## Context

现有审计依据：

- `.tocodex/plans/2026-04-24-global-gap-audit.md`
- `plan/2026-04-21_12-37-46-prototype-gap-audit-v2.md`
- `prototype/index.html`
- `docs/02-功能设计文档.md`
- `docs/06-开发计划与风险管理.md`

当前判断：

- 原型级 P1 缺口基本清零。
- 最大 gap 不再是“有没有页面”，而是“闭环是否成熟”。
- 不建议优先做大 UI 改版、云同步、移动端、i18n。

---

## Milestone 1: 日计划闭环包

**Goal:** 打通 Morning Guide → 固定日程 → 今日任务编排 → 计划锁定 → 日结算回看。

**Primary Files:**

- Modify: `src/components/common/MorningGuide.vue`
- Modify: `src/views/TodayView.vue`
- Modify: `src/views/CalendarView.vue`
- Modify: `src/components/calendar/FixedSchedulePanel.vue`
- Modify: `src/components/calendar/AvailableTimePanel.vue`
- Modify: `src/stores/useAssignmentStore.ts`
- Modify: `src/stores/useTaskStore.ts`
- Modify: `src-tauri/src/commands/assignment_commands.rs`
- Modify: `src-tauri/src/models/assignment.rs`

### Task 1: Add daily plan summary model

**Files:**

- Modify: `src/types/index.ts`
- Test: `src/stores/useAssignmentStore.spec.ts`

**Step 1: Write the failing test**

Create or extend `src/stores/useAssignmentStore.spec.ts` with a test that expects a daily plan summary object:

```ts
import { describe, expect, it } from "vitest";

import type { DailyPlanSummary } from "@/types";

describe("DailyPlanSummary", () => {
  it("represents fixed schedule, planned tasks, and remaining capacity", () => {
    const summary: DailyPlanSummary = {
      date: "2026-04-24",
      fixedMinutes: 180,
      plannedTaskMinutes: 240,
      availableMinutes: 540,
      remainingMinutes: 120,
      locked: false,
    };

    expect(summary.remainingMinutes).toBe(120);
  });
});
```

**Step 2: Run test to verify it fails**

Run:

```bash
pnpm test src/stores/useAssignmentStore.spec.ts
```

Expected: FAIL because `DailyPlanSummary` does not exist.

**Step 3: Write minimal implementation**

Add to `src/types/index.ts`:

```ts
export interface DailyPlanSummary {
  date: string;
  fixedMinutes: number;
  plannedTaskMinutes: number;
  availableMinutes: number;
  remainingMinutes: number;
  locked: boolean;
}
```

**Step 4: Run test to verify it passes**

Run:

```bash
pnpm test src/stores/useAssignmentStore.spec.ts
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src/types/index.ts src/stores/useAssignmentStore.spec.ts
git commit -m "feat(plan): add daily plan summary type"
```

### Task 2: Derive daily plan summary in assignment store

**Files:**

- Modify: `src/stores/useAssignmentStore.ts`
- Test: `src/stores/useAssignmentStore.spec.ts`

**Step 1: Write the failing test**

Add a test that builds assignments/tasks/fixed minutes and expects remaining capacity:

```ts
it("calculates remaining minutes from available, fixed, and planned task minutes", () => {
  const availableMinutes = 540;
  const fixedMinutes = 120;
  const plannedTaskMinutes = 210;

  const remainingMinutes = availableMinutes - fixedMinutes - plannedTaskMinutes;

  expect(remainingMinutes).toBe(210);
});
```

If the store already has test helpers, use the real store action instead of inline math.

**Step 2: Run test to verify it fails**

Run:

```bash
pnpm test src/stores/useAssignmentStore.spec.ts
```

Expected: FAIL until the store exposes summary calculation.

**Step 3: Write minimal implementation**

Add a small pure helper in `src/stores/useAssignmentStore.ts`:

```ts
export function calculateDailyPlanSummary(args: {
  date: string;
  availableMinutes: number;
  fixedMinutes: number;
  plannedTaskMinutes: number;
  locked?: boolean;
}): DailyPlanSummary {
  return {
    date: args.date,
    availableMinutes: args.availableMinutes,
    fixedMinutes: args.fixedMinutes,
    plannedTaskMinutes: args.plannedTaskMinutes,
    remainingMinutes: Math.max(0, args.availableMinutes - args.fixedMinutes - args.plannedTaskMinutes),
    locked: args.locked ?? false,
  };
}
```

**Step 4: Run test to verify it passes**

Run:

```bash
pnpm test src/stores/useAssignmentStore.spec.ts
```

Expected: PASS.

**Step 5: Commit**

```bash
git add src/stores/useAssignmentStore.ts src/stores/useAssignmentStore.spec.ts
git commit -m "feat(plan): calculate daily plan capacity"
```

### Task 3: Show daily plan summary in TodayView

**Files:**

- Modify: `src/views/TodayView.vue`
- Test: `src/views/TodayView.spec.ts`

**Step 1: Write the failing test**

Add a component test that expects visible summary labels:

```ts
it("renders daily plan capacity summary", async () => {
  const wrapper = mount(TodayView, { global: { plugins: [createTestingPinia()] } });

  expect(wrapper.text()).toContain("今日容量");
  expect(wrapper.text()).toContain("剩余");
});
```

**Step 2: Run test to verify it fails**

Run:

```bash
pnpm test src/views/TodayView.spec.ts
```

Expected: FAIL because the summary block is absent.

**Step 3: Write minimal implementation**

Add a compact summary card near the existing Today header in `TodayView.vue`:

```vue
<div class="fl-day-capacity-card">
  <span>今日容量</span>
  <strong>{{ dailySummary.availableMinutes }}m</strong>
  <span>剩余 {{ dailySummary.remainingMinutes }}m</span>
</div>
```

Use the helper from `useAssignmentStore.ts` and existing fixed schedule / task data.

**Step 4: Run tests**

```bash
pnpm test src/views/TodayView.spec.ts
pnpm type-check
```

Expected: PASS and type-check 0 errors.

**Step 5: Commit**

```bash
git add src/views/TodayView.vue src/views/TodayView.spec.ts
git commit -m "feat(today): show daily plan capacity"
```

### Task 4: Add plan lock state

**Files:**

- Modify: `src-tauri/src/models/assignment.rs`
- Modify: `src-tauri/src/commands/assignment_commands.rs`
- Modify: `src/stores/useAssignmentStore.ts`
- Test: Rust command tests if available, otherwise store tests

**Step 1: Write failing test**

Add a test that toggles a plan lock for a date and expects the store to expose `locked: true`.

**Step 2: Run test**

```bash
pnpm test src/stores/useAssignmentStore.spec.ts
```

Expected: FAIL.

**Step 3: Implement minimal persistence**

Prefer an existing key-value setting table if present. Store key:

```text
daily_plan_locked_YYYY-MM-DD = 1
```

Expose commands:

```rust
set_daily_plan_locked(date: String, locked: bool)
get_daily_plan_locked(date: String) -> bool
```

**Step 4: Wire UI**

Add a lock/unlock button in `TodayView.vue`.

**Step 5: Run checks**

```bash
pnpm test src/stores/useAssignmentStore.spec.ts
pnpm type-check
cargo test
```

Expected: all pass.

**Step 6: Commit**

```bash
git add src-tauri/src src/stores/useAssignmentStore.ts src/views/TodayView.vue
git commit -m "feat(plan): persist daily plan lock"
```

---

## Milestone 2: 任务智能闭环包

**Goal:** 打通重复任务、微复盘、预估偏差，让任务系统能根据历史反馈变聪明。

**Primary Files:**

- Modify: `src/components/task/MicroReview.vue`
- Modify: `src/composables/useMicroReviewScenario.ts`
- Modify: `src/views/TasksView.vue` or existing task editor component
- Modify: `src-tauri/src/commands/task_commands.rs`
- Modify: `src-tauri/src/models/task.rs`
- Modify: `src-tauri/src/commands/stats_commands.rs`

### Task 5: Persist structured micro-review result

**Files:**

- Modify: `src/types/index.ts`
- Modify: `src/components/task/MicroReview.vue`
- Modify: `src-tauri/src/commands/task_commands.rs`
- Test: `src/composables/useMicroReviewScenario.spec.ts`

**Step 1: Write failing test**

Expect a review payload with reason and confidence:

```ts
it("builds a structured review payload", () => {
  const payload = {
    taskId: "task-1",
    sessionId: "session-1",
    scenario: "deviation",
    reason: "任务被会议打断",
    nextEstimateMinutes: 45,
  };

  expect(payload.scenario).toBe("deviation");
});
```

**Step 2: Run test**

```bash
pnpm test src/composables/useMicroReviewScenario.spec.ts
```

Expected: FAIL until type/helper exists.

**Step 3: Implement minimal model and command**

Add `MicroReviewResult` type and a Tauri command that saves the record.

**Step 4: Wire MicroReview.vue submit**

Call the command on submit; do not silently swallow errors.

**Step 5: Run checks**

```bash
pnpm test src/composables/useMicroReviewScenario.spec.ts
pnpm type-check
cargo test
```

Expected: all pass.

**Step 6: Commit**

```bash
git add src/types/index.ts src/components/task/MicroReview.vue src-tauri/src
git commit -m "feat(review): persist structured micro reviews"
```

### Task 6: Add estimate accuracy summary

**Files:**

- Modify: `src-tauri/src/commands/stats_commands.rs`
- Modify: `src/views/StatsView.vue`
- Test: frontend stats test or Rust stats test

**Step 1: Write failing test**

Expect average deviation by task category or recent sessions.

**Step 2: Run test**

```bash
cargo test estimate_accuracy
```

Expected: FAIL.

**Step 3: Implement query**

Return:

```ts
interface EstimateAccuracySummary {
  averageDeviationPct: number;
  sampleSize: number;
  overrunCount: number;
  underrunCount: number;
}
```

**Step 4: Render compact card in StatsView**

Show average deviation and sample size.

**Step 5: Run checks**

```bash
pnpm type-check
cargo test
```

Expected: all pass.

**Step 6: Commit**

```bash
git add src/views/StatsView.vue src-tauri/src/commands/stats_commands.rs
git commit -m "feat(stats): add estimate accuracy summary"
```

### Task 7: Build recurring task editor

**Files:**

- Modify: task create/edit component currently used by `TodayView.vue`
- Modify: `src-tauri/src/commands/task_commands.rs`
- Test: task editor component test

**Step 1: Write failing test**

Mount the editor and expect recurrence controls:

```ts
expect(wrapper.text()).toContain("重复");
expect(wrapper.text()).toContain("每天");
expect(wrapper.text()).toContain("每周");
```

**Step 2: Run test**

```bash
pnpm test src/components/task/TaskEditor.spec.ts
```

Expected: FAIL.

**Step 3: Implement minimal UI**

Add recurrence rule fields:

- none
- daily
- weekly
- monthly

**Step 4: Save to existing backend field**

Use existing `recurrence_rule` if present.

**Step 5: Run checks**

```bash
pnpm test src/components/task/TaskEditor.spec.ts
pnpm type-check
```

Expected: all pass.

**Step 6: Commit**

```bash
git add src/components/task src-tauri/src/commands/task_commands.rs
git commit -m "feat(tasks): add recurring task editor"
```

---

## Milestone 3: 实验功能产品化包

**Goal:** 让 Persona / Badges / Mood 从展示型功能变成可持续反馈系统。

**Primary Files:**

- Modify: `src/views/PersonaView.vue`
- Modify: `src/views/BadgesView.vue`
- Modify: `src/components/common/MoodCheck.vue`
- Modify: `src/views/StatsView.vue`
- Modify: `src-tauri/src/commands/stats_commands.rs`

### Task 8: Extract persona behavior scoring

**Files:**

- Create: `src/composables/usePersonaScoring.ts`
- Test: `src/composables/usePersonaScoring.spec.ts`
- Modify: `src/views/PersonaView.vue`

**Step 1: Write failing test**

```ts
it("scores a focused planner persona from stable completion and planning data", () => {
  const result = scorePersona({ completionRate: 0.85, morningGuideDays: 5, deepWorkMinutes: 600 });

  expect(result.primaryTrait).toBe("planner");
});
```

**Step 2: Run test**

```bash
pnpm test src/composables/usePersonaScoring.spec.ts
```

Expected: FAIL.

**Step 3: Implement pure scoring helper**

Keep it deterministic and small. No AI call in this task.

**Step 4: Wire PersonaView**

Use scoring helper for visible progress hints.

**Step 5: Run checks**

```bash
pnpm test src/composables/usePersonaScoring.spec.ts
pnpm type-check
```

Expected: all pass.

**Step 6: Commit**

```bash
git add src/composables/usePersonaScoring.ts src/composables/usePersonaScoring.spec.ts src/views/PersonaView.vue
git commit -m "feat(persona): score behavior traits"
```

### Task 9: Improve badge unlock coverage report

**Files:**

- Modify: `src/views/BadgesView.vue`
- Test: badge logic test if present, otherwise create `src/views/BadgesView.spec.ts`

**Step 1: Write failing test**

Expect locked badges with missing data source to be counted separately.

**Step 2: Run test**

```bash
pnpm test src/views/BadgesView.spec.ts
```

Expected: FAIL.

**Step 3: Implement coverage summary**

Show:

- unlocked
- unlockable with current data
- waiting for data source

**Step 4: Run checks**

```bash
pnpm test src/views/BadgesView.spec.ts
pnpm type-check
```

Expected: all pass.

**Step 5: Commit**

```bash
git add src/views/BadgesView.vue src/views/BadgesView.spec.ts
git commit -m "feat(badges): show unlock coverage"
```

### Task 10: Feed mood signals into stats

**Files:**

- Modify: `src/components/common/MoodCheck.vue`
- Modify: `src/views/StatsView.vue`
- Modify: `src-tauri/src/commands/stats_commands.rs`
- Test: stats command or frontend helper test

**Step 1: Write failing test**

Expect mood summary to include morning and evening signal counts.

**Step 2: Run test**

```bash
cargo test mood_summary
```

Expected: FAIL.

**Step 3: Implement minimal stats command**

Return last 7 days mood distribution.

**Step 4: Render StatsView card**

Show “状态信号” card near existing insight sections.

**Step 5: Run checks**

```bash
pnpm type-check
cargo test
```

Expected: all pass.

**Step 6: Commit**

```bash
git add src/components/common/MoodCheck.vue src/views/StatsView.vue src-tauri/src/commands/stats_commands.rs
git commit -m "feat(stats): add mood signal summary"
```

---

## Milestone 4: 桌面工程化收尾包

**Goal:** 补齐 demo/内测所需的桌面级可信度：通知、快捷键、导入恢复、运行时验证。

**Primary Files:**

- Modify: `src-tauri/src/system/tray.rs`
- Modify: `src-tauri/src/commands/window_commands.rs`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src/views/SettingsView.vue`
- Modify: `src-tauri/src/commands/export_commands.rs`
- Modify: `src-tauri/src/commands/recovery_commands.rs`

### Task 11: Add desktop capability audit checklist inside Settings

**Files:**

- Modify: `src/views/SettingsView.vue`
- Test: `src/views/SettingsView.spec.ts`

**Step 1: Write failing test**

Expect Settings to show desktop capability states:

```ts
expect(wrapper.text()).toContain("桌面能力");
expect(wrapper.text()).toContain("系统托盘");
expect(wrapper.text()).toContain("快捷键");
```

**Step 2: Run test**

```bash
pnpm test src/views/SettingsView.spec.ts
```

Expected: FAIL.

**Step 3: Implement read-only checklist UI**

Show current state first. Do not implement full shortcut customization in this task.

**Step 4: Run checks**

```bash
pnpm test src/views/SettingsView.spec.ts
pnpm type-check
```

Expected: all pass.

**Step 5: Commit**

```bash
git add src/views/SettingsView.vue src/views/SettingsView.spec.ts
git commit -m "feat(settings): show desktop capability status"
```

### Task 12: Verify floating bubble transparency on Windows 11

**Files:**

- Modify only if runtime verification proves a code issue:
  - `src-tauri/tauri.conf.json`
  - `src-tauri/src/commands/window_commands.rs`
  - `src/views/BubbleView.vue`

**Step 1: Manual verification**

Ask the user to run:

```bash
pnpm tauri dev
```

Expected: App launches with main window and bubble window available.

**Step 2: Check transparency behavior**

Verify:

- bubble background is transparent
- no unexpected white/black rectangle
- shadow does not obscure content
- click/drag still works

**Step 3: Only if failed, write minimal fix**

Possible config direction:

```json
{
  "transparent": true,
  "decorations": false,
  "shadow": false
}
```

Apply only to bubble window creation, not the main window.

**Step 4: Run checks**

```bash
pnpm type-check
cargo test
```

Expected: all pass.

**Step 5: Commit if code changed**

```bash
git add src-tauri/src src-tauri/tauri.conf.json src/views/BubbleView.vue
git commit -m "fix(bubble): correct window transparency"
```

---

## Recommended Execution Order

1. Milestone 1 / Task 1-4: 日计划闭环包
2. Milestone 2 / Task 5-7: 任务智能闭环包
3. Milestone 3 / Task 8-10: 实验功能产品化包
4. Milestone 4 / Task 11-12: 桌面工程化收尾包

原因：日计划是产品主价值链；任务智能闭环能强化长期使用价值；实验功能适合增强 demo 差异化；桌面工程化适合在内测前集中收口。

---

## Verification Before Completion

Run after each milestone:

```bash
pnpm test
pnpm type-check
cargo test
```

For UI tasks, also manually verify the affected page in the running Tauri app. Do not claim UI completion if it was not opened and checked.

---

## Out of Scope

- Cloud sync
- Mobile app
- Social features
- Full i18n rollout
- Large visual redesign
- New AI scenario expansion unrelated to existing flows

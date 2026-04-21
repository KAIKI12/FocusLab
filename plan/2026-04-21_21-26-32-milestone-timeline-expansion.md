---
mode: plan
cwd: C:\Users\zhx\Desktop\FocusLab
task: MilestoneTimeline 补 5 项 feature(gap-audit v2 §P1.1)
complexity: complex
tool: plan
total_thoughts: 7
created_at: 2026-04-21_21-26-32
baseline_head: 2f17ca3
reference_audit: plan/2026-04-21_12-37-46-prototype-gap-audit-v2.md
---

# Plan: MilestoneTimeline v2 扩展(5 feature)

## 🎯 Task Overview

gap-audit v2 §P1.1 是目前最后一个 P1 缺口 · 原型 `prototype/goals/milestones.html` 663 行 vs 实现 `MilestoneTimeline.vue` 329 行,缺 5 项 feature:子任务列表 / 今日关联 / 备注列表 / 预计完成日期 / 本周投入柱图。本 plan 拆 schema → 后端 → 前端三层,**等用户决策后再动手**。

## 🔑 关键决策点(等用户确认)

### D1 · 子任务数据源:复用 `tasks` 表 vs 新增 `milestone_subtasks` 表

| 方案 | 优 | 劣 |
|---|---|---|
| **A · 复用 tasks 表(推荐)** | 零 schema 改动;tasks.milestone_id 已有;子任务能复用 Pomodoro / 四象限 / AI 分解 | 需要 UI 语义区分"里程碑子任务"(通常 Q1/Q2)和"一次性任务";原型里"1 天 · 还剩 ~1 天"这种 meta 映射成 `estimated_minutes` 换算单位 |
| B · 新增独立表 `milestone_subtasks(id, milestone_id, title, status, estimated_days, sort_order, ...)` | 语义干净 | 另起一套 CRUD;和 tasks 表的"可专注"能力分裂 |

**推荐 A** · 子任务 = 该 milestone 下 `tasks WHERE milestone_id = ?`,无新表。"预计天数"前端显示 `estimated_minutes` 换算。

### D2 · 里程碑备注:独立表 vs JSON 塞 `description`

| 方案 | 优 | 劣 |
|---|---|---|
| **A · 新增 `milestone_notes(id, milestone_id, text, created_at)`(推荐)** | 按日期排序 · 可单条删除 · 符合原型 3 条带日期笔记 | 新增一张表 + CRUD |
| B · `description` 存 JSON 数组 | 零 schema 改动 | 编辑 / 单条删除麻烦;`description` 原本语义被污染 |

**推荐 A**。

### D3 · `target_date` 列:加到 `milestones` 表

schema 改动:`ALTER TABLE milestones ADD COLUMN target_date DATE`。无选项,必做。

### D4 · 周投入图数据:前端拉 sessions 聚合 vs 后端新 command

| 方案 | 优 | 劣 |
|---|---|---|
| **A · 后端新 `get_goal_weekly_invest(goal_id)` 一次返回 7 桶(推荐)** | 单次 IPC · SQL 聚合高效 | 新增 command |
| B · 前端复用 list_sessions_in_range 再 groupBy | 复用已有 API | 7 次 filter + 换算 weekday,组件重 |

**推荐 A** · 返回 `{ weekday: 0-6, minutes: number }[]` + 总 minutes。

### D5 · UI 布局:展开主激活卡 vs 全列表展开

原型是"主激活里程碑卡全展开(子任务 + 今日关联 + 备注 + 周投入图) + 其他里程碑紧凑列表"。

**推荐** · 沿用原型:第一个 `status !== 'completed'` 的 milestone 卡默认展开,其他点击切换。避免全展开造成纵向爆炸。

---

## 📋 Execution Plan

**分 3 个阶段 · 每阶段一个独立 commit · 用户可在任一阶段后暂停检视。**

### Phase 1 · Schema + 后端(预计 3-4h)

1. `003_milestone_v2.sql` migration:
   - `ALTER TABLE milestones ADD COLUMN target_date DATE`
   - `CREATE TABLE milestone_notes (id, milestone_id FK, text, created_at, UNIQUE ...)`
2. `src-tauri/src/models/milestone.rs`:`target_date` 字段 + CRUD 扩展
3. `src-tauri/src/models/milestone_note.rs`(新):list / add / delete
4. `src-tauri/src/commands/goal_commands.rs` 暴露新 command:
   - `set_milestone_target_date(milestone_id, target_date)`
   - `list_milestone_notes(milestone_id)` / `add_milestone_note` / `delete_milestone_note`
   - `get_goal_weekly_invest(goal_id)` 返回 7 桶 + 今日已投入
5. 单元测试:每个新 command 至少 1 个 happy-path + 1 个边界

### Phase 2 · Store + Composable(预计 1-2h)

1. `src/types.ts`:`Milestone.target_date: string | null`;新增 `MilestoneNote` / `WeeklyInvest` 类型
2. `useGoalStore.ts` 扩展:
   - `notesByMilestone: Map<milestoneId, MilestoneNote[]>`
   - `weeklyInvest: WeeklyInvest | null`(响应 selectedGoalId 变化)
   - `addNote` / `removeNote` / `setTargetDate` 动作
3. `src/composables/useMilestoneSubtasks.ts`(新):从 useTaskStore 派生 `subtasksByMilestone` 计算 + 今日关联 task

### Phase 3 · UI(预计 4-5h)

1. `MilestoneTimeline.vue` 拆分:
   - 保留当前"紧凑行"作为非激活状态
   - 新组件 `MilestoneExpandedCard.vue`:展开卡 · 两栏布局(左子任务 / 右今日+备注+日期),下面周投入图
2. 子组件:
   - `MilestoneSubtaskList.vue` · 复用 TaskStore,点击可启动 Pomodoro
   - `MilestoneNotesList.vue` · 读写 notes · `@时间戳` 标记风格
   - `MilestoneTargetDate.vue` · 日期选择器 + 剩余天数计算
   - `GoalWeeklyInvestChart.vue` · 7 柱 bar,SVG 或 div + height%,当日高亮
3. 样式对齐 `prototype/goals/milestones.html`(sub-task-row / today-mapped / invest-bar 等 class)
4. Vue-tsc 0 错误 + 跑一遍手动 smoke check(加 milestone → 加子任务 → 改日期 → 加笔记 → 看图)

---

## 🧠 当前思考摘要

- 5 项 feature 看似并列,实际**依赖倒金字塔**:周投入图和子任务展示都依赖子任务数据源决策(D1)· 备注需要新表(D2)· target_date 独立(D3)。先定 D1/D2/D3 再动手,避免后期返工。
- **复用 tasks 表**是关键节省项 · 子任务能直接起番茄,这是 FocusLab 的核心价值;独立 `milestone_subtasks` 会让"子任务可专注"变成二等公民。
- Phase 1 和 2 无 UI 变动 · commit 后用户看不到效果,需要等 Phase 3 才能手动验证。可考虑先做 Phase 3 的"空组件骨架"再回填数据,但会割裂提交语义,不推荐。

## ⚠️ 风险和阻塞

- **Schema migration 无回滚** · 本项目 migrations 是累加执行,一旦 003 上线就不可撤。加列 + 建表都是非破坏性,相对安全,但需要用户确认后再合并。
- **tasks 表耦合** · 复用 tasks 当子任务后,如果用户把 milestone 删除,tasks.milestone_id 会成悬挂引用 — 需要检查现有 ON DELETE 策略(001_init.sql `tasks.milestone_id` 只 REFERENCES 不带 CASCADE,删里程碑时 task 会保留,OK)。
- **周投入图数据量** · 单周最多几十个 session,SQL 聚合无性能问题。
- **UI 信息密度** · 原型展开卡信息密度大,窄屏可能溢出;先按原型尺寸做,窄屏 fallback 后续迭代。

## 📎 References

- 原型: `prototype/goals/milestones.html:244`(子任务) · `:535`(今日) · `:546`(备注) · `:556`(日期) · `:566`(图)
- 当前实现: `src/components/goal/MilestoneTimeline.vue:89-127`
- Schema: `src-tauri/src/db/migrations/001_init.sql:17-28`(milestones) · `:84-97`(sessions)
- Gap audit: `plan/2026-04-21_12-37-46-prototype-gap-audit-v2.md:58`
- Milestone 模型: `src-tauri/src/models/milestone.rs`

---
mode: audit
cwd: C:\Users\zhx\Desktop\FocusLab
task: prototype/ 19 屏 vs src 实现 差距盘点 (v2)
complexity: medium
scope: 只盘点不实施,用户后续决定补齐顺序
ref_prototype_root: prototype/
ref_view_root: src/views/ + src/components/
baseline_head: 25aff75
previous_audit: plan/2026-04-20_00-15-25-prototype-gap-audit.md
created_at: 2026-04-21_12-37-46
---

# Audit v2: 原型 vs 实现 差距清单

## 🎯 背景

距上次 audit (2026-04-20 · HEAD `e9b0b72`) 14 个 commit 之后,再做一次盘点。上次 P0/P1/P2 共 9 条基本清零,本次重点挖被跳过的屏(Milestones / Settings / MorningGuide / MicroReview 等)。

**盘点基线:**
- HEAD `25aff75 chore(assets): 人格图鉴 44 张图入库`
- prototype/ 共 19 屏 (Batch 1+2+3+4 = 5+5+5+4,主题预览在 prototype 外)

**盘点原则:**
- 只列"原型有但实现缺/弱化/变形"的点,已一致的不重复
- 上次已列、仍未修复的条目标记「🔁 承袭」
- 已修复的条目标记「✅ 清零」放在卷首

---

## ✅ 上次 audit 清零状况

14 commit 期间完成的修复:

| 上次条目 | 承接 commit |
|---|---|
| P0 · TodayView DDL 条合并 + 右栏合并 | `8a7af62` |
| P0 · ParkedView 拆分按钮 + D2/D3 + 语言原则 | `8a7af62` |
| P0 · AI Payload Modal | `8a7af62` |
| P1 · CalendarView 固定日程/可用时间/周末高亮 | `b20b126` |
| P1 · PersonaView hatchDay 真实天数 | `0462c90` |
| P1 · BadgesView 解锁引擎 | `afcaeec` + `598f813` |
| P2 · PomodoroView 模式对比面板 | `e97a288` |
| P2 · StatsView AI 周度小结 | 已在 `StatsView.vue:18,168` 接入 |

**仍承袭未做:** P1 BubbleView 托盘菜单 (需 Tauri tray plugin,Phase 3)。

---

## 🔴 P0 · 破损 / 急缺

**无新增 P0。** 主链路 (Today → Pomodoro → Settlement) 与核心弹窗在本轮 commit 后均可用。

---

## 🟡 P1 · 功能偏差

### 1. MilestoneTimeline · 缺 5 项关键 feature (`src/components/goal/MilestoneTimeline.vue`)

原型 `goals/milestones.html` 663 行 vs 实现 329 行,是当前偏差最大的视图。

| 差距 | 原型出处 | 实现现状 |
|---|---|---|
| **子任务列表缺失** | `milestones.html:244` `.sub-task-row` 里程碑下可展开子任务 | 仅有里程碑名 + description 单行 |
| **今日关联任务缺失** | `milestones.html:541` `.today-mapped` "今天正在推进 · [任务] 已专注 Xh Ym" | 无 |
| **里程碑备注列表缺失** | `milestones.html:546-554` 带日期的多条笔记 | description 单字段 |
| **预计完成日期缺失** | `milestones.html:556-561` "剩余约 13 天" | Milestone 可能无 target_date 字段 |
| **本周时间投入柱状图** | `milestones.html:566-569` `.invest-chart` | 无 |

### 2. MorningGuide Step 3 · 固定日程未接入 (`src/components/common/MorningGuide.vue`)

| 差距 | 现状 |
|---|---|
| Step 3 显示"固定日程功能开发中,可直接跳过" | `MorningGuide.vue:136-142` 硬编码占位 |
| 但 CalendarView 已在 `b20b126` 落地 `FixedSchedulePanel` | 两端未连接 · 数据源已有,只差读取 |

### 3. MicroReview · 仅覆盖 1/4 场景 (`src/components/task/MicroReview.vue`)

| 场景 | 原型 | 实现 |
|---|---|---|
| 时间偏差 > 30% | ✅ `micro-review.html:284` | ✅ `MicroReview.vue:52` |
| 关闭 Q1 任务 | ✅ `micro-review.html` scenario 2 | ❌ |
| 里程碑达成 | ✅ scenario 3 | ❌ |
| 静默触发 (连续 3 天未完成同一任务) | ✅ scenario 4 | ❌ |

### 4. BubbleView · 托盘菜单 🔁 承袭

`floating-ball.html:289` `.tray-menu` 未实现。需引入 `tauri-plugin-positioner` + `tauri-plugin-tray` (或 `TrayIcon` 原生 API)。Phase 3 已标。

---

## 🟢 P2 · 细节 / 可延后

### 5. SettingsView · 缺「自由模式偏好」分区 (`src/views/SettingsView.vue`)

原型 `settings.html:883-950` 在 POMODORO 分区底部补了 v1.2.3 的 4 项自由模式设置:

| 字段 | 原型行 | 用途 |
|---|---|---|
| 最小计数时长 | 903 | 短于此的 free 会话不计入专注 (避免误触) |
| 最长提醒 | 918 | 连续 free 超 X 分呼吸提示 |
| 暂停行为 | 936 | 暂停 = 结束 or 仅暂停可恢复 |
| 切换任务行为 | 946 | 切任务前先结束当前会话 |

实现 `SettingsView.vue:29` 有 POMODORO tab,但只有 focus / shortBreak / longBreakInterval 3 项,没 free 子组。

### 6. BubbleView · Windows 11 透明度 🔁 承袭

需真机验证,非代码层问题。

---

## 🧭 设计合并决策 (非缺失,记录在案)

| 原型屏 | 实现合并点 | 评价 |
|---|---|---|
| `main-today-lite.html` 紧凑变体 | 未做独立 View | 低优先,TodayView 已够用 |
| `FocusLab-主题预览.html` (prototype 外) | SettingsView 外观分区已有 12 主题卡 | 合理 |
| `persona-hatch.html` 独立页 | PersonaView 内嵌 fl-hatch 模块 | 承袭上次决策 |

---

## 📌 推荐补齐顺序

按 "用户可见度 × 实现成本" 排:

1. **MorningGuide Step 3 接入 FixedSchedule** (P1 · 1-2h · 数据源已有)
2. **MicroReview 补 Q1/里程碑/静默 3 场景** (P1 · 2-3h · 需触发时机判定,不涉及 schema)
3. **MilestoneTimeline 补 5 项 feature** (P1 · 8-12h · 需扩 schema: subtasks + notes + target_date,工作量最大)
4. **SettingsView 补自由模式偏好** (P2 · 1-2h · 纯 UI + setting key)
5. **BubbleView 托盘菜单** (P1 · 3-5h · 需 Tauri plugin,留 Phase 3)

**建议按 1 → 2 → 4 → 3 → 5 顺序执行**:先收拾小而独立的 (#1 #2 #4),再啃 schema 扩展 (#3),最后 Phase 3 依赖项 (#5)。

---

## 📎 参考

- 基线 commit: `25aff75 chore(assets): 人格图鉴 44 张图入库 + 旧图清理`
- 原型根目录: `prototype/` (index.html 列 Batch 1-4 共 19 屏)
- 上次 audit: `plan/2026-04-20_00-15-25-prototype-gap-audit.md`
- 相关 ADR: ADR-009 (社交实验隔离) · ADR-010 (右栏 Daily 卡合并) · ADR-011 (人格图鉴)
- 追踪 memory: `project_bubble_issues.md` · `project_prototype_gap_audit.md`

---

## 🟢 进度追加 · 2026-04-21 晚

本次会话按推荐顺序执行 1 → 2 → 4,共 3 个 commit 清零 3 条:

| 条目 | 状态 | commit |
|---|---|---|
| §P1.2 MorningGuide Step 3 接 FixedSchedule | ✅ 清零 | `58c6dba` |
| §P1.3 MicroReview 补 Q1/milestone/静默 3 场景 + PomodoroView 路由 | ✅ 清零 | `d834bb4` |
| §P2.5 SettingsView 自由模式偏好 4 项(6 setting keys 已持久化) | ✅ 清零 | `9008e59` |

**剩余(v2 盘点范围内):**
- §P1.1 MilestoneTimeline 补 5 项 feature — 需扩 schema(subtasks + notes + target_date),8-12h,未启动
- §P1.4 BubbleView 托盘菜单 🔁 承袭 — Phase 3 依赖 Tauri tray plugin
- §P2.6 BubbleView Windows 11 透明度 🔁 承袭 — 需真机验证

**type-check:** `npx vue-tsc --noEmit` 0 错误。

---

## 🟢 进度追加 · 2026-04-21 深夜 · MilestoneTimeline v2 全量落地

按 `plan/2026-04-21_21-26-32-milestone-timeline-expansion.md` 三阶段执行,4 个 commit:

| Phase | 内容 | commit |
|---|---|---|
| 1 · backend | 003 migration(ALTER target_date + milestone_notes 表) · milestone_note 模块 · set_target_date + notes CRUD + get_goal_weekly_invest 共 5 新 command · cargo test 46/46 绿 | `6aefbe2` |
| 2 · store | types.ts 加 3 新类型 · useGoalStore 扩 notes/weeklyInvest/setTargetDate · 新 composable useMilestoneSubtasks | `3d13f09` |
| 3 · UI | 3 新子组件(Subtasks / InfoPanel / WeeklyInvestChart) · MilestoneTimeline 重构为"紧凑列表 + 激活展开" · vue-tsc 0 错 | `e9fe3e3` |

**清零结果(v2 盘点范围内):**
- §P1.1 MilestoneTimeline 5 feature ✅ 清零
- §P1.4 / §P2.6 BubbleView tray / 透明度 🔁 承袭到 Phase 3

**P1 至此除 Bubble 承袭项外全部清零。** 下一轮重点可转 Phase 3 Tauri tray plugin 或用户指定新方向。

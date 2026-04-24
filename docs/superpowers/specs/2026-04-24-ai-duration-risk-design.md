---
created: 2026-04-24
reason: AI 能力接入层第二批 — 任务预估时长建议（A）+ 里程碑到期风险预警（B）
---

# FocusLab · AI 能力接入层第二批设计文档

> 版本: v1.0  
> 创建: 2026-04-24  
> 对应 AI 能力分层: L2（任务预估时长）+ L3（风险预警）

---

## 1. 背景与目标

延续上一批 AI 接入工作（未完成提醒 / 完成正反馈 / 里程碑拆解），本批次聚焦两个高价值场景：

| 功能 | 价值 | 触发点 |
|------|------|--------|
| A. 任务预估时长建议 | 帮助用户在创建/编辑任务时填写合理的时长预估，基于历史数据降低估算偏差 | TaskEditModal — 预估时长字段旁 |
| B. 里程碑到期风险预警 | 在临近截止日且完成率偏低时主动提示 AI 风险分析，提供补救行动建议 | MilestoneInfoPanel — 截止日旁 |

---

## 2. 功能 A：任务预估时长 AI 建议

### 2.1 触发条件

用户在 `TaskEditModal.vue` 的"预估(分钟)"字段旁看到一个 `✨` 图标按钮，**随时可点击**（不要求字段为空），点击后异步调用 AI。

### 2.2 后端数据准备

在 `ai_commands.rs` 中新增命令 `ai_estimate_task_duration`，接收：

```rust
pub struct EstimateDurationInput {
    pub task_name: String,
    pub description: Option<String>,
    pub quadrant: Option<String>,
}
```

后端在调用 AI 前先查询历史相似任务平均用时：

```sql
SELECT AVG(s.actual_duration_minutes), COUNT(*)
FROM sessions s
JOIN tasks t ON t.id = s.task_id
WHERE t.name LIKE '%{关键词}%'
  AND s.status = 'completed'
  AND s.actual_duration_minutes > 0
LIMIT 20
```

关键词提取规则：取任务名的前 4 个汉字或前 8 个字符（优先汉字）。无结果时传空字符串 `[]`。

### 2.3 Prompt 模板

在 `prompt_templates.rs` 中新增 `task_duration_prompt()`，使用 `task-duration-estimation.json` 中的 `precise` 变体：

```
输入：任务名 / 描述 / 历史相似任务平均用时（无则空） / 象限
输出：{ estimated_minutes, confidence, reasoning, range: { min, max } }
```

- `estimated_minutes` 必须是 15 的倍数，范围 15-480
- `confidence`: `high` / `medium` / `low`
- `reasoning` ≤ 30 字

### 2.4 前端展示

在 [`TaskEditModal.vue`](src/components/task/TaskEditModal.vue:134) 的预估字段行：

```
预估(分钟)  [____]  [✨]
            AI 估算 · 中等置信度 · 参考历史同类任务平均 45min（±15min）
```

- 点击 `✨` 按钮 → loading 状态（按钮禁用）
- AI 返回后自动写入 `estimatedMinutes` 字段
- 在字段下方渲染一行小字提示（含 reasoning + range）
- 用户可手动覆盖字段值
- AI 调用失败时小字显示"AI 估算失败，请手动填写"（不报错、不弹框）

### 2.5 API 消耗

- `temperature: 0.3`，`max_tokens: 100`（固定，不受 intensity 影响）
- 分类：L2，低消耗

---

## 3. 功能 B：里程碑到期风险预警

### 3.1 触发条件（前端判断）

在 [`MilestoneInfoPanel.vue`](src/components/goal/MilestoneInfoPanel.vue:33) 中，当同时满足以下两个条件时，在截止日期行旁显示 `⚠️ 风险分析` 按钮：

```
remainingDays !== null
&& remainingDays >= 0        // 未过期（过期不显示，显示意义不大）
&& remainingDays <= 5        // 5天内
&& completionRate < 0.5      // 完成率 < 50%
```

`completionRate` = `doneCount / totalCount`（子任务完成数/总数），从 `useMilestoneSubtasks` 的 `progressOf()` 获取。若无子任务（totalCount = 0）则 completionRate 视为 0。

### 3.2 后端命令

新增 `ai_milestone_risk`，接收：

```rust
pub struct MilestoneRiskInput {
    pub milestone_name: String,
    pub goal_name: Option<String>,
    pub target_date: String,          // ISO date
    pub remaining_days: i32,
    pub done_subtasks: i32,
    pub total_subtasks: i32,
    pub recent_activity: Option<String>, // 最近7天专注分钟数，可选
}
```

### 3.3 Prompt 模板

新增 `milestone_risk_prompt()`，输出结构：

```json
{
  "risk_level": "high | medium | low",
  "summary": "≤50字，描述当前风险状况",
  "actions": ["建议行动1", "建议行动2", "建议行动3"]
}
```

约束：
- `risk_level` 严格枚举，不允许其他值
- `summary` ≤ 50 字，不使用"失败""放弃"等词
- `actions` 数组 2-4 项，每项 ≤ 25 字，具体可执行

### 3.4 前端展示

在 `MilestoneInfoPanel.vue` 截止日期行旁：

```
预计完成  [2026-04-28]  ⚠️ 风险分析
          剩余 3 天
```

点击"⚠️ 风险分析"后，在面板下方渲染风险卡片：

```
┌─────────────────────────────────────┐
│ ⚠️ AI 风险提示               高风险  │
│ 距截止仅剩3天，子任务完成率25%，     │
│ 当前进度偏慢，建议调整计划。         │
│                                     │
│ 建议行动                            │
│ · 今天集中完成最核心的2个子任务      │
│ · 推迟其他低优先级事项              │
│ · 与导师确认截止日是否可协商        │
└─────────────────────────────────────┘
```

- `risk_level` 映射颜色：`high` → 橙色警告，`medium` → 黄色，`low` → 蓝色
- 卡片可关闭（叉号）
- 不会自动重复触发，需用户主动点击

### 3.5 API 消耗

- `temperature: 0.5`，`max_tokens: 250`（固定）
- 分类：L3，低频

---

## 4. 文件变更清单

| 文件 | 变更说明 |
|------|---------|
| `docs/superpowers/prompts/deadline-risk-warning.json` | 新建 prompt 存档 |
| `src-tauri/src/ai/prompt_templates.rs` | 新增 `task_duration_prompt()` + `milestone_risk_prompt()` |
| `src-tauri/src/commands/ai_commands.rs` | 新增 `ai_estimate_task_duration` + `ai_milestone_risk` 命令 |
| `src-tauri/src/lib.rs` | 注册 2 个新命令 |
| `src/stores/useAIStore.ts` | 新增 `estimateTaskDuration()` + `milestoneRisk()` 方法及对应接口类型 |
| `src/components/task/TaskEditModal.vue` | 预估字段旁新增 AI 估算按钮 + 结果提示行 |
| `src/components/goal/MilestoneInfoPanel.vue` | 截止日旁新增风险按钮 + 风险卡片 |

---

## 5. 不做的事（明确边界）

- **不自动触发 AI**：两个功能均为用户主动点击，不做轮询或自动推送
- **不存储 AI 结果**：结果仅在当前组件会话内保留，关闭后不持久化
- **不影响核心保存逻辑**：AI 建议仅作为辅助参考，用户可完全忽略
- **不做风险等级的本地计算**：风险等级由 AI 判断，前端仅判断是否满足"显示入口"的触发条件

---

## 6. 实现顺序

```
1. 新建 deadline-risk-warning.json prompt 存档
2. prompt_templates.rs — 新增 2 个函数
3. ai_commands.rs — 新增 2 个命令
4. lib.rs — 注册命令
5. cargo check
6. useAIStore.ts — 新增 2 个方法
7. TaskEditModal.vue — 集成 AI 估算
8. MilestoneInfoPanel.vue — 集成风险预警
9. vue-tsc --noEmit
10. git commit
```

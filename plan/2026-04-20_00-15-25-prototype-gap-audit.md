---
mode: audit
cwd: C:\Users\zhx\Desktop\FocusLab
task: prototype/ 19 屏 vs src/views+components 差距盘点
complexity: medium
scope: 只盘点不实施,用户后续决定补齐顺序
ref_prototype_root: prototype/
ref_view_root: src/views/ + src/components/
created_at: 2026-04-20_00-15-25
---

# Audit: 原型 vs 实现 差距清单

## 🎯 背景

HEAD `e9b0b72 feat(task): 三态轮转` 落地后,对照 `prototype/` 下 4 个 batch 共 19 屏做一次盘点,确认哪些原型定义的功能/视觉/交互在实现侧缺失、弱化、或已合并。本清单用于决定下一阶段开发顺序。

**盘点原则**:
- 只列"原型有但实现缺/弱化/变形"的点,已一致的不重复
- 设计合并(原型多页 → 实现一页)若合理则单列"合并决策"而非"缺失"
- 优先级以"用户可见度 + 是否阻塞主链路"判定

---

## 🔴 P0 · 破损 / 急缺

### 1. TodayView · 今日页 (`src/views/TodayView.vue`)

| 差距 | 原型出处 | 实现现状 |
|---|---|---|
| **DDL 到期条缺失** | `prototype/screens/main-today.html:599` v1.2.2 醒目横条 | `dueToday` computed 已算出,但模板未渲染 |
| **右栏未按 ADR-010 合并** | `main-today.html:715` Daily 卡合并 AI 建议 + 昨日小结 | 目前是 `.fl-ai-card` + 独立 `<YesterdayCard>` 两张 |

### 2. ParkedView · 搁置区 (`src/views/ParkedView.vue`)

| 差距 | 原型出处 | 实现现状 |
|---|---|---|
| **「拆分」按钮是假按钮** | `parked.html` 三档操作都可用 | `src/views/ParkedView.vue:99` 只有 `title` 无 `@click` |
| **降级规则只有 3 档** | `parked.html:493-524` D1/D2/D3/D30 四档 | 只有 D1/D7/D30,跳过 D2(连续遗留)/D3(弹卡片) |
| **缺语言原则面板** | `parked.html:530` 「💬 语言原则」禁用"失败""拖延"的说明 | 未实现 |

### 3. 弹窗体系 · 6 种 vs 5 种

| 弹窗 | 原型 | 实现 |
|---|---|---|
| 命令面板 | ✅ | ✅ `CommandPalette.vue` |
| AI 隐私 | ✅ | ✅ `AIPrivacyModal.vue` |
| 危险确认 | ✅ | ✅ `DangerConfirmModal.vue` |
| 数据导出 | ✅ | ✅ `ExportModal.vue` |
| 快速加任务 | ✅ | ✅ `QuickAddModal.vue` |
| **AI Payload 查看** | `modals.html:783` System Prompt + User Prompt 预览 | **❌ 缺失** |

---

## 🟡 P1 · 功能偏差

### 4. CalendarView · 日历 (`src/views/CalendarView.vue`)

原型 `screens/calendar.html` 三件套:月网格 + 侧栏 day detail + 每周固定日程模板。

| 差距 | 原型出处 |
|---|---|
| 缺「每周固定日程模板」 | `calendar.html:630` 周一~周日配固定事项 |
| 缺「每日可用工作时间预估」 | `calendar.html:692` "去掉固定日程和吃饭时间" |
| 缺周末红色高亮 | `calendar.html:89` `.weekend` `.weekend-num` warning 色 |

### 5. BubbleView · 悬浮球 (`src/views/BubbleView.vue`)

已有 project memory `project_bubble_issues.md` 追踪 7 项遗留。本次补充:

| 差距 | 原型出处 |
|---|---|
| 托盘右键菜单未接入 | `floating-ball.html:399-404` Tauri 托盘菜单原型 |

### 6. PersonaView · 科研人格 (`src/views/PersonaView.vue`)

| 差距 | 原型出处 | 实现现状 |
|---|---|---|
| **hatchDay 硬编码 = 3** | `persona-hatch.html` 按真实首启日期推算 | `src/views/PersonaView.vue:100` `const hatchDay = ref(3)` 模拟值 |
| **社交分享未实现** | `persona-card.html` 分享优先定位 | 分享卡 UI 已做,无导出图片/复制链接动作 |

### 7. BadgesView · 成就徽章 (`src/views/BadgesView.vue`)

| 差距 |
|---|
| **45 枚徽章的 `unlocked` 字段全部硬编码** (`src/views/BadgesView.vue:35+` 静态数组) |
| **未接入真实解锁引擎** — 需对齐 settlement/timer_session/task 的数据算 |

---

## 🟢 P2 · 细节 / 可延后

### 8. PomodoroView
- 5 态齐全(focus/sprint/break/done/free)
- `pomodoro.html:508` 的「🌀 自由模式 vs 🍅 番茄模式」对比说明面板未搬,目前只在底部一行文字提示

### 9. StatsView
- 概览/热力图/趋势/分类齐
- `useAIStore` 已 import,**AI 周度小结是否渲染需确认**(未找到明显调用点)

### 10. FTUE / MorningGuide / MicroReview / MoodCheck
- 组件存在,集成度良好,差距点小,不单列
- 后续做全链路回归测时再逐条对

---

## 🧭 设计合并决策(非缺失,记录在案)

| 原型独立页 | 实现合并点 | 评价 |
|---|---|---|
| `main-tasks.html` 四象限独立页 | `TodayView.viewMode='quadrant'` | 合理,减少一个 View |
| `main-today-lite.html` 紧凑变体 | 未做独立 View | 低优先,视用户反馈 |
| `persona-hatch.html` 独立页 | `PersonaView` 内 `fl-hatch` 模块 | 合理,人格孵化与图鉴本就同一场景 |

---

## 📌 推荐补齐顺序

1. **TodayView DDL 条 + 右栏合并** (P0 · 用户每日必见)
2. **ParkedView 拆分按钮 + D2/D3 降级 + 语言原则** (P0 · 拆分按钮当前是破损)
3. **AI Payload 查看 Modal** (P0 · AI 隐私透明化链路的最后一环)
4. **PersonaView hatchDay 接入真实天数** (P1 · 1-2 小时)
5. **CalendarView 固定日程模板 + 可用时间** (P1 · 较重,需建 schema)
6. **BadgesView 徽章解锁引擎** (P1 · 后端+前端都改)
7. **BubbleView 7 项遗留 + 托盘菜单** (P1 · 已有独立 memory 追踪)
8. **StatsView AI 周度小结** (P2 · 确认后补)
9. **PomodoroView 模式对比面板** (P2 · 纯说明)

---

## 📎 参考

- 提交基线: `e9b0b72 feat(task): 三态轮转 pending→in_progress→completed + 勾选双写`
- 原型根目录: `prototype/` (index.html 列 4 个 batch 共 19 屏)
- 相关 ADR: ADR-009 (社交实验隔离) · ADR-010 (右栏 Daily 卡合并) · ADR-011 (人格图鉴)
- 已追踪问题 memory: `project_bubble_issues.md`

---
mode: design
project: FocusLab
feature: 科研灵感工作台 / 灵感图谱 L1+L2
created_at: 2026-04-30
status: draft-approved-in-chat
---

# FocusLab 科研灵感工作台 / 灵感图谱设计

## 1. 设计目标

本次设计不是从零新建一套“科研图谱系统”，而是在 FocusLab 已有的“快速灵感笔记”能力上，增量长出更适合科研场景的结构化与关联能力。

目标是跑通这条闭环：

```text
低负担记录原始灵感
→ AI 生成 3 个更清晰的梳理候选
→ 用户保存原文或候选版本
→ AI 异步回看旧灵感并给出关联 / 纠偏提示
→ 用户确认连接、标记待复查、创建后续实验卡片
→ 某条灵感可进入局部图谱视图
```

本次范围覆盖：
- L1 锚点层：挂到目标、手动关联、灵感数据迁到 SQLite
- L2 AI 层：独立 embedding provider、异步相关推荐、AI 纠偏提示、局部图谱视图

不包含：
- 全局宇宙图谱
- 外部数据源接入(PDF/Zotero/网页)
- 本地 embedding 模型
- 6 类复杂关系类型系统

---

## 2. 为什么不是直接按 v0.3 文档做

现有《科研灵感图谱功能设计文档 v0.3》在产品方向上是对的，但实现假设过重，没有充分复用当前项目已有能力。

现状里已经存在：
- `useInspirationStore`：灵感记录与转任务链路
- `InspirationsView.vue`：完整灵感页面
- `QuickNoteModal`：AI 整理 3 候选入口
- `useGoalStore`：目标 / 里程碑系统
- `useAIStore`：多类 AI 能力入口
- Tauri + SQLite 迁移体系

因此本设计采用“复用 + 渐进扩展”策略，而不是直接引入新的 `research_plans / ai_recommendations / correction_warnings` 多表体系。

核心取舍：
- 长线计划直接复用现有 Goal
- 关系类型 UI 只保留两类：`related` / `contradicts`
- 推荐不持久化为复杂 recommendation 表，接受后直接落连接
- 局部图谱先做，一度关系优先；全局图谱后置

---

## 3. 用户体验原则

### 3.1 低负担优先
用户记录灵感时，第一目标不是“结构化输入”，而是“先别丢”。

因此输入区默认支持：
- 直接保存原文
- 点击 `AI 梳理` 后，生成 3 个候选版本：
  - 学术表述版
  - 行动拆解版
  - 问题假设版
- 每个候选都可：
  - 保存该版本
  - 转为任务

### 3.2 AI 只提示，不替用户下结论
AI 可做：
- 归属目标推荐
- 相关推荐
- 纠偏提醒
- 生成后续实验卡片草案

AI 不可做：
- 自动建立连接
- 自动判定旧观点错误
- 自动修改验证状态

最终动作必须由用户确认。

### 3.3 科研逻辑优先于关键词相似
推荐关系不以“相似文本”为核心，而以以下逻辑为核心：
- 一个提出问题，另一个提供方法
- 一个是现象，另一个是解释
- 一个支持另一个方向
- 一个修正了早期判断

### 3.4 局部有用优先于全局震撼
MVP 优先做好：
- 时间线卡片下的 AI 推荐
- 纠偏处理流
- 单条灵感的一度关系图谱

而不是一上来做全局图谱大画布。

---

## 4. 范围确认(已在对话中确认)

已确认的关键决策：

1. 范围：L1 + L2 一起做
2. 长线计划：直接复用现有 Goal
3. AI 推荐触发：保存后异步自动触发，静默回填
4. 默认值：AI 推荐默认开启
5. 关系类型：UI 只用两类 `related` / `contradicts`
6. Embedding provider：完全独立配置
7. 数据迁移：localStorage → SQLite 启动时一次性迁移
8. 入口策略：
   - 今日页轻量输入/摘要
   - 完整页承载重交互
   - Quick Note 保留原始 AI 三候选捕获逻辑
9. 架构方案：Rust 侧编排推荐链路，前端负责展示与操作

---

## 5. 信息架构

## 5.1 入口分层

### A. 今日页 `InspirationPanel`
职责：轻量速记 + 最近 3 条浏览 + 待处理推荐角标

不承担：
- AI 三候选 inline 展开
- 挂目标
- 手动关联
- 局部图谱

### B. `/inspirations` 完整页(核心工作台)
职责：
- 原始输入 / AI 三候选
- 搜索 / 筛选 / 分组浏览
- 挂目标
- 手动关联
- AI 推荐处理
- AI 纠偏处理
- 局部图谱入口

### C. `QuickNoteModal`
职责：
- 全局捕获原始灵感
- AI 三候选整理
- 直接保存 / 转任务

不承担：
- 历史推荐 review
- 图谱浏览

---

## 5.2 工作台版块

`inspiration-graph-v5.html` 对应的版块为：

1. 页头
   - 标题：科研灵感工作台
   - 副标题：强调“AI 只提示，用户确认”

2. 输入区
   - 原始灵感输入
   - `AI 梳理`
   - `直接保存原文`
   - inline 3 候选卡片

3. 主时间线
   - 按天分组
   - 每条卡片显示：
     - 内容
     - 时间
     - 所属目标
     - 标签
     - 连接数
     - 状态标签(待复查 / 已修正等)

4. AI 推荐区
   - 普通相关推荐
   - 纠偏提醒

5. 右侧辅助区
   - 灵感概览
   - 局部图谱视图
   - 待处理推荐
   - 目标分布
   - AI 索引状态

6. 弹层
   - 挂到目标
   - 手动建立关联

---

## 6. 数据模型

本设计采用 3 张核心表。

### 6.1 inspirations

```sql
CREATE TABLE inspirations (
  id                TEXT PRIMARY KEY,
  content           TEXT NOT NULL,
  goal_id           TEXT,
  summary           TEXT,
  keywords          TEXT,
  verification      TEXT NOT NULL DEFAULT 'none',
  embedding_status  TEXT NOT NULL DEFAULT 'pending',
  converted_task_id TEXT,
  converted_at      TEXT,
  created_at        TEXT NOT NULL,
  updated_at        TEXT NOT NULL,
  FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE SET NULL
);
```

说明：
- `goal_id` 直接挂载现有 goals
- `verification` 当前只保留两态：
  - `none`
  - `needs_check`
- `summary` / `keywords` 由 AI 补充生成，可为空

### 6.2 inspiration_embeddings

```sql
CREATE TABLE inspiration_embeddings (
  inspiration_id TEXT PRIMARY KEY,
  model          TEXT NOT NULL,
  dim            INTEGER NOT NULL,
  vector         BLOB NOT NULL,
  created_at     TEXT NOT NULL,
  FOREIGN KEY (inspiration_id) REFERENCES inspirations(id) ON DELETE CASCADE
);
```

说明：
- 记录 embedding model，便于未来切换模型时重算
- 不引入 pgvector / 向量库，当前量级可直接内存余弦扫描

### 6.3 inspiration_links

```sql
CREATE TABLE inspiration_links (
  id          TEXT PRIMARY KEY,
  source_id   TEXT NOT NULL,
  target_id   TEXT NOT NULL,
  relation    TEXT NOT NULL DEFAULT 'related',
  source_type TEXT NOT NULL DEFAULT 'manual',
  reason      TEXT,
  created_at  TEXT NOT NULL,
  FOREIGN KEY (source_id) REFERENCES inspirations(id) ON DELETE CASCADE,
  FOREIGN KEY (target_id) REFERENCES inspirations(id) ON DELETE CASCADE,
  UNIQUE(source_id, target_id)
);
```

说明：
- `relation` 当前只允许：
  - `related`
  - `contradicts`
- `source_type`：
  - `manual`
  - `ai_accepted`
- `reason` 仅在 AI 接受后的连接中保留理由

---

## 7. 迁移策略

### 7.1 启动迁移
旧灵感数据当前在 localStorage `fl-inspirations`。

迁移策略：
- 应用启动时检测旧数据
- 一次性写入 SQLite
- 成功后再清除 localStorage
- 失败时保留 localStorage 并显式提示

迁入时默认值：
- `goal_id = NULL`
- `summary = NULL`
- `keywords = NULL`
- `embedding_status = 'pending'`
- `verification = 'none'`

### 7.2 旧数据补索引
迁完后后台分批生成 embedding：
- 每批 10 条
- 更新进度条
- 失败置 `failed`
- 前端显式显示“AI 推荐暂不可用，可重试”

---

## 8. AI 架构设计

## 8.1 Provider 结构

当前 `AIProvider` 只有 `complete()`。
扩展为：

```rust
async fn embed(&self, texts: Vec<String>) -> AppResult<Vec<Vec<f32>>>;
```

策略：
- chat provider 继续服务文本推理
- embedding provider 独立配置
- Claude provider 默认不支持 embedding
- embedding 走 OpenAI-compatible 独立 provider

### 8.2 为什么 embedding 独立配置
已确认不与 chat provider 绑定。
原因：
- Claude 官方无 embedding
- 用户可能聊天用 Claude、embedding 用 OpenAI-compatible
- 避免把用户锁死在同一 provider

### 8.3 推荐工作流

```text
保存灵感
→ 立即写 inspirations
→ 前端先渲染卡片
→ 后台异步调用 embedding provider
→ 计算新向量
→ 扫描历史向量取 Top-K
→ 调 chat provider 精排
→ 返回 1-3 条候选
→ 前端回填到卡片下
```

### 8.4 AI 输出类型

AI 可输出两类关系：
- `related`
- `contradicts`

其中：
- `related`：普通相关、问题→方法、方法→验证等都收敛到这一类
- `contradicts`：用于纠偏、待复查、修正链路

---

## 9. 页面交互流

## 9.1 原始速记与 AI 梳理

用户路径：
1. 输入原始灵感
2. 选择：
   - 直接保存原文
   - AI 梳理
3. AI 梳理后出现 3 个候选：
   - 学术表述版
   - 行动拆解版
   - 问题假设版
4. 用户可：
   - 保存某个候选
   - 将某个候选直接转任务

设计理由：
- 对齐当前 `optimizeQuickNote` 产品心智
- 保持“先记录，后整理”的低负担模式
- 避免强迫用户一上来就写得很正式

## 9.2 挂到目标

交互：
- 卡片右上角点击 `挂到目标`
- 弹出目标 picker
- 展示：
  - 最近使用目标
  - 搜索目标
  - 取消挂载
  - AI 归属判断理由

结果：
- 卡片 badge 更新
- 目标统计同步更新

## 9.3 手动建立关联

交互：
- 点击 `手动关联`
- 搜索另一条灵感
- 默认关系类型为 `related`
- 建立后：
  - 连接数 +1
  - 局部图谱新增蓝色边

## 9.4 AI 推荐

普通推荐卡可执行：
- 接受为相关
- 忽略

接受后：
- 写入 `inspiration_links`
- 卡片连接数更新
- 局部图谱新增节点/边

## 9.5 AI 纠偏

纠偏场景 demo 已覆盖三步：

### a. 待复查
旧判断被多条新记录挑战时：
- 卡片显示 `待复查`
- 橙色关系边出现

### b. 修正动作
用户可：
- 标记待复查
- 建立修正连接(`contradicts`)
- 创建后续实验卡片

### c. 已处理结果态
当后续实验或新证据支持修正时：
- 新卡片出现 `纠偏已处理`
- 局部图谱出现绿色修正边
- 右侧待处理列表更新为已处理

---

## 10. 图谱体验设计

## 10.1 为什么先做局部图谱
全局图谱在数据少时价值不大，且容易噪声化。

局部图谱能更清晰地回答用户：
- 这条新灵感为什么和旧灵感有关？
- 哪些观点正在被挑战？
- 下一步实验应该接在哪条思路后面？

## 10.2 局部图谱规则
- 只展示一度关系
- 当前选中卡片居中
- 蓝边：`related`
- 橙边：`contradicts / 待复查`
- 绿边：`修正已完成 / 后续实验`

## 10.3 图谱入口
- 卡片工具区点 `circle-dashed`
- 或右侧 `局部图谱视图` 默认跟随当前选中卡片

## 10.4 图谱与时间线联动
- 时间线选卡 → 右侧图谱刷新
- 接受推荐 / 建立关联 → 图谱即时更新
- 纠偏处理完成 → 橙边可转为绿边

---

## 11. 缺省/异常状态

还需在实现中覆盖这些状态：

1. 冷启动空态
   - 没有任何灵感
2. 有灵感但没有 AI 推荐
   - 不显示空推荐容器
3. 索引进行中
   - 显示进度条
4. embedding 失败
   - 显式提示，不静默降级
5. 待复查列表为空
   - 轻提示，不制造焦虑

---

## 12. Rust / Tauri 命令建议

建议新增命令：
- `create_inspiration`
- `list_inspirations`
- `update_inspiration`
- `delete_inspiration`
- `link_inspirations`
- `unlink_inspirations`
- `get_inspiration_links`
- `migrate_inspirations_from_local`
- `suggest_related_inspirations`
- `accept_recommendation`
- `configure_embedding`
- `batch_embed_pending`

说明：
- 推荐链路建议全部放 Rust 侧完成
- 前端只负责：
  - 发起动作
  - 接收事件
  - 渲染状态

---

## 13. Demo 结论

当前 demo 已有 5 个版本，最终以 `inspiration-graph-v5.html` 作为最完整展示。

已覆盖场景：
- 原始速记 + AI 三候选
- 时间线主工作台
- AI 推荐
- AI 纠偏
- 局部图谱
- 挂目标弹层
- 手动关联弹层
- 已修正结果态

这已经足够支撑 implementation plan。

---

## 14. 本次设计的核心结论

这个功能真正的产品骨架不是“画一个图”，而是：

```text
让科研灵感从时间堆积
→ 变成可被 AI 重新激活的研究链条
→ 再由用户确认成自己的思考网络
```

因此 MVP 的关键不是全局图谱，而是 4 个最小可用能力：
1. 原始记录 + AI 三候选整理
2. 挂目标 / 手动关联
3. AI 相关推荐
4. AI 纠偏 + 局部图谱

以上四点跑通，这个功能就具备真实产品价值。

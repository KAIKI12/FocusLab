# FocusLab 科研灵感工作台 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在 FocusLab 现有快速灵感笔记能力上，实现科研灵感工作台 L1+L2：SQLite 持久化、独立 embedding provider、AI 三候选速记流、挂目标、手动关联、AI 相关推荐、AI 纠偏、局部图谱视图。

**Architecture:** 继续沿用现有 Vue + Pinia + Tauri + Rust + SQLite 架构。前端负责输入、列表、局部图谱和交互状态；Rust 侧负责 SQLite 持久化、AI provider 配置、embedding 计算、推荐召回与精排。推荐链路全部在 Rust 端编排，前端只接结果并呈现。

**Tech Stack:** Vue 3, Pinia, TypeScript, Tauri, Rust, rusqlite, existing AI service/provider abstraction

---

## Task 1: 定义前端类型与 store 目标状态

**Files:**
- Modify: `src/types.ts:198-261`
- Modify: `src/stores/useInspirationStore.ts:1-145`
- Test: `src/__tests__/setup.ts`

**Step 1: 为灵感工作台补充前端类型定义**

在 `src/types.ts` 新增最小类型：

```ts
export type InspirationVerification = "none" | "needs_check";
export type InspirationRelation = "related" | "contradicts";

export interface InspirationRecord {
  id: string;
  content: string;
  goalId: string | null;
  summary: string | null;
  keywords: string[];
  verification: InspirationVerification;
  embeddingStatus: "pending" | "done" | "failed";
  convertedTaskId: string | null;
  convertedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface InspirationLink {
  id: string;
  sourceId: string;
  targetId: string;
  relation: InspirationRelation;
  sourceType: "manual" | "ai_accepted";
  reason: string | null;
  createdAt: string;
}

export interface InspirationRecommendation {
  candidateId: string;
  candidateContent: string;
  relation: InspirationRelation;
  reason: string;
  confidence: number;
}
```

**Step 2: 先在 `useInspirationStore` 里声明目标 API 形状（先允许 TODO stub）**

把 store 的对外接口从 localStorage-only 形态，调整为后续可切 Tauri 的形态：

```ts
const items = ref<InspirationRecord[]>([]);
const linksById = ref<Record<string, InspirationLink[]>>({});
const pendingRecommendations = ref<Record<string, InspirationRecommendation[]>>({});
```

保留当前 `create / remove / convertToTask` 名称，但注释和参数签名改成以 SQLite / invoke 为目标。

**Step 3: 运行现有前端测试，确认纯类型/签名修改未破坏基础环境**

Run: `pnpm vitest run src/stores/useGoalStore.spec.ts`
Expected: PASS

**Step 4: Commit**

```bash
git add src/types.ts src/stores/useInspirationStore.ts
git commit -m "refactor: prepare inspiration types for graph workspace"
```

---

## Task 2: 增加 SQLite 迁移文件并注册 migration

**Files:**
- Create: `src-tauri/src/db/migrations/004_inspiration_graph.sql`
- Modify: `src-tauri/src/db/migrator.rs:21-34`
- Test: `src-tauri/src/db/migrator.rs:36-83`

**Step 1: 写迁移 SQL**

创建 `004_inspiration_graph.sql`，内容包含 3 张表：

```sql
CREATE TABLE inspirations (
  id                TEXT PRIMARY KEY,
  content           TEXT NOT NULL,
  goal_id           TEXT,
  summary           TEXT,
  keywords          TEXT NOT NULL DEFAULT '[]',
  verification      TEXT NOT NULL DEFAULT 'none',
  embedding_status  TEXT NOT NULL DEFAULT 'pending',
  converted_task_id TEXT,
  converted_at      TEXT,
  created_at        TEXT NOT NULL,
  updated_at        TEXT NOT NULL,
  FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE SET NULL
);

CREATE TABLE inspiration_embeddings (
  inspiration_id TEXT PRIMARY KEY,
  model          TEXT NOT NULL,
  dim            INTEGER NOT NULL,
  vector         BLOB NOT NULL,
  created_at     TEXT NOT NULL,
  FOREIGN KEY (inspiration_id) REFERENCES inspirations(id) ON DELETE CASCADE
);

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

CREATE INDEX idx_inspirations_goal ON inspirations(goal_id);
CREATE INDEX idx_inspirations_created ON inspirations(created_at DESC);
CREATE INDEX idx_inspiration_links_source ON inspiration_links(source_id);
CREATE INDEX idx_inspiration_links_target ON inspiration_links(target_id);
```

**Step 2: 在 migrator 注册 migration**

在 `src-tauri/src/db/migrator.rs` 的 `MIGRATIONS` 里追加：

```rust
Migration {
    version: "004_inspiration_graph",
    sql: include_str!("migrations/004_inspiration_graph.sql"),
},
```

**Step 3: 写一个最小 Rust 单测，验证迁移后表存在**

在 `migrator.rs` 或独立测试模块中，使用内存 SQLite：

```rust
#[test]
fn runs_inspiration_graph_migration() {
    let mut conn = Connection::open_in_memory().unwrap();
    run(&mut conn).unwrap();

    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='inspirations'")
        .unwrap();
    let exists: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(exists, vec!["inspirations".to_string()]);
}
```

**Step 4: 运行 Rust 测试**

Run: `cargo test migrator --manifest-path src-tauri/Cargo.toml`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/db/migrations/004_inspiration_graph.sql src-tauri/src/db/migrator.rs
git commit -m "feat: add inspiration graph sqlite schema"
```

---

## Task 3: 建立 Rust model 层的 inspirations 基础 CRUD

**Files:**
- Create: `src-tauri/src/models/inspiration.rs`
- Modify: `src-tauri/src/models/mod.rs`
- Test: `src-tauri/src/models/inspiration.rs`

**Step 1: 参考 `goal.rs` / `milestone.rs` 写最小 inspiration model**

最少实现：

```rust
pub struct InspirationRecord { ... }
pub fn list_inspirations(conn: &Connection) -> AppResult<Vec<InspirationRecord>>
pub fn create_inspiration(conn: &Connection, content: &str, goal_id: Option<&str>) -> AppResult<InspirationRecord>
pub fn update_inspiration_goal(conn: &Connection, id: &str, goal_id: Option<&str>) -> AppResult<()>
pub fn update_inspiration_verification(conn: &Connection, id: &str, verification: &str) -> AppResult<()>
pub fn delete_inspiration(conn: &Connection, id: &str) -> AppResult<()>
```

**Step 2: 为 list/create/update/delete 写内存数据库单测**

至少覆盖：
- create 后能 list 到
- update goal 后字段变化
- update verification 后字段变化
- delete 后列表为空

**Step 3: 运行模型测试**

Run: `cargo test inspiration --manifest-path src-tauri/Cargo.toml`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/src/models/inspiration.rs src-tauri/src/models/mod.rs
git commit -m "feat: add inspiration model crud"
```

---

## Task 4: 建立 Rust model 层的 links CRUD

**Files:**
- Modify: `src-tauri/src/models/inspiration.rs`
- Test: `src-tauri/src/models/inspiration.rs`

**Step 1: 在同一 model 文件增加 link 结构与操作**

```rust
pub struct InspirationLink { ... }
pub fn list_links_for_inspiration(conn: &Connection, inspiration_id: &str) -> AppResult<Vec<InspirationLink>>
pub fn create_link(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
    relation: &str,
    source_type: &str,
    reason: Option<&str>,
) -> AppResult<InspirationLink>
pub fn delete_link(conn: &Connection, source_id: &str, target_id: &str) -> AppResult<()>
```

处理去重时，统一 source/target 排序，避免 A-B / B-A 重复。

**Step 2: 写失败测试覆盖重复连接**

```rust
#[test]
fn prevents_duplicate_links_regardless_of_direction() {
    ...
}
```

**Step 3: 运行测试**

Run: `cargo test inspiration --manifest-path src-tauri/Cargo.toml`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/src/models/inspiration.rs
git commit -m "feat: add inspiration link persistence"
```

---

## Task 5: 增加 embedding provider 配置持久化

**Files:**
- Modify: `src-tauri/src/commands/ai_commands.rs:129-205`
- Modify: `src-tauri/src/models/settings.rs`
- Modify: `src/stores/useAIStore.ts:51-83`
- Test: `src-tauri/src/commands/ai_commands.rs`

**Step 1: 在 settings 层定义 embedding 相关 key**

最少 key：
- `ai_embedding_base_url`
- `ai_embedding_api_key`
- `ai_embedding_model`
- `ai_embedding_enabled`

**Step 2: 在 `ai_commands.rs` 增加配置命令**

```rust
#[tauri::command]
pub async fn configure_embedding(...)
```

写入 settings 表。

**Step 3: 在 `useAIStore.ts` 新增前端方法**

```ts
async function configureEmbedding(baseUrl: string, apiKey: string, model: string, enabled?: string)
```

**Step 4: 为配置命令写单测**

验证写入后 settings 表能读到对应值。

**Step 5: 运行测试**

Run: `cargo test ai_commands --manifest-path src-tauri/Cargo.toml`
Expected: PASS

**Step 6: Commit**

```bash
git add src-tauri/src/commands/ai_commands.rs src-tauri/src/models/settings.rs src/stores/useAIStore.ts
git commit -m "feat: add embedding provider configuration"
```

---

## Task 6: 扩展 AIProvider trait，支持 embedding

**Files:**
- Modify: `src-tauri/src/ai/mod.rs`
- Test: `src-tauri/src/ai/mod.rs`

**Step 1: 给 trait 增加默认 `embed` 方法**

```rust
async fn embed(&self, texts: Vec<String>) -> AppResult<Vec<Vec<f32>>> {
    Err(AppError::NotSupported("embedding not supported".into()))
}
```

**Step 2: 为 OpenAI-compatible provider 增加 embeddings 请求结构**

新增：

```rust
#[derive(Serialize)]
struct EmbeddingRequest { ... }
#[derive(Deserialize)]
struct EmbeddingResponse { ... }
```

并实现 `embed()`。

**Step 3: 为 Claude provider 保持默认不支持**

不新增 hack，不 silent fallback。

**Step 4: 为 `embed()` 写解析测试**

可以用纯 JSON decode 单测，不强求真实 HTTP。

**Step 5: 运行测试**

Run: `cargo test ai --manifest-path src-tauri/Cargo.toml`
Expected: PASS

**Step 6: Commit**

```bash
git add src-tauri/src/ai/mod.rs
git commit -m "feat: extend ai provider with embedding support"
```

---

## Task 7: 在 Rust 侧封装 inspiration recommendation service

**Files:**
- Create: `src-tauri/src/services/inspiration_service.rs`
- Modify: `src-tauri/src/services/mod.rs`
- Test: `src-tauri/src/services/inspiration_service.rs`

**Step 1: 建立最小 service 结构**

职责：
- 为单条灵感生成 embedding
- 拉取历史 embedding
- 做余弦相似度 Top-K
- 将 Top-K 候选送入 LLM 精排

核心 API：

```rust
pub async fn suggest_related_inspirations(...) -> AppResult<Vec<InspirationRecommendation>>
```

**Step 2: 先写纯函数测试：余弦 Top-K**

```rust
#[test]
fn returns_top_k_by_cosine_similarity() { ... }
```

**Step 3: 写推荐过滤测试**

验证：
- 低于阈值 0.7 的候选被过滤
- 最多返回 3 条
- 同 goal 可做轻微加权

**Step 4: 运行 service 测试**

Run: `cargo test inspiration_service --manifest-path src-tauri/Cargo.toml`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/services/inspiration_service.rs src-tauri/src/services/mod.rs
git commit -m "feat: add inspiration recommendation service"
```

---

## Task 8: 暴露 Tauri commands 给前端

**Files:**
- Create: `src-tauri/src/commands/inspiration_commands.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs:84-160`
- Test: `src-tauri/src/commands/inspiration_commands.rs`

**Step 1: 增加命令文件并暴露基础命令**

最少命令：
- `create_inspiration`
- `list_inspirations`
- `update_inspiration_goal`
- `update_inspiration_verification`
- `delete_inspiration`
- `link_inspirations`
- `unlink_inspirations`
- `list_inspiration_links`
- `suggest_related_inspirations`

**Step 2: 把命令注册进 `lib.rs` invoke_handler**

按现有 `goal_commands` / `ai_commands` 风格追加。

**Step 3: 写命令级测试**

验证 command 层至少能调用 model/service 成功返回正确结果。

**Step 4: 运行测试**

Run: `cargo test inspiration_commands --manifest-path src-tauri/Cargo.toml`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/commands/inspiration_commands.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: expose inspiration graph tauri commands"
```

---

## Task 9: 用 TDD 改造 `useInspirationStore`，从 localStorage 切换到 Tauri

**Files:**
- Modify: `src/stores/useInspirationStore.ts`
- Create: `src/stores/useInspirationStore.spec.ts`
- Test: `src/stores/useInspirationStore.spec.ts`

**Step 1: 写 failing test，约束 store 行为**

至少覆盖：
- `ensureLoaded` 调 list command
- `create` 调 create command 并插入顶部
- `convertToTask` 成功后更新 converted 字段
- `loadRecommendations` / `acceptRecommendation` 更新本地状态

**Step 2: 运行测试确认失败**

Run: `pnpm vitest run src/stores/useInspirationStore.spec.ts`
Expected: FAIL because store methods not implemented / command mocks missing

**Step 3: 最小实现 store 切换**

引入 `invokeCmd`，实现：

```ts
ensureLoaded()
create(content: string)
assignGoal(id: string, goalId: string | null)
markNeedsCheck(id: string)
linkManually(sourceId: string, targetId: string)
loadRecommendations(id: string)
acceptRecommendation(...)
```

**Step 4: 运行测试确认通过**

Run: `pnpm vitest run src/stores/useInspirationStore.spec.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add src/stores/useInspirationStore.ts src/stores/useInspirationStore.spec.ts
git commit -m "feat: migrate inspiration store to tauri commands"
```

---

## Task 10: 保留并复用现有 QuickNoteModal 三候选能力

**Files:**
- Modify: `src/components/common/QuickNoteModal.vue`
- Modify: `src/views/InspirationsView.vue:59-72,312-314`
- Modify: `src/stores/useUIStore.ts`
- Test: `src/__tests__/setup.ts`

**Step 1: 让 `/inspirations` 输入区继续走现有 AI 三候选逻辑**

保留：
- `ui.quickNotePrefilledText`
- `ui.showQuickNote = true`

新增目标：
- 在 `QuickNoteModal` 成功保存灵感后，能回调到 inspirations 页面并刷新列表

**Step 2: 增加一个 “保存为灵感” 结果回调**

当前 modal 更偏转任务，计划改为支持：
- 保存为灵感
- 保存并转任务

**Step 3: 写最小页面联动测试（可用 mock UI store）**

验证 `onAiAssist()` 仍能把 draft 带进 modal。

**Step 4: 运行测试**

Run: `pnpm vitest run src/stores/useInspirationStore.spec.ts src/stores/useGoalStore.spec.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add src/components/common/QuickNoteModal.vue src/views/InspirationsView.vue src/stores/useUIStore.ts
git commit -m "feat: reuse quick note ai candidates in inspirations workspace"
```

---

## Task 11: 在 `/inspirations` 页面加入目标、推荐、纠偏状态

**Files:**
- Modify: `src/views/InspirationsView.vue`
- Test: `src/stores/useInspirationStore.spec.ts`

**Step 1: 先写 failing test / fixture，定义页面应显示的新状态**

最少检查：
- goal badge
- link count
- verification badge
- recommendation summary area

**Step 2: 实现最小 UI 改造**

在卡片上增加：
- `goalId` → 显示目标名 badge
- `links count`
- `needs_check` → 橙色待复查 badge
- 推荐折叠头

**Step 3: 运行页面相关测试**

Run: `pnpm vitest run src/stores/useInspirationStore.spec.ts`
Expected: PASS

**Step 4: Commit**

```bash
git add src/views/InspirationsView.vue
git commit -m "feat: add goal and recommendation state to inspirations page"
```

---

## Task 12: 实现挂到目标 picker

**Files:**
- Create: `src/components/inspiration/InspirationGoalPicker.vue`
- Modify: `src/views/InspirationsView.vue`
- Modify: `src/stores/useGoalStore.ts`
- Test: `src/stores/useGoalStore.spec.ts`

**Step 1: 写 failing test：goal store 已加载时，picker 能显示目标列表**

**Step 2: 实现最小 picker**

内容：
- 最近使用目标列表
- 搜索输入框
- 取消挂载

点击目标时调用：

```ts
await inspiration.assignGoal(inspirationId, goalId)
```

**Step 3: 运行测试**

Run: `pnpm vitest run src/stores/useGoalStore.spec.ts src/stores/useInspirationStore.spec.ts`
Expected: PASS

**Step 4: Commit**

```bash
git add src/components/inspiration/InspirationGoalPicker.vue src/views/InspirationsView.vue src/stores/useGoalStore.ts
git commit -m "feat: add inspiration goal picker"
```

---

## Task 13: 实现手动建立关联弹层

**Files:**
- Create: `src/components/inspiration/InspirationLinkModal.vue`
- Modify: `src/views/InspirationsView.vue`
- Test: `src/stores/useInspirationStore.spec.ts`

**Step 1: 写 failing test：link modal 选中候选后调用 `linkManually`**

**Step 2: 实现最小弹层**

内容：
- 搜索另一条灵感
- 展示时间 + 目标
- 默认 relation = `related`
- 建立后关闭 modal

**Step 3: 运行测试**

Run: `pnpm vitest run src/stores/useInspirationStore.spec.ts`
Expected: PASS

**Step 4: Commit**

```bash
git add src/components/inspiration/InspirationLinkModal.vue src/views/InspirationsView.vue
git commit -m "feat: add manual inspiration linking flow"
```

---

## Task 14: 实现 AI 推荐处理结果态

**Files:**
- Modify: `src/views/InspirationsView.vue`
- Modify: `src/stores/useInspirationStore.ts`
- Test: `src/stores/useInspirationStore.spec.ts`

**Step 1: 写 failing test：accept/reject recommendation 会更新本地 pendingRecommendations**

**Step 2: 实现推荐处理动作**

- `acceptRecommendation`:
  - 创建 link
  - 从 pending recommendations 移除
- `rejectRecommendation`:
  - 仅移除推荐
- `markNeedsCheck`:
  - 更新卡片 verification

**Step 3: 在页面显示处理结果**

接受后：
- 连接数变化
- 推荐卡消失
- 需要时显示 “已建立连接” 或 “待复查” 标签

**Step 4: 运行测试**

Run: `pnpm vitest run src/stores/useInspirationStore.spec.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add src/views/InspirationsView.vue src/stores/useInspirationStore.ts
git commit -m "feat: add recommendation result states"
```

---

## Task 15: 实现 AI 纠偏完整流

**Files:**
- Modify: `src/views/InspirationsView.vue`
- Create: `src/components/inspiration/InspirationCorrectionPanel.vue`
- Modify: `src/stores/useInspirationStore.ts`
- Test: `src/stores/useInspirationStore.spec.ts`

**Step 1: 写 failing test：contradicts 推荐可触发 `markNeedsCheck` 和“后续实验卡片”草案**

**Step 2: 实现纠偏面板**

展示：
- 旧判断内容
- “近期记录指向其他原因”的 AI 理由
- 三个动作：
  - 标记待复查
  - 建立修正连接
  - 创建后续实验卡片

**Step 3: 最小实现“创建后续实验卡片”**

MVP 不直接建实验对象，先：
- 创建一条新的 inspiration，内容为 AI 生成的实验草案文本
- 自动挂同一 goal
- 默认 tag / summary 可空

**Step 4: 运行测试**

Run: `pnpm vitest run src/stores/useInspirationStore.spec.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add src/components/inspiration/InspirationCorrectionPanel.vue src/views/InspirationsView.vue src/stores/useInspirationStore.ts
git commit -m "feat: add ai correction workflow"
```

---

## Task 16: 实现局部图谱视图组件

**Files:**
- Create: `src/components/inspiration/InspirationGraphView.vue`
- Modify: `src/views/InspirationsView.vue`
- Test: `src/stores/useInspirationStore.spec.ts`

**Step 1: 写 failing test：给定一条灵感和 links，可产出一度关系视图数据**

**Step 2: 先实现“非画布版”最小图谱数据转换**

写一个前端 helper：

```ts
function buildLocalGraph(centerId, items, links) {
  return { nodes, edges }
}
```

只做一度关系，不做全局图谱。

**Step 3: 用轻量 DOM/SVG 先渲染，不引第三方图库**

要求：
- 中心节点居中
- related 蓝边
- contradicts 橙边
- 后续实验 / 已修正状态可用绿标签表现

**Step 4: 运行测试**

Run: `pnpm vitest run src/stores/useInspirationStore.spec.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add src/components/inspiration/InspirationGraphView.vue src/views/InspirationsView.vue
git commit -m "feat: add local inspiration graph view"
```

---

## Task 17: 添加冷启动 / 无推荐 / 索引失败状态

**Files:**
- Modify: `src/views/InspirationsView.vue`
- Modify: `src/stores/useInspirationStore.ts`
- Test: `src/stores/useInspirationStore.spec.ts`

**Step 1: 写 failing test：空态和 failed 状态有显式 UI 文案**

**Step 2: 加入 4 种状态文案**

- 无灵感空态
- 有灵感但无推荐
- embedding 进行中
- embedding 失败，可重试

不要 silent degradation。

**Step 3: 运行测试**

Run: `pnpm vitest run src/stores/useInspirationStore.spec.ts`
Expected: PASS

**Step 4: Commit**

```bash
git add src/views/InspirationsView.vue src/stores/useInspirationStore.ts
git commit -m "feat: add inspiration workspace empty and error states"
```

---

## Task 18: 实现 localStorage → SQLite 一次性迁移

**Files:**
- Modify: `src/stores/useInspirationStore.ts`
- Modify: `src-tauri/src/commands/inspiration_commands.rs`
- Test: `src/stores/useInspirationStore.spec.ts`
- Test: `src-tauri/src/commands/inspiration_commands.rs`

**Step 1: 写 failing test：检测旧 `fl-inspirations` 时触发迁移命令**

**Step 2: 实现命令 `migrate_inspirations_from_local`**

命令接收一组旧 records，批量插入 SQLite。

**Step 3: 前端迁移逻辑**

`ensureLoaded()` 中：
- 若本地存在旧 localStorage 且后端未迁移
- 调迁移 command
- 成功后清理 localStorage
- 失败则保留并显式提示

**Step 4: 运行测试**

Run: `pnpm vitest run src/stores/useInspirationStore.spec.ts && cargo test inspiration_commands --manifest-path src-tauri/Cargo.toml`
Expected: PASS

**Step 5: Commit**

```bash
git add src/stores/useInspirationStore.ts src-tauri/src/commands/inspiration_commands.rs
git commit -m "feat: migrate inspirations from local storage to sqlite"
```

---

## Task 19: 把新命令接进 AI / 引导 / 详情层

**Files:**
- Modify: `src/App.vue`
- Modify: `src/views/TodayView.vue`
- Modify: `src/components/common/Sidebar.vue`
- Test: `src/__tests__/setup.ts`

**Step 1: 检查并补全入口联动**

- 今日页继续能打开 quick note
- Sidebar 跳转 `/inspirations`
- App 级 modal / page 不冲突

**Step 2: 跑最小 smoke**

Run: `pnpm vitest run`
Expected: PASS or only unrelated known failures

**Step 3: Commit**

```bash
git add src/App.vue src/views/TodayView.vue src/components/common/Sidebar.vue
git commit -m "feat: wire inspiration workspace into app navigation"
```

---

## Task 20: 端到端手工验证与收尾

**Files:**
- Review only: `src/views/InspirationsView.vue`
- Review only: `src/components/common/QuickNoteModal.vue`
- Review only: `src/components/inspiration/*.vue`

**Step 1: 本地运行前端与 Tauri 所需检查命令**

Run: `pnpm vitest run`
Expected: PASS

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS

**Step 2: 手工走 7 条验证路径**

1. 输入原始灵感，直接保存原文
2. 输入原始灵感，点 AI 梳理，保存其中一个候选
3. 给灵感挂到某个 Goal
4. 手动建立一条 related 连接
5. 触发一条 AI related 推荐并接受
6. 触发一条 AI contradicts 推荐并标记待复查
7. 在局部图谱里看到蓝边/橙边更新

**Step 3: 最终收尾 commit**

```bash
git add src src-tauri docs
git commit -m "feat: add research inspiration workspace and local graph flow"
```

---

## Notes for execution

- 不要引入全局图谱
- 不要引入向量数据库
- 不要接外部资料源
- 不要做本地 embedding 模型
- 不要在关系类型上扩展到 6 类
- MVP 保持 `related / contradicts` 两类即可
- 所有失败状态必须显式展示，不做 silent fallback

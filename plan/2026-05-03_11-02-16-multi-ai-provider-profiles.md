---
mode: plan
cwd: C:/Users/zhx/Desktop/FocusLab
task: 多 AI Provider Profile 管理（chat / embedding 双独立池子 + 自定义命名 + 池子式快速切换）
complexity: complex
tool: 手工分阶段
total_thoughts: 0
created_at: 2026-05-03_11-02-16
---

# Plan: 多 AI Provider Profile 管理

## 🎯 Task Overview

把当前"单条 AI 配置"改造成 **profile 池子**：可以保存任意多份连接配置，每份带自定义名字。
chat 和 embedding 两条**互相独立**的 profile 列表，每条列表"当前激活"一个 profile，
顶部一键切换。保留旧 settings keys 作为镜像，旧用户首次启动自动迁移成"默认 profile"，
零配置丢失风险。

## 📋 Execution Plan

### Phase 1 · DB schema 与一次性迁移
1. 新建 `src-tauri/src/db/migrations/005_ai_profiles.sql`，建两张表：
   - `ai_chat_profiles(id, name, provider, api_format, base_url, api_key, model_fast, model_strong, created_at, updated_at)`
   - `ai_embedding_profiles(id, name, base_url, api_key, model, created_at, updated_at)`
2. 在 `db/migrator.rs` 注册 005。
3. 启动迁移钩子 `db::ensure_default_ai_profile`：
   - 用 `settings.ai_default_profile_migrated` 作为一次性 flag。
   - 若 flag 缺失且旧 keys（`ai_base_url`/`ai_api_key`/`ai_model`）非空 → 插入一条名为"默认"的 chat profile。
   - 若 `ai_embedding_base_url` 非空 → 插入一条名为"默认 Embedding"的 embedding profile。
   - 写 `ai_default_profile_migrated=1`，避免用户删除"默认"后又被复活。
4. 新增两个 settings key：`ai_active_chat_profile_id`、`ai_active_embedding_profile_id`，
   迁移时填入新建 profile 的 id。

### Phase 2 · Rust models + commands
1. `src-tauri/src/models/ai_profile.rs`：定义 `ChatProfile` / `EmbeddingProfile` struct + CRUD helper。
2. `src-tauri/src/commands/ai_profile_commands.rs`：每类 profile 5 个命令，共 10 个：
   - `list_chat_profiles` / `list_embedding_profiles`
   - `create_chat_profile` / `create_embedding_profile`
   - `update_chat_profile` / `update_embedding_profile`
   - `delete_chat_profile` / `delete_embedding_profile`
   - `activate_chat_profile` / `activate_embedding_profile`（写 active id + 同步旧 keys + 调 `AIService::configure*`）
3. 改 `lib.rs:38-67` 启动注入：读 active profile id → 查表 → 配置 `AIService`，
   找不到时回退到旧 keys（兼容路径）。
4. **保留** `configure_ai` / `configure_embedding` 旧 command 作"快捷直配"（前端短期内仍可用），
   内部行为改为：upsert 到默认 profile + 激活。
5. 在 `lib.rs::run` 的 `invoke_handler!` 注册新 10 个命令。

### Phase 3 · Vue store + UI
1. `src/stores/useAIProfileStore.ts`：封装 list/create/update/delete/activate，类型与 Rust 对齐。
2. 改 `src/views/SettingsView.vue` AI 区块（B / C 两段）：
   - chat 区：`profile 卡片列表`（name + provider 徽章 + base_url 简写 + model 摘要 + 三个按钮：激活 / 编辑 / 删除）+ 顶部"新建 chat profile"按钮 → 弹出现有连接配置表单（复用现有字段）。
   - 激活的 profile 卡片高亮（左侧色条 + "当前激活"标签）。
   - "测试连接"按钮挂在每张卡片上，调用现有 `test_ai_connection`（它读 active profile，所以测哪张要先激活）。
   - embedding 区：同结构，独立卡片列表。
3. `onMounted` 改为先 `loadProfiles()`，不再直接读散 keys。

### Phase 4 · 测试 & 验证
1. Rust 单测：
   - `ensure_default_ai_profile` 在空 / 有旧 keys / flag 已置位 三种状态下分别幂等。
   - `activate_chat_profile` 写入 settings 镜像后，`lib.rs` 启动逻辑能复原。
2. Vue 单测：`useAIProfileStore` CRUD 正常调用 invoke。
3. 手工冒烟：
   - 旧用户首启 → 看到"默认"profile，原配置完整。
   - 新建第二个 profile → 切换 → 测试连接 → 灵感梳理走新 profile。
   - 删除非激活 profile → 列表更新；删除激活 profile → 自动激活第一个剩余 profile（或清空 active id）。

## ⚠️ Risks and Precautions

- **迁移幂等性**：`ai_default_profile_migrated` flag 是关键，必须在写入"默认"profile **同一事务**里置位，
  否则启动崩溃可能导致重复迁移。
- **active profile 缺失**：删除 active profile 后必须自动选下一个 / 清空 + 弹空状态，不允许 active id 指向已删行。
- **旧 keys 镜像同步**：每次激活时，把 active profile 的字段写回 `ai_base_url` 等旧 key，保证 `lib.rs` 启动 hook 即使读旧 key 也能拿到最新值（双保险，迁移完成后旧 key 可逐步淘汰）。
- **API key 仍明文存储**：与现状一致，不在本次改动范围；profile 表加密留作未来 task。
- **UI 复杂度**：列表卡片 + 编辑表单 + 弹层切换 ~150 行 Vue + 30 行 CSS，注意 KISS，不要做拖拽排序等花活。
- **现有命令兼容**：`configure_ai` / `configure_embedding` 不删，前端旧调用点照常工作 1~2 个版本，再做清理 commit。

## 📎 References

- `src-tauri/src/commands/ai_commands.rs:114-218` configure_ai / test_ai_connection
- `src-tauri/src/commands/ai_commands.rs:163-193` configure_embedding
- `src-tauri/src/lib.rs:38-67` 启动注入逻辑
- `src-tauri/src/db/migrator.rs` 迁移注册位置
- `src/views/SettingsView.vue:572-740` AI / Embedding 设置 UI
- `src/stores/useAIStore.ts:53-91` 现有 configure / configureEmbedding 调用形态

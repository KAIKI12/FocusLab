---
mode: plan
cwd: C:\Users\zhx\Desktop\FocusLab
task: AI 聊天侧边板块（流式 / 隔离会话 / 工具协议位 / 与灵感"研究问题版"贯通）
complexity: complex
tool: 直接实施（已对齐设计，分 4 批可独立验收）
total_thoughts: 2 轮交互（设计 → 决策）
created_at: 2026-05-03T13:58:39+08:00
---

# Plan: AI 聊天侧边板块

## 🎯 任务概览

在现有 AI 基础设施（同步 `complete` / `complete_required`）之上，加一个真正的对话式 AI 模块。
入口形态为**右侧抽屉**（`pin` 模式默认推主区，不占用主路由），支持流式输出、独立会话隔离、模型切换、历史记录持久化。
与灵感"研究问题版"贯通：B 候选卡片新增"深入聊聊"按钮，预填首条用户消息（不自动发送），
system prompt 注入"原始速记 + AI 优化后的研究问题 + 研究伙伴角色"。

---

## 📐 用户决策（已锁定）

| 项 | 选择 |
|----|------|
| 流式 vs 同步起步 | **流式**（升级整个 AI 子系统） |
| 会话隔离粒度 | **每条灵感"研究问题版"建独立会话** |
| 原始速记入上下文 | **要**（system prompt 注入） |
| 工具调用 | **MVP 不实现，但预留协议位**（DB 字段 + provider trait flag） |
| 全局快捷键 | **不加** |
| 入口形态 | **右侧 ChatPanel 抽屉**（不占用 `/chat` 路由） |
| Pin 默认值 | **`pinned=true`**（推主区） |
| 会话列表位置 | **顶部下拉** + 当前会话标题 |
| 首条消息从灵感跳进 | **预填**到输入框（不自动发送） |
| 用户输入字数上限 | **无上限** |

---

## 🗂 文件落点

### 后端
- `src-tauri/src/db/migrations/006_ai_chat.sql` 新增 — 两张表 + 索引
- `src-tauri/src/db/migrator.rs` 改 — `MIGRATIONS` 加 `006_ai_chat`
- `src-tauri/src/models/ai_chat.rs` 新增 — struct + CRUD（对齐 `ai_profile.rs` 风格）
- `src-tauri/src/models/mod.rs` 改 — `pub mod ai_chat;`
- `src-tauri/src/services/ai_chat_service.rs` 新增 — 业务逻辑（生成标题、上下文装配、调流式 provider）
- `src-tauri/src/services/mod.rs` 改 — `pub mod ai_chat_service;`
- `src-tauri/src/ai/streaming.rs` 新增 — OpenAI SSE / Claude stream 解析
- `src-tauri/src/ai/chat_prompts.rs` 新增 — `build_inspiration_research_system(raw, optimized)` 等
- `src-tauri/src/ai/mod.rs` 改 — `AIProvider` trait 加 `stream_complete`，两个 provider 实现，`AIService` 加 `stream_required`
- `src-tauri/src/commands/chat_commands.rs` 新增 — list/create/send/abort/rename/delete/pin/archive/setModel
- `src-tauri/src/commands/mod.rs` 改 — `pub mod chat_commands;`
- `src-tauri/src/lib.rs` 改 — `invoke_handler!` 注册新 commands

### 前端
- `src/types/chat.ts` 新增 — Conversation / Message / StreamChunk 类型
- `src/stores/useChatStore.ts` 新增 — Pinia store，管理会话列表、流式状态、`abort()`
- `src/stores/useUIStore.ts` 改 — 加 `showChat / chatPanelWidth / chatPanelPinned`（持久化到 localStorage）
- `src/components/chat/ChatPanel.vue` 新增 — 主容器（顶部 ConversationDropdown + 中间 MessageList + 底部 MessageInput）
- `src/components/chat/ConversationDropdown.vue` 新增 — 会话切换下拉 + 新建/重命名/删除/置顶/归档
- `src/components/chat/MessageList.vue` 新增 — 消息流（含流式打字效果）
- `src/components/chat/MessageBubble.vue` 新增 — 单条消息（user/assistant，复制按钮，失败重试）
- `src/components/chat/MessageInput.vue` 新增 — 多行输入 + 发送/中止按钮 + ModelPicker
- `src/components/chat/ModelPicker.vue` 新增 — 切换会话级模型
- `src/App.vue` 改 — 主区右侧挂 `<ChatPanel v-if="ui.showChat" />`
- `src/components/common/Sidebar.vue` 改 — 加"AI 聊天"切换按钮
- `src/components/common/QuickNoteModal.vue` 改 — B 候选加"深入聊聊 →"按钮
- `src/router/index.ts` 不动（不开新路由）

---

## 📋 执行批次

### 批次 1 · 后端骨架 + 数据层
**目标**：建好 SQLite 表 + Rust models + service 骨架 + commands CRUD（不含 send_message）
- `006_ai_chat.sql`：`ai_conversations` + `ai_messages` 两表，含 `tool_calls/tool_results` 字段（JSON TEXT，预留）
- 注册迁移到 `migrator.rs`
- `models/ai_chat.rs`：`Conversation` / `Message` 结构 + 全部 CRUD
- `services/ai_chat_service.rs`：暴露 service-level 函数
- `commands/chat_commands.rs`：list/get_messages/create/rename/delete/pin/archive/set_model
- 注册到 `commands/mod.rs` 与 `lib.rs::invoke_handler`
- 单测：models 层 CRUD round-trip、pin/archive 状态切换

**验收**：`cargo test` 通过；前端可调 `list_conversations` 拿到空数组

---

### 批次 2 · 流式核心
**目标**：把 AI 子系统从同步升级为支持流式
- `ai/streaming.rs`：`StreamChunk { delta, done, error }`，OpenAI SSE 解析、Claude `stream: true` 解析
- `ai/mod.rs::AIProvider` trait 加 `stream_complete(messages, opts, channel)` + `supports_tools()` 默认 false
- 两个 provider 实现 `stream_complete`
- `AIService::stream_required(messages, opts, channel)`
- `commands/chat_commands.rs::send_message(conversation_id, content, channel)`：写 user → 装上下文 → 创 assistant(streaming) → 推 chunk → 完成/失败/中止三态写库
- `abort_message(conversation_id)`：用 `Arc<Mutex<HashMap<id, AbortHandle>>>` 管理
- 前端 `useChatStore`：订阅 Channel，`streamingMessageId` 拼接，`abortStreaming()`
- 离线/未配置 provider 时**直接报错**（不兜底）

**验收**：临时 UI 触发 `sendMessage`，chunk 增量到达；中止能停；DB 中 user/assistant 两条消息齐全

---

### 批次 3 · 右侧 ChatPanel UI
- `useUIStore` 加 showChat / chatPanelWidth / chatPanelPinned，持久化 localStorage
- `App.vue` 主区右侧挂 `<ChatPanel>`
- `ChatPanel.vue` 三段式（顶部 + 消息区 + 输入区）
- `ConversationDropdown.vue` 顶部下拉 + 三点菜单
- `MessageList.vue` / `MessageBubble.vue`：流式打字效果、复制、失败重试
- `MessageInput.vue` 自动高度 textarea，回车发送 / Shift+回车换行 / 流式时变中止
- `ModelPicker.vue` 切换会话级模型
- Sidebar 加"AI 聊天"按钮

**验收**：完整跑通新建/多轮/切会话/切模型/中止/改名/删/置顶/归档

---

### 批次 4 · 灵感"研究问题版"贯通
- `ai/chat_prompts.rs::build_inspiration_research_system(raw, optimized, tone)`
- `chat_commands::create_from_inspiration(noteId, raw, question)`：创建会话写好 system prompt，**不发消息**
- 前端 `chatStore.createFromInspiration` → 打开面板 + 选中会话 + 预填 question 到输入框
- `QuickNoteModal.vue` B 候选加"深入聊聊 →"按钮
- `InspirationsView.vue` 同步入口

**验收**：从 QuickNote B 候选点击 → 面板打开 → 新会话已建 → 输入框已预填 → 用户回车发送 → 流式回复

---

## ⚠️ 风险与注意事项

1. **流式取消**：reqwest stream 中止需 drop response stream，`tokio::select!` + `AbortHandle`
2. **多模型混用**：旧消息保留各自 `model` 字段；切模型只影响后续轮次，不重置 system
3. **DB 锁**：流式过程中**只在结束/失败/中止时写 DB**，中途增量只走 channel
4. **隐私**：聊天记录全本地；设置页"清空所有 AI 对话"按钮列入后续
5. **离线**：不配置 provider 直接报错，不兜底
6. **低分屏**：≤1200px 下 pinned 模式建议关闭，文档提示
7. **工具协议位**：DB 字段先建，send_message 当前不解析

---

## 📎 参考

- AIService: `src-tauri/src/ai/mod.rs:288`
- OpenAICompatibleProvider: `src-tauri/src/ai/mod.rs:69`
- ClaudeProvider: `src-tauri/src/ai/mod.rs:200`
- ai_profile CRUD 模板: `src-tauri/src/models/ai_profile.rs`
- 灵感模型: `src-tauri/src/models/inspiration.rs:10`
- "研究问题版" prompt: `src-tauri/src/ai/prompt_templates.rs:114`
- 离线兜底: `src-tauri/src/ai/mod.rs:388`
- QuickNote 消费: `src/components/common/QuickNoteModal.vue:24`
- 主布局: `src/App.vue:128`
- 迁移注册: `src-tauri/src/db/migrator.rs:21`
- invoke_handler: `src-tauri/src/lib.rs:114`

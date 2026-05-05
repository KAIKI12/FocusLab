-- 006 · AI 聊天会话与消息
--
-- ai_conversations: 一条对话会话(支持来源标记: 灵感/任务/目标/手动)
-- ai_messages:      会话下的消息(role + content + 流式状态 + 工具协议位预留)
--
-- 设计要点:
-- * provider/api_format 是创建时的快照,中途切 profile 不影响历史会话上下文格式
-- * model 是会话级覆盖(空表示沿用全局 active profile 的 fast/strong)
-- * tool_calls / tool_results 当前不被 send_message 消费,只占位等后续扩展
-- * pinned + archived 双布尔位状态(归档不删除,可恢复)

CREATE TABLE ai_conversations (
  id             TEXT PRIMARY KEY,
  title          TEXT NOT NULL DEFAULT '新会话',
  origin_type    TEXT NOT NULL DEFAULT 'manual',   -- 'manual' | 'inspiration' | 'task' | 'goal'
  origin_id      TEXT,
  provider       TEXT NOT NULL DEFAULT '',
  api_format     TEXT NOT NULL DEFAULT '',
  model          TEXT NOT NULL DEFAULT '',
  system_prompt  TEXT NOT NULL DEFAULT '',
  message_count  INTEGER NOT NULL DEFAULT 0,
  pinned         INTEGER NOT NULL DEFAULT 0,
  archived       INTEGER NOT NULL DEFAULT 0,
  created_at     TEXT NOT NULL,
  updated_at     TEXT NOT NULL
);

CREATE TABLE ai_messages (
  id              TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  role            TEXT NOT NULL,                   -- 'system' | 'user' | 'assistant' | 'tool'
  content         TEXT NOT NULL DEFAULT '',
  model           TEXT,
  status          TEXT NOT NULL DEFAULT 'ok',      -- 'ok' | 'streaming' | 'error' | 'aborted'
  error_message   TEXT,
  tool_calls      TEXT,                            -- JSON, 预留
  tool_results    TEXT,                            -- JSON, 预留
  tokens_in       INTEGER,
  tokens_out      INTEGER,
  created_at      TEXT NOT NULL,
  FOREIGN KEY (conversation_id) REFERENCES ai_conversations(id) ON DELETE CASCADE
);

CREATE INDEX idx_ai_conversations_pinned_updated
  ON ai_conversations(pinned DESC, updated_at DESC);

CREATE INDEX idx_ai_conversations_origin
  ON ai_conversations(origin_type, origin_id);

CREATE INDEX idx_ai_messages_conversation_created
  ON ai_messages(conversation_id, created_at);

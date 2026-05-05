CREATE TABLE ai_chat_profiles (
  id           TEXT PRIMARY KEY,
  name         TEXT NOT NULL,
  provider     TEXT NOT NULL,
  api_format   TEXT NOT NULL,
  base_url     TEXT NOT NULL,
  api_key      TEXT NOT NULL,
  model_fast   TEXT NOT NULL DEFAULT '',
  model_strong TEXT NOT NULL DEFAULT '',
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL
);

CREATE TABLE ai_embedding_profiles (
  id         TEXT PRIMARY KEY,
  name       TEXT NOT NULL,
  base_url   TEXT NOT NULL,
  api_key    TEXT NOT NULL,
  model      TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX idx_ai_chat_profiles_updated ON ai_chat_profiles(updated_at DESC);
CREATE INDEX idx_ai_embedding_profiles_updated ON ai_embedding_profiles(updated_at DESC);

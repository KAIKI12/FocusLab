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

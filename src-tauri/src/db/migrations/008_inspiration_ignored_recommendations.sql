CREATE TABLE inspiration_ignored_recommendations (
  source_id    TEXT NOT NULL,
  candidate_id TEXT NOT NULL,
  relation     TEXT NOT NULL,
  ignored_at   TEXT NOT NULL,
  FOREIGN KEY (source_id) REFERENCES inspirations(id) ON DELETE CASCADE,
  FOREIGN KEY (candidate_id) REFERENCES inspirations(id) ON DELETE CASCADE,
  UNIQUE(source_id, candidate_id, relation)
);

CREATE INDEX idx_inspiration_ignored_recommendations_source
  ON inspiration_ignored_recommendations(source_id, ignored_at DESC);

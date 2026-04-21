-- 003_milestone_v2 · 里程碑 v2 扩展(gap-audit §P1.1)
-- 两个改动:
--   1. milestones 表加 target_date (预计完成日期,DATE,可空)
--   2. 新建 milestone_notes 表(里程碑带日期的多条备注 / 科研日志)

ALTER TABLE milestones ADD COLUMN target_date DATE;

CREATE TABLE milestone_notes (
    id              TEXT PRIMARY KEY,
    milestone_id    TEXT NOT NULL REFERENCES milestones(id) ON DELETE CASCADE,
    text            TEXT NOT NULL,
    created_at      DATETIME NOT NULL
);

CREATE INDEX idx_milestone_notes_milestone_id
    ON milestone_notes(milestone_id, created_at DESC);

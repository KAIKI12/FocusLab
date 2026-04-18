-- 001_init · FocusLab 初始 schema (v1.1 + v1.2.2)
-- 对齐 docs/04 §7.2 完整数据表设计。

-- ========== 长线目标 ==========
CREATE TABLE goals (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      DATETIME NOT NULL,
    updated_at      DATETIME NOT NULL,
    completed_at    DATETIME,
    target_date     DATETIME,
    sort_order      INTEGER NOT NULL DEFAULT 0
);

-- ========== 里程碑 ==========
CREATE TABLE milestones (
    id              TEXT PRIMARY KEY,
    goal_id         TEXT NOT NULL REFERENCES goals(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    description     TEXT,
    status          TEXT NOT NULL DEFAULT 'pending',
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_at      DATETIME NOT NULL,
    updated_at      DATETIME NOT NULL,
    completed_at    DATETIME
);

-- ========== 任务分类 ==========
CREATE TABLE categories (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    icon            TEXT,
    color           TEXT,
    is_default      BOOLEAN NOT NULL DEFAULT 0,
    sort_order      INTEGER NOT NULL DEFAULT 0
);

-- ========== 任务 ==========
-- v1.1: plan_date 已由 daily_task_assignments 替代;status 只表示全局生命周期;
-- consecutive_carry_over_days 更名为 carry_over_count
CREATE TABLE tasks (
    id                      TEXT PRIMARY KEY,
    name                    TEXT NOT NULL,
    description             TEXT,
    category_id             TEXT REFERENCES categories(id),
    milestone_id            TEXT REFERENCES milestones(id),
    quadrant                TEXT NOT NULL DEFAULT 'important_not_urgent',
    urgency_level           TEXT NOT NULL DEFAULT 'ongoing',
    status                  TEXT NOT NULL DEFAULT 'pending',
    estimated_minutes       INTEGER,
    actual_minutes          INTEGER DEFAULT 0,
    due_date                DATE,
    due_reminder_sent_date  DATE,
    is_recurring            BOOLEAN NOT NULL DEFAULT 0,
    recurrence_rule         TEXT,
    source                  TEXT NOT NULL DEFAULT 'manual',
    is_background           BOOLEAN NOT NULL DEFAULT 0,
    shelved_at              DATETIME,
    shelve_reason           TEXT,
    carry_over_count        INTEGER DEFAULT 0,
    last_assigned_date      DATE,
    created_at              DATETIME NOT NULL,
    updated_at              DATETIME NOT NULL,
    completed_at            DATETIME,
    sort_order              INTEGER NOT NULL DEFAULT 0
);

-- ========== 标签 ==========
CREATE TABLE tags (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,
    color           TEXT
);

CREATE TABLE task_tags (
    task_id         TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    tag_id          TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, tag_id)
);

-- ========== 专注时段 ==========
CREATE TABLE sessions (
    id                          TEXT PRIMARY KEY,
    task_id                     TEXT NOT NULL REFERENCES tasks(id),
    start_time                  DATETIME NOT NULL,
    end_time                    DATETIME,
    planned_duration_minutes    INTEGER,
    actual_duration_minutes     INTEGER,
    mode                        TEXT NOT NULL DEFAULT 'pomodoro',
    pomodoro_preset             TEXT,
    status                      TEXT NOT NULL DEFAULT 'in_progress',
    is_manual_entry             BOOLEAN NOT NULL DEFAULT 0,
    abandon_reason              TEXT,
    created_at                  DATETIME NOT NULL
);

-- ========== 中断 ==========
CREATE TABLE interruptions (
    id                  TEXT PRIMARY KEY,
    session_id          TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    start_time          DATETIME NOT NULL,
    end_time            DATETIME,
    duration_minutes    INTEGER,
    reason              TEXT,
    note                TEXT,
    created_at          DATETIME NOT NULL
);

-- ========== 每日计划 ==========
CREATE TABLE daily_plans (
    id                      TEXT PRIMARY KEY,
    plan_date               DATE NOT NULL UNIQUE,
    energy_level            TEXT,
    fixed_schedule          TEXT,
    available_hours         REAL,
    note                    TEXT,
    guided_flow_completed   BOOLEAN DEFAULT 0,
    plan_locked_at          DATETIME,
    target_focus_minutes    INTEGER,
    day_boundary_date       DATE,
    created_at              DATETIME NOT NULL,
    updated_at              DATETIME NOT NULL
);

-- ========== 日结算 ==========
CREATE TABLE settlements (
    id                      TEXT PRIMARY KEY,
    settle_date             DATE NOT NULL UNIQUE,
    total_tasks             INTEGER NOT NULL DEFAULT 0,
    completed_tasks         INTEGER NOT NULL DEFAULT 0,
    extra_tasks             INTEGER NOT NULL DEFAULT 0,
    shelved_tasks           INTEGER NOT NULL DEFAULT 0,
    completion_rate         REAL NOT NULL DEFAULT 0,
    total_focus_minutes     INTEGER NOT NULL DEFAULT 0,
    total_pomodoros         INTEGER NOT NULL DEFAULT 0,
    total_interruptions     INTEGER NOT NULL DEFAULT 0,
    grade                   TEXT NOT NULL,
    time_distribution       TEXT,
    longest_focus_task_id   TEXT REFERENCES tasks(id),
    longest_focus_minutes   INTEGER,
    ai_summary              TEXT,
    ai_suggestion           TEXT,
    user_reflection         TEXT,
    trigger_type            TEXT NOT NULL DEFAULT 'manual',
    created_at              DATETIME NOT NULL
);

-- ========== AI 反馈 ==========
CREATE TABLE ai_feedbacks (
    id              TEXT PRIMARY KEY,
    type            TEXT NOT NULL,
    context         TEXT,
    prompt_used     TEXT,
    response        TEXT NOT NULL,
    user_rating     INTEGER,
    model_used      TEXT,
    token_count     INTEGER,
    created_at      DATETIME NOT NULL
);

-- ========== 设置 (KV) ==========
CREATE TABLE settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  DATETIME NOT NULL
);

-- ========== 埋点事件 ==========
CREATE TABLE analytics_events (
    id          TEXT PRIMARY KEY,
    event_name  TEXT NOT NULL,
    event_data  TEXT,
    created_at  DATETIME NOT NULL
);

-- ========== v1.1: 每日任务分配 ==========
CREATE TABLE daily_task_assignments (
    id              TEXT PRIMARY KEY,
    plan_date       DATE NOT NULL,
    task_id         TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    is_planned      BOOLEAN NOT NULL DEFAULT 1,
    source          TEXT NOT NULL DEFAULT 'manual',
    day_status      TEXT NOT NULL DEFAULT 'pending',
    added_at        DATETIME NOT NULL,
    completed_at    DATETIME,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    UNIQUE(plan_date, task_id)
);

-- ========== v1.1: 计时器状态 (单行崩溃恢复表) ==========
CREATE TABLE timer_state (
    id              TEXT PRIMARY KEY DEFAULT 'current',
    task_id         TEXT REFERENCES tasks(id),
    session_id      TEXT REFERENCES sessions(id),
    start_time      DATETIME,
    elapsed_seconds INTEGER DEFAULT 0,
    planned_seconds INTEGER,
    mode            TEXT,
    pomodoro_preset TEXT,
    status          TEXT,
    pomodoro_count  INTEGER DEFAULT 0,
    is_break        BOOLEAN DEFAULT 0,
    break_remaining INTEGER,
    updated_at      DATETIME NOT NULL
);

INSERT INTO timer_state (id, status, updated_at)
VALUES ('current', 'idle', datetime('now'));

-- ========== v1.1: 每周固定日程模板 ==========
CREATE TABLE weekly_schedule_templates (
    id          TEXT PRIMARY KEY,
    day_of_week INTEGER NOT NULL,
    start_time  TEXT NOT NULL,
    end_time    TEXT NOT NULL,
    name        TEXT NOT NULL,
    is_active   BOOLEAN NOT NULL DEFAULT 1,
    color       TEXT,
    sort_order  INTEGER DEFAULT 0,
    created_at  DATETIME NOT NULL,
    updated_at  DATETIME NOT NULL
);

-- ========== v1.1: 任务复盘 ==========
CREATE TABLE task_reflections (
    id                  TEXT PRIMARY KEY,
    task_id             TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    plan_date           DATE NOT NULL,
    planned_minutes     INTEGER,
    actual_minutes      INTEGER,
    overtime_reason     TEXT,
    note                TEXT,
    created_at          DATETIME NOT NULL
);

-- ========== 索引 ==========
CREATE INDEX idx_tasks_status         ON tasks(status);
CREATE INDEX idx_tasks_milestone      ON tasks(milestone_id);
CREATE INDEX idx_tasks_category       ON tasks(category_id);
CREATE INDEX idx_tasks_quadrant       ON tasks(quadrant);
CREATE INDEX idx_tasks_urgency        ON tasks(urgency_level);
CREATE INDEX idx_tasks_due_date       ON tasks(due_date);
CREATE INDEX idx_tasks_last_assigned  ON tasks(last_assigned_date);
CREATE INDEX idx_sessions_task        ON sessions(task_id);
CREATE INDEX idx_sessions_start       ON sessions(start_time);
CREATE INDEX idx_sessions_status      ON sessions(status);
CREATE INDEX idx_milestones_goal      ON milestones(goal_id);
CREATE INDEX idx_milestones_status    ON milestones(status);
CREATE INDEX idx_goals_status         ON goals(status);
CREATE INDEX idx_interruptions_session ON interruptions(session_id);
CREATE INDEX idx_settlements_date     ON settlements(settle_date);
CREATE INDEX idx_ai_feedbacks_type    ON ai_feedbacks(type);
CREATE INDEX idx_ai_feedbacks_date    ON ai_feedbacks(created_at);
CREATE INDEX idx_ae_name              ON analytics_events(event_name);
CREATE INDEX idx_ae_time              ON analytics_events(created_at);
CREATE INDEX idx_dta_plan_date        ON daily_task_assignments(plan_date);
CREATE INDEX idx_dta_task_id          ON daily_task_assignments(task_id);
CREATE INDEX idx_dta_status           ON daily_task_assignments(day_status);
CREATE INDEX idx_wst_day              ON weekly_schedule_templates(day_of_week);
CREATE INDEX idx_tr_task              ON task_reflections(task_id);
CREATE INDEX idx_tr_date              ON task_reflections(plan_date);
CREATE INDEX idx_tr_reason            ON task_reflections(overtime_reason);

-- ========== 预置默认设置项 ==========
INSERT INTO settings (key, value, updated_at) VALUES
    ('day_boundary_hour', '4',           datetime('now')),
    ('pomodoro_preset',   'classic_25',  datetime('now')),
    ('theme',             'light',       datetime('now')),
    ('accent',            'default',     datetime('now'));

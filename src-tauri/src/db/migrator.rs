//! 简易迁移执行器。
//!
//! - 版本表: `schema_migrations(version TEXT PRIMARY KEY, applied_at DATETIME)`
//! - 文件名即版本号,字典序从老到新
//! - 每条迁移包在一个事务里,失败整体回滚,不记录 version
//! - 只做正向迁移(单向),对齐 docs/04 §13
//! - 末尾有一个 `ensure_task_columns` heal 步骤,补齐 dev 期 `001_init.sql`
//!   曾被追加过新列、但老数据库已标记 applied 而跳过的缺失列(幂等)

use std::collections::HashSet;

use rusqlite::{params, Connection};

use crate::utils::errors::AppResult;

struct Migration {
    version: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: "001_init",
        sql: include_str!("migrations/001_init.sql"),
    },
    Migration {
        version: "002_mood_checkin",
        sql: include_str!("migrations/002_mood_checkin.sql"),
    },
    Migration {
        version: "003_milestone_v2",
        sql: include_str!("migrations/003_milestone_v2.sql"),
    },
    Migration {
        version: "004_inspiration_graph",
        sql: include_str!("migrations/004_inspiration_graph.sql"),
    },
    Migration {
        version: "005_ai_profiles",
        sql: include_str!("migrations/005_ai_profiles.sql"),
    },
    Migration {
        version: "006_ai_chat",
        sql: include_str!("migrations/006_ai_chat.sql"),
    },
    Migration {
        version: "007_ai_selected_models",
        sql: include_str!("migrations/007_ai_selected_models.sql"),
    },
    Migration {
        version: "008_inspiration_ignored_recommendations",
        sql: include_str!("migrations/008_inspiration_ignored_recommendations.sql"),
    },
    Migration {
        version: "009_inspiration_images",
        sql: include_str!("migrations/009_inspiration_images.sql"),
    },
];

pub fn run(conn: &mut Connection) -> AppResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version    TEXT PRIMARY KEY,
            applied_at DATETIME NOT NULL
         )",
        [],
    )?;

    for m in MIGRATIONS {
        let already: i64 = conn.query_row(
            "SELECT COUNT(1) FROM schema_migrations WHERE version = ?1",
            params![m.version],
            |row| row.get(0),
        )?;
        if already > 0 {
            tracing::debug!("migration {} already applied", m.version);
            continue;
        }

        tracing::info!("applying migration {}", m.version);
        let tx = conn.transaction()?;
        tx.execute_batch(m.sql).map_err(|e| {
            tracing::error!("migration {} failed: {}", m.version, e);
            e
        })?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, datetime('now'))",
            params![m.version],
        )?;
        tx.commit()?;
        tracing::info!("migration {} applied", m.version);
    }

    ensure_task_columns(conn)?;

    crate::models::ai_profile::ensure_default_profile(conn)?;

    Ok(())
}

/// 幂等地补齐 `tasks` 表应有但本地库缺失的列。
///
/// 背景:dev 期间 `001_init.sql` 被追加过列(is_background 等),
/// 但更早跑过迁移的开发库会把 `001_init` 标记为 applied,新列永远不会被补。
/// 这里对照 schema 定义逐列 `ALTER TABLE ADD COLUMN`,已存在的自然跳过。
///
/// NOT NULL 列必须带常量 DEFAULT,否则 SQLite 会拒绝。所有字段定义直接照搬
/// `migrations/001_init.sql` 的 tasks 段,以保持单一真相。
fn ensure_task_columns(conn: &Connection) -> AppResult<()> {
    let expected: &[(&str, &str)] = &[
        ("category_id", "TEXT"),
        ("milestone_id", "TEXT"),
        ("urgency_level", "TEXT NOT NULL DEFAULT 'ongoing'"),
        ("estimated_minutes", "INTEGER"),
        ("actual_minutes", "INTEGER DEFAULT 0"),
        ("due_date", "DATE"),
        ("due_reminder_sent_date", "DATE"),
        ("is_recurring", "BOOLEAN NOT NULL DEFAULT 0"),
        ("recurrence_rule", "TEXT"),
        ("source", "TEXT NOT NULL DEFAULT 'manual'"),
        ("is_background", "BOOLEAN NOT NULL DEFAULT 0"),
        ("shelved_at", "DATETIME"),
        ("shelve_reason", "TEXT"),
        ("carry_over_count", "INTEGER DEFAULT 0"),
        ("last_assigned_date", "DATE"),
        ("completed_at", "DATETIME"),
        ("sort_order", "INTEGER NOT NULL DEFAULT 0"),
    ];

    let mut existing: HashSet<String> = HashSet::new();
    {
        let mut stmt = conn.prepare("PRAGMA table_info(tasks)")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(1))?;
        for r in rows {
            existing.insert(r?);
        }
    }

    if existing.is_empty() {
        // 未建表(干净库 + migration 刚创建它),PRAGMA 在建表事务外也可能空;
        // 保守起见不作任何操作,让正规 migration 负责。
        return Ok(());
    }

    for (col, col_def) in expected {
        if !existing.contains(*col) {
            let sql = format!("ALTER TABLE tasks ADD COLUMN {col} {col_def}");
            conn.execute(&sql, [])?;
            tracing::warn!("schema heal: tasks.{} added via ALTER", col);
        }
    }

    Ok(())
}

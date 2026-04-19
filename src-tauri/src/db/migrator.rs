//! 简易迁移执行器。
//!
//! - 版本表: `schema_migrations(version TEXT PRIMARY KEY, applied_at DATETIME)`
//! - 文件名即版本号,字典序从老到新
//! - 每条迁移包在一个事务里,失败整体回滚,不记录 version
//! - 只做正向迁移(单向),对齐 docs/04 §13

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

    Ok(())
}

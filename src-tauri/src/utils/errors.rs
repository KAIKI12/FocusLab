//! 统一错误类型。
//!
//! `impl Serialize` 把错误透传给前端,对应的 JS 侧会拿到一个字符串 message。
//! Tauri 2 要求 `#[tauri::command]` 返回类型的 `Err` 必须是 `Serialize`,
//! 这里直接序列化成 `Display` 文本。

use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("tauri: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Custom(String),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

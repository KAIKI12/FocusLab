//! AI provider 流式解析。
//!
//! 设计:
//! - `StreamChunk` 是推到前端 Tauri Channel 的统一帧格式
//! - `stream_sse` 是通用的 SSE 行解析循环;不同 provider 通过传不同的
//!   `extract` 函数(从一个 SSE 事件 data 里抽出 delta 文本)复用它
//! - abort 通过 `Arc<AtomicBool>` 协作:外部置 true 后循环立即跳出,
//!   reqwest 流被 drop,连接关闭。无需 select! 也无需新依赖。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::utils::errors::{AppError, AppResult};

/// 推送给前端 Channel 的单帧。
///
/// - `delta`: 此次推送的文本增量(可空)
/// - `done`: 是否最终帧
/// - `error`: 失败时的简短描述(用户可见)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamChunk {
    pub delta: String,
    pub done: bool,
    pub error: Option<String>,
}

impl StreamChunk {
    pub fn delta(s: &str) -> Self {
        Self {
            delta: s.to_string(),
            done: false,
            error: None,
        }
    }
    pub fn done() -> Self {
        Self {
            delta: String::new(),
            done: true,
            error: None,
        }
    }
    pub fn aborted() -> Self {
        Self {
            delta: String::new(),
            done: true,
            error: Some("aborted".into()),
        }
    }
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            delta: String::new(),
            done: true,
            error: Some(msg.into()),
        }
    }
}

/// 通用 SSE 流循环。
///
/// `extract`(单个事件 data 文本) ->
///   - `None` 跳过
///   - `Some("")` [DONE] 哨兵,正常结束
///   - `Some(text)` 增量文本,push 到 accumulator + 调 on_chunk
///
/// 返回累积的完整文本。abort 触发或 stream 自然结束都会正常返回 Ok;
/// 网络/解析错误返回 Err。调用方根据这个 + abort flag 写最终 status。
pub async fn stream_sse(
    response: reqwest::Response,
    extract: impl Fn(&str) -> Option<String>,
    on_delta: &mpsc::UnboundedSender<String>,
    abort: Arc<AtomicBool>,
) -> AppResult<String> {
    let mut buffer = String::new();
    let mut accumulated = String::new();
    let mut byte_stream = response.bytes_stream();

    while let Some(chunk_result) = byte_stream.next().await {
        if abort.load(Ordering::Relaxed) {
            break;
        }
        let bytes = chunk_result.map_err(|e| AppError::Custom(format!("流式读取失败: {e}")))?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|e| AppError::Custom(format!("流式 UTF-8 解码失败: {e}")))?;
        buffer.push_str(text);

        // SSE 事件以空行(\n\n 或 \r\n\r\n)分隔
        loop {
            let eol = buffer.find("\n\n").or_else(|| buffer.find("\r\n\r\n"));
            let Some(idx) = eol else { break };
            let sep_len = if buffer[idx..].starts_with("\r\n\r\n") {
                4
            } else {
                2
            };
            let raw_event: String = buffer.drain(..idx + sep_len).collect();

            // 一个事件可能含 event:/id:/data: 多行,只关心 data:
            let mut data_parts: Vec<&str> = Vec::new();
            for line in raw_event.lines() {
                if let Some(rest) = line.strip_prefix("data:") {
                    data_parts.push(rest.strip_prefix(' ').unwrap_or(rest));
                }
            }
            if data_parts.is_empty() {
                continue;
            }
            let data = data_parts.join("\n");
            match extract(&data) {
                Some(delta) if delta.is_empty() => return Ok(accumulated),
                Some(delta) => {
                    accumulated.push_str(&delta);
                    let _ = on_delta.send(delta);
                }
                None => {}
            }
            if abort.load(Ordering::Relaxed) {
                return Ok(accumulated);
            }
        }
    }

    Ok(accumulated)
}

/// OpenAI 兼容: `data: {"choices":[{"delta":{"content":"x"}}]}` / `[DONE]`
pub fn openai_extract(data: &str) -> Option<String> {
    let trimmed = data.trim();
    if trimmed == "[DONE]" {
        return Some(String::new());
    }
    let v: serde_json::Value = serde_json::from_str(trimmed).ok()?;
    let content = v.get("choices")?.get(0)?.get("delta")?.get("content")?;
    let text = content.as_str().unwrap_or("");
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

/// Claude messages stream: 关注 `content_block_delta.delta.text` + `message_stop`
pub fn claude_extract(data: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(data.trim()).ok()?;
    match v.get("type")?.as_str()? {
        "content_block_delta" => {
            let text = v.get("delta")?.get("text")?.as_str()?;
            if text.is_empty() {
                None
            } else {
                Some(text.to_string())
            }
        }
        "message_stop" => Some(String::new()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_extract_pulls_content_delta() {
        let raw = r#"{"choices":[{"index":0,"delta":{"content":"Hi"}}]}"#;
        assert_eq!(openai_extract(raw), Some("Hi".into()));
    }

    #[test]
    fn openai_extract_recognizes_done_sentinel() {
        assert_eq!(openai_extract("[DONE]"), Some(String::new()));
    }

    #[test]
    fn openai_extract_skips_role_only_first_chunk() {
        let raw = r#"{"choices":[{"index":0,"delta":{"role":"assistant","content":""}}]}"#;
        assert_eq!(openai_extract(raw), None);
    }

    #[test]
    fn claude_extract_pulls_text_delta() {
        let raw = r#"{"type":"content_block_delta","delta":{"type":"text_delta","text":"Hello"}}"#;
        assert_eq!(claude_extract(raw), Some("Hello".into()));
    }

    #[test]
    fn claude_extract_recognizes_message_stop() {
        let raw = r#"{"type":"message_stop"}"#;
        assert_eq!(claude_extract(raw), Some(String::new()));
    }

    #[test]
    fn claude_extract_ignores_unrelated_events() {
        let raw = r#"{"type":"message_start","message":{}}"#;
        assert_eq!(claude_extract(raw), None);
    }
}

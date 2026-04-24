//! AI 服务层 — 多 provider 抽象 + OpenAI-compatible 实现。
//!
//! 架构(对齐 docs/03):
//! - `AIProvider` trait: `complete(messages, opts) -> Result<String>`
//! - `OpenAICompatibleProvider`: 走 OpenAI `/v1/chat/completions` 格式,
//!   兼容 OpenAI / DeepSeek / Zhipu / 本地代理
//! - `ClaudeProvider`: 走 Anthropic `/v1/messages` 原生格式
//! - `AIService`: 持有当前 provider,由 settings KV 驱动切换
//! - prompt_templates: 预置结算叙事 / 任务拆解 / 每日建议模板

pub mod prompt_templates;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::utils::errors::{AppError, AppResult};

// ---------- 数据结构 ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct CompletionOptions {
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
}

// ---------- Provider Trait ----------

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn complete(
        &self,
        messages: Vec<Message>,
        opts: CompletionOptions,
    ) -> AppResult<String>;
}

// ---------- OpenAICompatibleProvider (OpenAI-format) ----------

pub struct OpenAICompatibleProvider {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAICompatibleProvider {
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

#[async_trait]
impl AIProvider for OpenAICompatibleProvider {
    async fn complete(
        &self,
        messages: Vec<Message>,
        opts: CompletionOptions,
    ) -> AppResult<String> {
        let url = format!("{}/v1/chat/completions", self.base_url);
        let body = ChatRequest {
            model: self.model.clone(),
            messages,
            temperature: opts.temperature,
            max_tokens: opts.max_tokens,
        };

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| AppError::Custom(format!("AI 请求失败: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Custom(format!(
                "AI 返回 {status}: {text}"
            )));
        }

        let data: ChatResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Custom(format!("AI 响应解析失败: {e}")))?;

        data.choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AppError::Custom("AI 返回空结果".into()))
    }
}

// ---------- ClaudeProvider (Anthropic Messages API) ----------

pub struct ClaudeProvider {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl ClaudeProvider {
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Deserialize)]
struct ClaudeContent {
    text: Option<String>,
}

#[async_trait]
impl AIProvider for ClaudeProvider {
    async fn complete(
        &self,
        messages: Vec<Message>,
        opts: CompletionOptions,
    ) -> AppResult<String> {
        let url = format!("{}/v1/messages", self.base_url);
        let body = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: opts.max_tokens.unwrap_or(1024),
            messages: messages
                .into_iter()
                .map(|message| ClaudeMessage {
                    role: message.role,
                    content: message.content,
                })
                .collect(),
        };

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| AppError::Custom(format!("AI 请求失败: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Custom(format!(
                "AI 返回 {status}: {text}"
            )));
        }

        let data: ClaudeResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Custom(format!("AI 响应解析失败: {e}")))?;

        data.content
            .first()
            .and_then(|c| c.text.clone())
            .ok_or_else(|| AppError::Custom("AI 返回空结果".into()))
    }
}

// ---------- AIService ----------

#[derive(Default)]
pub struct AIService {
    provider: tokio::sync::RwLock<Option<Box<dyn AIProvider>>>,
}

impl AIService {
    pub fn new() -> Self {
        Self {
            provider: tokio::sync::RwLock::new(None),
        }
    }

    /// 根据 settings KV 配置 provider
    pub async fn configure(
        &self,
        provider_type: &str,
        api_format: &str,
        base_url: &str,
        api_key: &str,
        model: &str,
    ) {
        let p: Box<dyn AIProvider> = match api_format {
            "claude" => Box::new(ClaudeProvider::new(
                if base_url.is_empty() { "https://api.anthropic.com" } else { base_url },
                api_key,
                if model.is_empty() { "claude-3-5-sonnet-latest" } else { model },
            )),
            _ => match provider_type {
                "ollama" => Box::new(OpenAICompatibleProvider::new(
                    if base_url.is_empty() { "http://localhost:11434" } else { base_url },
                    "",
                    if model.is_empty() { "llama3" } else { model },
                )),
                _ => Box::new(OpenAICompatibleProvider::new(
                    if base_url.is_empty() { "https://api.openai.com" } else { base_url },
                    api_key,
                    if model.is_empty() { "gpt-4o-mini" } else { model },
                )),
            },
        };
        *self.provider.write().await = Some(p);
    }

    pub async fn complete(
        &self,
        messages: Vec<Message>,
        opts: CompletionOptions,
    ) -> AppResult<String> {
        let guard = self.provider.read().await;
        let p = match guard.as_ref() {
            Some(p) => p,
            None => {
                // 离线降级：返回预设话术
                return Ok(Self::offline_fallback(&messages));
            }
        };
        match p.complete(messages.clone(), opts).await {
            Ok(result) => Ok(result),
            Err(_) => {
                // API 调用失败也降级
                Ok(Self::offline_fallback(&messages))
            }
        }
    }

    /// 离线降级预设话术
    fn offline_fallback(messages: &[Message]) -> String {
        let last = messages.last().map(|m| m.content.as_str()).unwrap_or("");
        if last.contains("总结") || last.contains("表现") {
            "今天辛苦了,每一步推进都有价值。明天继续保持节奏。".into()
        } else if last.contains("拆解") || last.contains("子任务") {
            r#"[{"name":"第一步：分析需求","estimatedMinutes":30,"quadrant":"important_not_urgent"},{"name":"第二步：实现核心","estimatedMinutes":60,"quadrant":"important_not_urgent"},{"name":"第三步：测试验证","estimatedMinutes":30,"quadrant":"important_not_urgent"}]"#.into()
        } else if last.contains("建议") || last.contains("推荐") {
            "建议优先处理紧急重要的任务,上午精力最好时段推进核心工作。".into()
        } else if last.contains("象限") || last.contains("分类") {
            "important_not_urgent".into()
        } else if last.contains("速记") || last.contains("整理") || last.contains("candidates") {
            Self::default_quick_note_for(messages)
        } else {
            "保持节奏,每天进步一点点。".into()
        }
    }

    pub async fn is_configured(&self) -> bool {
        self.provider.read().await.is_some()
    }

    fn default_quick_note_for(messages: &[Message]) -> String {
        let raw = messages.last().map(|m| m.content.as_str()).unwrap_or("你的想法");
        let short: String = raw.chars().take(80).collect();
        format!(
            r#"{{"candidates":[{{"label":"A","style":"task","styleName":"偏任务导向","text":"{}","quadrant":"important_not_urgent"}},{{"label":"B","style":"note","styleName":"偏笔记梳理","text":"{}"}},{{"label":"C","style":"checklist","styleName":"偏简洁行动清单","text":"{}"}}]}}"#,
            short, short, short
        )
    }
}

/// AI 输出校验器 — 防止幻觉
pub struct ResponseValidator;

impl ResponseValidator {
    /// 校验任务拆解结果
    pub fn validate_decompose(raw: &str) -> String {
        // 尝试解析 JSON
        let parsed: Result<Vec<serde_json::Value>, _> = serde_json::from_str(raw);
        match parsed {
            Ok(mut arr) => {
                // 数量校验：3-7
                if arr.len() < 3 { return Self::default_decompose(); }
                if arr.len() > 7 { arr.truncate(7); }

                // 逐项校验
                for item in &mut arr {
                    // 时间校验：5-480
                    if let Some(min) = item.get("estimatedMinutes").and_then(|v| v.as_i64()) {
                        if min < 5 || min > 480 {
                            item["estimatedMinutes"] = serde_json::json!(30);
                        }
                    }
                    // 象限校验
                    if let Some(q) = item.get("quadrant").and_then(|v| v.as_str()) {
                        let valid = ["important_urgent", "important_not_urgent", "not_important_urgent", "not_important_not_urgent"];
                        if !valid.contains(&q) {
                            item["quadrant"] = serde_json::json!("important_not_urgent");
                        }
                    }
                }
                serde_json::to_string(&arr).unwrap_or_else(|_| Self::default_decompose())
            }
            Err(_) => Self::default_decompose(),
        }
    }

    /// 校验结算叙事（禁用词过滤）
    pub fn validate_narrative(raw: &str) -> String {
        let forbidden = ["失败", "落后", "拖延", "懒惰", "差劲"];
        let mut result = raw.to_string();
        for word in &forbidden {
            result = result.replace(word, "调整");
        }
        // 长度校验
        if result.chars().count() > 200 {
            result = result.chars().take(200).collect();
        }
        if result.chars().count() < 5 {
            return "今天辛苦了，保持节奏。".into();
        }
        result
    }

    fn default_decompose() -> String {
        r#"[{"name":"分析需求","estimatedMinutes":30,"quadrant":"important_not_urgent"},{"name":"实现核心","estimatedMinutes":60,"quadrant":"important_not_urgent"},{"name":"验证测试","estimatedMinutes":30,"quadrant":"important_not_urgent"}]"#.into()
    }

    /// 校验速记便签优化结果
    pub fn validate_quick_note(raw: &str) -> String {
        let cleaned = raw.trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let parsed: Result<serde_json::Value, _> = serde_json::from_str(cleaned);
        match parsed {
            Ok(val) => {
                if let Some(arr) = val.get("candidates").and_then(|v| v.as_array()) {
                    if arr.len() >= 3 {
                        let mut result = val.clone();
                        if let Some(candidates) = result.get_mut("candidates").and_then(|v| v.as_array_mut()) {
                            candidates.truncate(3);
                            for c in candidates.iter_mut() {
                                if let Some(text) = c.get("text").and_then(|v| v.as_str()) {
                                    if text.chars().count() > 200 {
                                        let short: String = text.chars().take(200).collect();
                                        c["text"] = serde_json::json!(short);
                                    }
                                }
                                if c.get("style").and_then(|v| v.as_str()) == Some("task") {
                                    if let Some(q) = c.get("quadrant").and_then(|v| v.as_str()) {
                                        let valid = ["important_urgent", "important_not_urgent", "not_important_urgent", "not_important_not_urgent"];
                                        if !valid.contains(&q) {
                                            c["quadrant"] = serde_json::json!("important_not_urgent");
                                        }
                                    } else {
                                        c["quadrant"] = serde_json::json!("important_not_urgent");
                                    }
                                }
                            }
                        }
                        return serde_json::to_string(&result).unwrap_or_else(|_| cleaned.to_string());
                    }
                }
                cleaned.to_string()
            }
            Err(_) => cleaned.to_string(),
        }
    }
}

//! AI 服务层 — 多 provider 抽象 + OpenAI-compatible 实现。
//!
//! 架构(对齐 docs/03):
//! - `AIProvider` trait: `complete(messages, opts) -> Result<String>` + `embed(texts)`
//! - `OpenAICompatibleProvider`: 走 OpenAI `/v1/chat/completions` 与 `/v1/embeddings`
//! - `ClaudeProvider`: 走 Anthropic `/v1/messages` 原生格式
//! - `AIService`: 持有当前 chat provider 与 embedding provider
//! - prompt_templates: 预置结算叙事 / 任务拆解 / 每日建议模板

pub mod local_quotes;
pub mod prompt_templates;
pub mod streaming;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::ai::streaming::stream_sse;

use crate::utils::errors::{AppError, AppResult};

/// 浏览器化 default headers - 部分 provider(如 voidai) 在 Cloudflare 后面,
/// 没有 UA / Accept 的请求会被 Managed Challenge 拦成 403。
fn browser_default_headers() -> reqwest::header::HeaderMap {
    use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, USER_AGENT};
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        ),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/json, text/plain, */*"),
    );
    headers.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"),
    );
    headers
}

fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .default_headers(browser_default_headers())
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct CompletionOptions {
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub model_override: Option<String>,
}

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn complete(&self, messages: Vec<Message>, opts: CompletionOptions) -> AppResult<String>;

    async fn embed(
        &self,
        _texts: Vec<String>,
        _model_override: Option<String>,
    ) -> AppResult<Vec<Vec<f32>>> {
        Err(AppError::Custom("当前 provider 不支持 embedding".into()))
    }

    /// 流式补全。每次拿到 delta 文本时调用 `on_chunk`,返回最终累积全文。
    /// 默认实现回退到 `complete` 一次性回吐(供不支持流的 provider 用)。
    async fn stream_complete(
        &self,
        messages: Vec<Message>,
        opts: CompletionOptions,
        on_delta: &tokio::sync::mpsc::UnboundedSender<String>,
        abort: Arc<AtomicBool>,
    ) -> AppResult<String> {
        let full = self.complete(messages, opts).await?;
        if !abort.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = on_delta.send(full.clone());
        }
        Ok(full)
    }

    /// MVP 占位,后续 tool-use 协议用。
    fn supports_tools(&self) -> bool {
        false
    }
}

pub struct OpenAICompatibleProvider {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAICompatibleProvider {
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            client: build_http_client(),
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

#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[async_trait]
impl AIProvider for OpenAICompatibleProvider {
    async fn complete(&self, messages: Vec<Message>, opts: CompletionOptions) -> AppResult<String> {
        let url = format!("{}/v1/chat/completions", self.base_url);
        let body = ChatRequest {
            model: opts.model_override.unwrap_or_else(|| self.model.clone()),
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
            return Err(AppError::Custom(format!("AI 返回 {status}: {text}")));
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

    async fn embed(
        &self,
        texts: Vec<String>,
        model_override: Option<String>,
    ) -> AppResult<Vec<Vec<f32>>> {
        let url = format!("{}/v1/embeddings", self.base_url);
        let body = EmbeddingRequest {
            model: model_override.unwrap_or_else(|| self.model.clone()),
            input: texts,
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
            .map_err(|e| AppError::Custom(format!("Embedding 请求失败: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Custom(format!("Embedding 返回 {status}: {text}")));
        }

        let data: EmbeddingResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Custom(format!("Embedding 响应解析失败: {e}")))?;

        Ok(data.data.into_iter().map(|row| row.embedding).collect())
    }

    async fn stream_complete(
        &self,
        messages: Vec<Message>,
        opts: CompletionOptions,
        on_delta: &tokio::sync::mpsc::UnboundedSender<String>,
        abort: Arc<AtomicBool>,
    ) -> AppResult<String> {
        let url = format!("{}/v1/chat/completions", self.base_url);
        let body = serde_json::json!({
            "model": opts.model_override.unwrap_or_else(|| self.model.clone()),
            "messages": messages,
            "temperature": opts.temperature,
            "max_tokens": opts.max_tokens,
            "stream": true,
        });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Custom(format!("AI 流式请求失败: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Custom(format!("AI 返回 {status}: {text}")));
        }

        stream_sse(resp, streaming::openai_extract, on_delta, abort).await
    }
}

pub struct ClaudeProvider {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl ClaudeProvider {
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            client: build_http_client(),
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
    async fn complete(&self, messages: Vec<Message>, opts: CompletionOptions) -> AppResult<String> {
        let url = format!("{}/v1/messages", self.base_url);
        let body = ClaudeRequest {
            model: opts.model_override.unwrap_or_else(|| self.model.clone()),
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
            return Err(AppError::Custom(format!("AI 返回 {status}: {text}")));
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

    async fn stream_complete(
        &self,
        messages: Vec<Message>,
        opts: CompletionOptions,
        on_delta: &tokio::sync::mpsc::UnboundedSender<String>,
        abort: Arc<AtomicBool>,
    ) -> AppResult<String> {
        let url = format!("{}/v1/messages", self.base_url);
        let body = serde_json::json!({
            "model": opts.model_override.unwrap_or_else(|| self.model.clone()),
            "max_tokens": opts.max_tokens.unwrap_or(1024),
            "messages": messages,
            "stream": true,
        });

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Custom(format!("AI 流式请求失败: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Custom(format!("AI 返回 {status}: {text}")));
        }

        stream_sse(resp, streaming::claude_extract, on_delta, abort).await
    }
}

#[derive(Default)]
pub struct AIService {
    provider: tokio::sync::RwLock<Option<Box<dyn AIProvider>>>,
    embedding_provider: tokio::sync::RwLock<Option<Box<dyn AIProvider>>>,
}

impl AIService {
    pub fn new() -> Self {
        Self {
            provider: tokio::sync::RwLock::new(None),
            embedding_provider: tokio::sync::RwLock::new(None),
        }
    }

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
                if base_url.is_empty() {
                    "https://api.anthropic.com"
                } else {
                    base_url
                },
                api_key,
                if model.is_empty() {
                    "claude-3-5-sonnet-latest"
                } else {
                    model
                },
            )),
            _ => match provider_type {
                "ollama" => Box::new(OpenAICompatibleProvider::new(
                    if base_url.is_empty() {
                        "http://localhost:11434"
                    } else {
                        base_url
                    },
                    "",
                    if model.is_empty() { "llama3" } else { model },
                )),
                _ => Box::new(OpenAICompatibleProvider::new(
                    if base_url.is_empty() {
                        "https://api.openai.com"
                    } else {
                        base_url
                    },
                    api_key,
                    if model.is_empty() {
                        "gpt-4o-mini"
                    } else {
                        model
                    },
                )),
            },
        };
        *self.provider.write().await = Some(p);
    }

    pub async fn configure_embedding(&self, base_url: &str, api_key: &str, model: &str) {
        let provider: Box<dyn AIProvider> = Box::new(OpenAICompatibleProvider::new(
            if base_url.is_empty() {
                "https://api.openai.com"
            } else {
                base_url
            },
            api_key,
            if model.is_empty() {
                "text-embedding-3-small"
            } else {
                model
            },
        ));
        *self.embedding_provider.write().await = Some(provider);
    }

    pub async fn complete(
        &self,
        messages: Vec<Message>,
        opts: CompletionOptions,
    ) -> AppResult<String> {
        let guard = self.provider.read().await;
        let p = match guard.as_ref() {
            Some(p) => p,
            None => return Ok(Self::offline_fallback(&messages)),
        };
        match p.complete(messages.clone(), opts).await {
            Ok(result) => Ok(result),
            Err(_) => Ok(Self::offline_fallback(&messages)),
        }
    }

    pub async fn complete_required(
        &self,
        messages: Vec<Message>,
        opts: CompletionOptions,
    ) -> AppResult<String> {
        let guard = self.provider.read().await;
        let provider = guard.as_ref().ok_or_else(|| {
            AppError::Custom("AI provider 未配置，请先在设置中完成 AI 配置".into())
        })?;
        provider.complete(messages, opts).await
    }

    /// 流式补全(强校验)。`on_chunk` 在每次拿到 delta 文本时被调用,
    /// 返回值是累积的全文(供调用方写库)。`abort` 让外部立刻打断。
    pub async fn stream_required(
        &self,
        messages: Vec<Message>,
        opts: CompletionOptions,
        on_delta: &tokio::sync::mpsc::UnboundedSender<String>,
        abort: Arc<AtomicBool>,
    ) -> AppResult<String> {
        let guard = self.provider.read().await;
        let provider = guard.as_ref().ok_or_else(|| {
            AppError::Custom("AI provider 未配置，请先在设置中完成 AI 配置".into())
        })?;
        provider
            .stream_complete(messages, opts, on_delta, abort)
            .await
    }

    pub async fn embed(
        &self,
        texts: Vec<String>,
        model_override: Option<String>,
    ) -> AppResult<Vec<Vec<f32>>> {
        let guard = self.embedding_provider.read().await;
        let provider = guard
            .as_ref()
            .ok_or_else(|| AppError::Custom("Embedding provider 未配置".into()))?;
        provider.embed(texts, model_override).await
    }

    fn offline_fallback(messages: &[Message]) -> String {
        let last = messages.last().map(|m| m.content.as_str()).unwrap_or("");
        if last.contains("总结") || last.contains("表现") {
            "今天辛苦了,每一步推进都有价值。明天继续保持节奏。".into()
        } else if last.contains("速记") || last.contains("梳理") || last.contains("candidates")
        {
            Self::default_quick_note_for(messages)
        } else if last.contains("拆解") || last.contains("子任务") {
            r#"[{\"name\":\"第一步：分析需求\",\"estimatedMinutes\":30,\"quadrant\":\"important_not_urgent\"},{\"name\":\"第二步：实现核心\",\"estimatedMinutes\":60,\"quadrant\":\"important_not_urgent\"},{\"name\":\"第三步：测试验证\",\"estimatedMinutes\":30,\"quadrant\":\"important_not_urgent\"}]"#.into()
        } else if last.contains("建议") || last.contains("推荐") {
            "建议优先处理紧急重要的任务,上午精力最好时段推进核心工作。".into()
        } else if last.contains("象限") || last.contains("分类") {
            "important_not_urgent".into()
        } else {
            "保持节奏,每天进步一点点。".into()
        }
    }

    pub async fn is_configured(&self) -> bool {
        self.provider.read().await.is_some()
    }

    fn default_quick_note_for(messages: &[Message]) -> String {
        let raw = messages
            .last()
            .map(|m| m.content.as_str())
            .unwrap_or("你的想法");
        let input = Self::extract_quick_note_input(raw);
        serde_json::json!({
            "candidates": [
                {"label": "A", "style": "faithful", "styleName": "忠实整理版", "text": format!("{input}")},
                {"label": "B", "style": "question", "styleName": "研究问题版", "text": format!("这个想法可以先保留为一个待澄清的问题：{input}")},
                {"label": "C", "style": "direction", "styleName": "推进思路版", "text": format!("可以沿着这个方向继续想：{input}。下一步重点是确认其中最值得展开的关系或假设。")}
            ]
        })
        .to_string()
    }

    fn extract_quick_note_input(prompt: &str) -> String {
        let input = prompt
            .split_once("用户输入:\n")
            .and_then(|(_, rest)| rest.split_once("\n\n").map(|(value, _)| value))
            .unwrap_or(prompt)
            .trim();
        if input.is_empty() {
            "你的想法".into()
        } else {
            input.into()
        }
    }
}

pub struct ResponseValidator;

impl ResponseValidator {
    pub fn validate_decompose(raw: &str) -> String {
        let parsed: Result<Vec<serde_json::Value>, _> = serde_json::from_str(raw);
        match parsed {
            Ok(mut arr) => {
                if arr.len() < 3 {
                    return Self::default_decompose();
                }
                if arr.len() > 7 {
                    arr.truncate(7);
                }
                for item in &mut arr {
                    if let Some(min) = item.get("estimatedMinutes").and_then(|v| v.as_i64()) {
                        if min < 5 || min > 480 {
                            item["estimatedMinutes"] = serde_json::json!(30);
                        }
                    }
                    if let Some(q) = item.get("quadrant").and_then(|v| v.as_str()) {
                        let valid = [
                            "important_urgent",
                            "important_not_urgent",
                            "not_important_urgent",
                            "not_important_not_urgent",
                        ];
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

    pub fn validate_narrative(raw: &str) -> String {
        let forbidden = ["失败", "落后", "拖延", "懒惰", "差劲"];
        let mut result = raw.to_string();
        for word in &forbidden {
            result = result.replace(word, "调整");
        }
        if result.chars().count() > 200 {
            result = result.chars().take(200).collect();
        }
        if result.chars().count() < 5 {
            return "今天辛苦了，保持节奏。".into();
        }
        result
    }

    fn default_decompose() -> String {
        r#"[{\"name\":\"分析需求\",\"estimatedMinutes\":30,\"quadrant\":\"important_not_urgent\"},{\"name\":\"实现核心\",\"estimatedMinutes\":60,\"quadrant\":\"important_not_urgent\"},{\"name\":\"验证测试\",\"estimatedMinutes\":30,\"quadrant\":\"important_not_urgent\"}]"#.into()
    }

    pub fn validate_quick_note(raw: &str) -> AppResult<String> {
        let json_str = Self::extract_json_object(raw);
        let val: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| AppError::Custom(format!("AI 返回的速记 JSON 无法解析: {e}")))?;
        let arr = val
            .get("candidates")
            .and_then(|v| v.as_array())
            .ok_or_else(|| AppError::Custom("AI 返回的速记 JSON 缺少 candidates 数组".into()))?;
        if arr.len() < 3 {
            return Err(AppError::Custom("AI 返回的速记候选不足 3 个".into()));
        }

        let mut result = val.clone();
        if let Some(candidates) = result.get_mut("candidates").and_then(|v| v.as_array_mut()) {
            candidates.truncate(3);
            for c in candidates.iter_mut() {
                if c.get("style").and_then(|v| v.as_str()) == Some("task") {
                    if let Some(q) = c.get("quadrant").and_then(|v| v.as_str()) {
                        let valid = [
                            "important_urgent",
                            "important_not_urgent",
                            "not_important_urgent",
                            "not_important_not_urgent",
                        ];
                        if !valid.contains(&q) {
                            c["quadrant"] = serde_json::json!("important_not_urgent");
                        }
                    } else {
                        c["quadrant"] = serde_json::json!("important_not_urgent");
                    }
                }
            }
        }
        serde_json::to_string(&result)
            .map_err(|e| AppError::Custom(format!("AI 返回的速记 JSON 序列化失败: {e}")))
    }

    fn extract_json_object(raw: &str) -> String {
        let s = raw.trim();
        if let Some(start) = s.find('{') {
            if let Some(end) = s.rfind('}') {
                if end > start {
                    return s[start..=end].to_string();
                }
            }
        }
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{AIService, CompletionOptions, Message, ResponseValidator};

    #[test]
    fn browser_default_headers_uses_browser_like_user_agent_and_accept() {
        let headers = super::browser_default_headers();

        let ua = headers
            .get(reqwest::header::USER_AGENT)
            .expect("HTTP client must announce a User-Agent so Cloudflare does not block as bot")
            .to_str()
            .expect("User-Agent should be ASCII");
        assert!(
            ua.starts_with("Mozilla/"),
            "User-Agent should look like a browser, got: {ua}"
        );

        let accept = headers
            .get(reqwest::header::ACCEPT)
            .expect("Accept header must be set so the request matches a real browser")
            .to_str()
            .expect("Accept should be ASCII");
        assert!(
            accept.contains("application/json"),
            "Accept header should advertise JSON support, got: {accept}"
        );
    }

    #[test]
    fn validate_quick_note_rejects_unquoted_json_keys() {
        let result = ResponseValidator::validate_quick_note("{candidates:[]}");

        assert!(result.is_err());
    }

    #[test]
    fn offline_quick_note_fallback_uses_user_input_not_prompt() {
        let raw = super::AIService::offline_fallback(&[super::Message {
            role: "user".into(),
            content: super::prompt_templates::quick_note_optimization_prompt("论文实验思路"),
        }]);

        let result =
            ResponseValidator::validate_quick_note(&raw).expect("fallback should be strict JSON");
        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("validated JSON should parse");
        let candidates = parsed["candidates"].as_array().unwrap();

        assert_eq!(candidates.len(), 3);
        for candidate in candidates {
            let text = candidate["text"].as_str().unwrap();
            assert!(text.contains("论文实验思路"));
            assert!(!text.contains("你是一个专注力管理助手"));
            assert!(!text.contains("请返回 JSON"));
        }
    }

    #[tokio::test]
    async fn complete_required_returns_error_without_provider() {
        let ai = AIService::new();
        let result = ai
            .complete_required(
                vec![Message {
                    role: "user".into(),
                    content: "hello".into(),
                }],
                CompletionOptions::default(),
            )
            .await;

        assert!(result.is_err());
    }

    #[test]
    fn validate_quick_note_preserves_full_ai_text() {
        let long_text = "这是一段超过二百字的完整 AI 梳理结果，用来确认后端不会把模型已经生成好的内容截断。它应该完整返回给前端，由界面决定如何展示，而不是在服务层提前裁掉。这里继续补充一些研究背景、问题意识、实验假设、可能的变量控制、预期观察指标，以及后续需要验证的方向。最终用户需要看到完整内容，才能判断这个版本是否值得保存。".repeat(3);
        let raw = serde_json::json!({
            "candidates": [
                {"label": "A", "style": "task", "styleName": "偏任务导向", "text": long_text, "quadrant": "important_not_urgent"},
                {"label": "B", "style": "note", "styleName": "偏笔记梳理", "text": long_text},
                {"label": "C", "style": "checklist", "styleName": "偏简洁行动清单", "text": long_text}
            ]
        })
        .to_string();

        let result =
            ResponseValidator::validate_quick_note(&raw).expect("valid AI JSON should pass");
        let parsed: serde_json::Value =
            serde_json::from_str(&result).expect("validated JSON should parse");

        assert_eq!(parsed["candidates"][0]["text"].as_str().unwrap(), long_text);
    }
}

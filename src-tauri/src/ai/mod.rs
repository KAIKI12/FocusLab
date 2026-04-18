//! AI 服务层 — 多 provider 抽象 + OpenAI-compatible 实现。
//!
//! 架构(对齐 docs/03):
//! - `AIProvider` trait: `complete(messages, opts) -> Result<String>`
//! - `CompatibleProvider`: 走 OpenAI `/v1/chat/completions` 格式,
//!   兼容 OpenAI / DeepSeek / Zhipu / 本地代理
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

// ---------- CompatibleProvider (OpenAI-format) ----------

pub struct CompatibleProvider {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl CompatibleProvider {
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
impl AIProvider for CompatibleProvider {
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
        base_url: &str,
        api_key: &str,
        model: &str,
    ) {
        let p: Box<dyn AIProvider> = match provider_type {
            "ollama" => Box::new(CompatibleProvider::new(
                if base_url.is_empty() { "http://localhost:11434" } else { base_url },
                "",
                if model.is_empty() { "llama3" } else { model },
            )),
            _ => Box::new(CompatibleProvider::new(
                if base_url.is_empty() { "https://api.openai.com" } else { base_url },
                api_key,
                if model.is_empty() { "gpt-4o-mini" } else { model },
            )),
        };
        *self.provider.write().await = Some(p);
    }

    pub async fn complete(
        &self,
        messages: Vec<Message>,
        opts: CompletionOptions,
    ) -> AppResult<String> {
        let guard = self.provider.read().await;
        let p = guard
            .as_ref()
            .ok_or_else(|| AppError::Custom("AI 未配置,请在设置中填入 API Key".into()))?;
        p.complete(messages, opts).await
    }

    pub async fn is_configured(&self) -> bool {
        self.provider.read().await.is_some()
    }
}

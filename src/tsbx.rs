use crate::config::Config;
use anyhow::{anyhow, Context, Result};
use reqwest::{header, Client, Url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, trace};

pub struct TsbxClient {
    http: Client,
    base_url: Url,
}

impl TsbxClient {
    pub fn new(config: &Config) -> Result<Self> {
        let http = Client::builder()
            .user_agent("askrepo-service/0.1")
            .default_headers(Self::default_headers(&config.tsbx_admin_token)?)
            .build()
            .context("failed to build TSBX reqwest client")?;

        let mut base_url = Url::parse(&config.tsbx_host_url).or_else(|_| {
            Url::parse(&format!("http://{}", config.tsbx_host_url))
        })
        .context(
            "TSBX_HOST_URL is not a valid URL (expected absolute URL, e.g., http://localhost:9000)",
        )?;
        if base_url.path().is_empty() {
            base_url.set_path("/");
        }

        Ok(Self { http, base_url })
    }

    fn default_headers(token: &str) -> Result<header::HeaderMap> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", token))
                .context("invalid TSBX admin token for header")?,
        );
        Ok(headers)
    }

    fn sandboxes_url(&self) -> Result<Url> {
        self.base_url
            .join("/api/v0/sandboxes")
            .context("failed to construct sandboxes URL")
    }

    pub async fn sandbox_exists_with_tag(&self, tag: &str) -> Result<bool> {
        let mut url = self.sandboxes_url()?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("tags", tag);
            pairs.append_pair("limit", "1");
        }

        trace!(%url, tag = tag, "checking for existing sandbox by tag");
        let response = self
            .http
            .get(url.clone())
            .send()
            .await
            .with_context(|| format!("failed to query TSBX for tag '{}'", tag))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "TSBX API returned {} while listing sandboxes (body: {})",
                status,
                body
            ));
        }

        let list = response
            .json::<SandboxListResponse>()
            .await
            .context("failed to deserialize sandbox list")?;
        Ok(!list.items.is_empty())
    }

    pub async fn create_sandbox(&self, payload: &NewSandboxPayload) -> Result<SandboxRecord> {
        let url = self.sandboxes_url()?;
        let tags = &payload.tags;
        trace!(%url, tags = ?tags, "creating sandbox via TSBX API");

        let response = self
            .http
            .post(url.clone())
            .json(payload)
            .send()
            .await
            .context("failed to submit create-sandbox request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "TSBX API returned {} when creating sandbox (body: {})",
                status,
                body
            ));
        }

        let sandbox = response
            .json::<SandboxRecord>()
            .await
            .context("failed to deserialize create-sandbox response")?;
        debug!(sandbox = %sandbox.id, "created sandbox via TSBX API");
        Ok(sandbox)
    }
}

#[derive(Debug, Serialize)]
pub struct NewSandboxPayload {
    pub metadata: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startup_task: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idle_timeout_seconds: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inference_model: Option<String>,
}

impl NewSandboxPayload {
    pub fn new(metadata: serde_json::Value) -> Self {
        Self {
            metadata,
            description: None,
            tags: Vec::new(),
            instructions: None,
            setup: None,
            startup_task: None,
            env: HashMap::new(),
            idle_timeout_seconds: None,
            inference_model: None,
        }
    }

    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_instructions(mut self, instructions: String) -> Self {
        self.instructions = Some(instructions);
        self
    }

    pub fn with_idle_timeout(mut self, secs: Option<i32>) -> Self {
        self.idle_timeout_seconds = secs;
        self
    }

    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    pub fn with_startup_task(mut self, task: String) -> Self {
        self.startup_task = Some(task);
        self
    }
}

#[derive(Debug, Deserialize)]
pub struct SandboxRecord {
    pub id: String,
}

#[derive(Debug, Deserialize)]
struct SandboxListResponse {
    pub items: Vec<SandboxRecord>,
}

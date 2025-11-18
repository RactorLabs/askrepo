use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::env;
use std::time::Duration;

pub struct Config {
    pub twitter_bearer_token: String,
    pub twitter_user_id: String,
    pub twitter_api_base: String,
    pub poll_interval: Duration,
    pub tsbx_host_url: String,
    pub tsbx_admin_token: String,
    pub initial_since_id: Option<String>,
    pub twitter_api_key: Option<String>,
    pub twitter_api_secret: Option<String>,
    pub twitter_access_token: Option<String>,
    pub twitter_access_token_secret: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let twitter_bearer_token = required_env("TWITTER_BEARER_TOKEN")?;
        let twitter_user_id = required_env("TWITTER_USER_ID")?;
        let twitter_api_base =
            optional_env("TWITTER_API_BASE").unwrap_or_else(|| "https://api.x.com".to_string());
        let poll_interval_secs: u64 = optional_env("TWITTER_POLL_INTERVAL_SECS")
            .and_then(|val| val.parse().ok())
            .unwrap_or(90);
        let tsbx_host_url = required_env("TSBX_HOST_URL")?;
        let tsbx_admin_token = required_env("TSBX_ADMIN_TOKEN")?;
        let initial_since_id = optional_env("TWITTER_SINCE_ID");
        let twitter_api_key = required_env("TWITTER_API_KEY")?;
        let twitter_api_secret = required_env("TWITTER_API_SECRET")?;
        let twitter_access_token = required_env("TWITTER_ACCESS_TOKEN")?;
        let twitter_access_token_secret = required_env("TWITTER_ACCESS_TOKEN_SECRET")?;

        Ok(Self {
            twitter_bearer_token,
            twitter_user_id,
            twitter_api_base,
            poll_interval: Duration::from_secs(poll_interval_secs.max(10)),
            tsbx_host_url,
            tsbx_admin_token,
            initial_since_id,
            twitter_api_key: Some(twitter_api_key.clone()),
            twitter_api_secret: Some(twitter_api_secret.clone()),
            twitter_access_token: Some(twitter_access_token.clone()),
            twitter_access_token_secret: Some(twitter_access_token_secret.clone()),
        })
    }

    pub fn sandbox_env(&self) -> HashMap<String, String> {
        let mut env_map = HashMap::new();
        env_map.insert(
            "TWITTER_BEARER_TOKEN".to_string(),
            self.twitter_bearer_token.clone(),
        );
        env_map.insert(
            "TWITTER_API_KEY".to_string(),
            self.twitter_api_key
                .clone()
                .expect("api key checked at startup"),
        );
        env_map.insert(
            "TWITTER_API_SECRET".to_string(),
            self.twitter_api_secret
                .clone()
                .expect("api secret checked at startup"),
        );
        env_map.insert(
            "TWITTER_ACCESS_TOKEN".to_string(),
            self.twitter_access_token
                .clone()
                .expect("access token checked at startup"),
        );
        env_map.insert(
            "TWITTER_ACCESS_TOKEN_SECRET".to_string(),
            self.twitter_access_token_secret
                .clone()
                .expect("access token secret checked at startup"),
        );
        env_map
    }
}

fn optional_env(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|raw| clean_env(&raw))
        .filter(|value| !value.is_empty())
}

fn required_env(key: &str) -> Result<String> {
    let raw = env::var(key).context(format!("{key} is required"))?;
    let value = clean_env(&raw);
    if value.is_empty() {
        return Err(anyhow!("{key} is required"));
    }
    Ok(value)
}

fn clean_env(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 {
        let bytes = trimmed.as_bytes();
        let first = bytes[0];
        let last = bytes[trimmed.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return trimmed[1..trimmed.len() - 1].to_string();
        }
    }
    trimmed.to_string()
}

use serde::{Deserialize, Serialize};
use std::env;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)] 
pub struct Config {
    pub env: EnvConfig,
    pub event_queue_id: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EnvConfig {
    pub moesif_application_id: String,
    pub user_id_header: Option<String>,
    pub company_id_header: Option<String>,
    #[serde(default = "default_batch_max_size")]
    pub batch_max_size: usize,
    #[serde(default = "default_batch_max_wait")]
    pub batch_max_wait: usize,
    pub upstream: String,
    #[serde(default = "default_base_uri")]
    pub base_uri: String,
    #[serde(default = "default_debug")]
    pub debug: bool,
    #[serde(default = "connection_timeout")]
    pub connection_timeout: usize,
}

impl EnvConfig {
    pub fn new() -> Self {
        // Check for the UPSTREAM environment variable, otherwise use the default
        let upstream = env::var("UPSTREAM").unwrap_or_else(|_| "outbound|443||api.moesif.net".to_string());
        let base_uri = parse_upstream_url(&upstream).unwrap_or_else(|_| "https://api.moesif.net".to_string());

        EnvConfig {
            upstream,
            base_uri,
            ..Default::default()
        }
    }
}

fn default_batch_max_size() -> usize {
    100
}

fn default_batch_max_wait() -> usize {
    2000
}

fn default_base_uri() -> String {
    "https://api.moesif.net".to_string()
}

fn default_debug() -> bool {
    false
}

fn connection_timeout() -> usize {
    5000
}

fn parse_upstream_url(upstream: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Assuming the format is always `outbound|443||api.moesif.net`
    let parts: Vec<&str> = upstream.split('|').collect();

    if parts.len() == 4 {
        let domain = parts[3];
        let scheme = if parts[1] == "443" { "https" } else { "http" };
        Ok(format!("{}://{}", scheme, domain))
    } else {
        Err("Invalid UPSTREAM format".into())
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct AppConfigResponse {
    pub org_id: String,
    pub app_id: String,
    pub sample_rate: i32,
    pub block_bot_traffic: bool,
    pub user_sample_rate: HashMap<String, i32>,
    pub company_sample_rate: HashMap<String, i32>,
    pub user_rules: HashMap<String, Vec<EntityRuleValues>>,
    pub company_rules: HashMap<String, Vec<EntityRuleValues>>,
    pub ip_addresses_blocked_by_name: HashMap<String, String>,
    pub regex_config: Vec<RegexRule>,
    pub billing_config_jsons: HashMap<String, String>,
    pub e_tag: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EntityRuleValues {
    pub rules: String,
    pub values: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RegexRule {
    pub conditions: Vec<RegexCondition>,
    pub sample_rate: i32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RegexCondition {
    pub path: String,
    pub value: String,
}
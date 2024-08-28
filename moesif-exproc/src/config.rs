use serde::{Deserialize, Serialize};
use std::{env, collections::HashMap};

#[derive(Default, Clone)]
pub struct Config {
    pub env: EnvConfig,
    pub event_queue_id: u32,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct EnvConfig {
    pub moesif_application_id: String,
    pub user_id_header: Option<String>,
    pub company_id_header: Option<String>,
    #[serde(default = "default_batch_max_size")]
    pub batch_max_size: usize,
    #[serde(default = "default_batch_max_wait")]
    pub batch_max_wait: usize,
    #[serde(default = "default_upstream")]
    pub upstream: String,
    #[serde(default = "default_base_uri")]
    pub base_uri: String,
    #[serde(default = "default_debug")]
    pub debug: bool,
    #[serde(default = "connection_timeout")]
    pub connection_timeout: usize,
}

fn default_batch_max_size() -> usize {
    100
}

fn default_batch_max_wait() -> usize {
    2000
}

fn default_upstream() -> String {
    "moesif_api".to_string()
}

fn default_base_uri() -> String {
    "api.moesif.net".to_string()
}

fn default_debug() -> bool {
    false
}

fn connection_timeout() -> usize {
    5000
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

impl AppConfigResponse {
    pub fn new() -> AppConfigResponse {
        AppConfigResponse {
            sample_rate: 100,
            ..Default::default()
        }
    }

    pub fn get_sampling_percentage(&self, user_id: Option<&str>, company_id: Option<&str>) -> i32 {
        if let Some(user_id) = user_id {
            if let Some(user_rate) = self.user_sample_rate.get(user_id) {
                return *user_rate;
            }
        }

        if let Some(company_id) = company_id {
            if let Some(company_rate) = self.company_sample_rate.get(company_id) {
                return *company_rate;
            }
        }

        self.sample_rate
    }
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


impl EnvConfig {
    pub fn new() -> Self {
        let moesif_application_id = env::var("MOESIF_APPLICATION_ID").unwrap_or_else(|_| String::new());
        let user_id_header = env::var("USER_ID_HEADER").ok();
        let company_id_header = env::var("COMPANY_ID_HEADER").ok();
        let batch_max_size = env::var("BATCH_MAX_SIZE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or_else(default_batch_max_size);
        let batch_max_wait = env::var("BATCH_MAX_WAIT")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or_else(default_batch_max_wait);
        let upstream = env::var("UPSTREAM").unwrap_or_else(|_| default_upstream());
        let base_uri = Self::parse_upstream_url(&upstream).unwrap_or_else(|_| default_base_uri());
        let debug = env::var("DEBUG").ok().map_or_else(default_debug, |v| v == "true");
        let connection_timeout = env::var("CONNECTION_TIMEOUT")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or_else(connection_timeout);

        let config = EnvConfig {
            moesif_application_id,
            user_id_header,
            company_id_header,
            batch_max_size,
            batch_max_wait,
            upstream,
            base_uri,
            debug,
            connection_timeout,
        };

        log::info!("Config initialized: {:?}", config); // Add this line to print the entire config
        
        config
    }

    fn parse_upstream_url(upstream: &str) -> Result<String, ()> {
        // Logic to parse the upstream string and extract base_uri
        // Example logic assuming the upstream format: "outbound|443||api.moesif.net"
        let parts: Vec<&str> = upstream.split('|').collect();
        if parts.len() == 4 {
            Ok(format!("https://{}", parts[3]))
        } else {
            Err(())
        }
    }
}

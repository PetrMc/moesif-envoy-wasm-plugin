use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use chrono::Utc;
use crate::config::{AppConfigResponse, Config, EnvConfig};
use crate::http_callback::{get_header, Handler, HttpCallbackManager};
use crate::rules::{template, GovernanceRule, GovernanceRulesResponse};

const EVENT_QUEUE: &str = "moesif_event_queue";

pub struct EventRootContext {
    pub context_id: String,
    pub config: Arc<Config>,
    pub is_start: bool,
    pub event_byte_buffer: Arc<Mutex<Vec<u8>>>,
    pub http_manager: Arc<Mutex<HttpCallbackManager>>, // Wrapped in Arc<Mutex<>>
    pub moesif_application_id: String,
    pub user_id_header: String,
    pub company_id_header: Option<String>,
    pub upstream: String,
    pub debug: bool,  // Debug field
}

impl EventRootContext {
    pub fn new(
        moesif_application_id: String,
        user_id_header: String,
        company_id_header: Option<String>,
        upstream: String,
        debug: bool,
    ) -> Self {
        let env_config = EnvConfig {
            moesif_application_id: moesif_application_id.clone(), // Pass to both struct and field
            user_id_header: Some(user_id_header.clone()),
            company_id_header: company_id_header.clone(),
            batch_max_size: 100,
            batch_max_wait: 2000,
            upstream: upstream.clone(),
            base_uri: "api.moesif.net".to_string(),
            debug,
            connection_timeout: 5000,
        };

        EventRootContext {
            context_id: uuid::Uuid::new_v4().to_string(),
            config: Arc::new(Config {
                env: env_config,
                event_queue_id: 0, // Adjusted for Exproc
            }),
            is_start: true,
            event_byte_buffer: Arc::new(Mutex::new(Vec::new())),
            http_manager: Arc::new(Mutex::new(HttpCallbackManager::default())), // Wrapped in Arc<Mutex<>>
            moesif_application_id,
            user_id_header,
            company_id_header,
            upstream,
            debug,  // Initialize debug field
        }
    }

    pub fn tick(&mut self) {
        log::trace!(
            "tick context_id {} at {}",
            self.context_id,
            Utc::now().to_rfc3339()
        );

        if self.is_start {
            log::debug!("tick: first tick after initialization");
            self.is_start = false;
            // Adjust the tick period here if needed
        }
        self.poll_queue();
        self.drain_and_send(1);
    }

    fn poll_queue(&self) {
        let mut more = true;
        while more {
            match self.dequeue_shared_queue(self.config.event_queue_id) {
                Ok(Some(event_bytes)) => {
                    self.add_event(event_bytes);
                }
                Ok(None) => {
                    more = false;
                }
                Err(e) => {
                    more = false;
                    log::error!("Failed to dequeue event: {:?}", e);
                }
            }
        }
    }

    fn add_event(&self, event_bytes: Vec<u8>) {
        let mut buffer: MutexGuard<Vec<u8>> = self.event_byte_buffer.lock().unwrap();
        buffer.extend(event_bytes);
    }

    fn drain_and_send(&self, drain_at_least: usize) {
        let mut buffer: MutexGuard<Vec<u8>> = self.event_byte_buffer.lock().unwrap();
        while buffer.len() >= drain_at_least {
            let end = std::cmp::min(buffer.len(), self.config.env.batch_max_size);
            let body = self.write_events_json(vec![buffer.drain(..end).collect::<Vec<u8>>()]);
            self.dispatch_http_request(
                "POST",
                "/v1/events/batch",
                body,
                Box::new(|headers, _| {
                    let config_etag = get_header(&headers, "X-Moesif-Config-Etag");
                    let rules_etag = get_header(&headers, "X-Moesif-Rules-Etag");
                    log::info!(
                        "Event Response eTags: config={:?} rules={:?}",
                        config_etag, rules_etag
                    );
                }),
            );
        }
    }

    fn write_events_json(&self, events: Vec<Vec<u8>>) -> Vec<u8> {
        let total_size: usize = events.iter().map(|event_bytes| event_bytes.len()).sum();
        let json_array_size = total_size + events.len() - 1 + 2;
        let mut event_json_array = Vec::with_capacity(json_array_size);

        event_json_array.push(b'[');
        for (i, event_bytes) in events.iter().enumerate() {
            if i > 0 {
                event_json_array.push(b',');
            }
            event_json_array.extend(event_bytes);
        }
        event_json_array.push(b']');

        event_json_array
    }

    fn request_config_api(&self) {
        self.dispatch_http_request(
            "GET",
            "/v1/config",
            Vec::new(),
            Box::new(|headers, body| {
                let status = get_header(&headers, ":status").unwrap_or_default();
                log::info!("Config Response status {:?}", status);
                if let Some(body) = body {
                    match serde_json::from_slice::<AppConfigResponse>(&body) {
                        Ok(mut app_config_response) => {
                            log::info!("Config Response app_config_response: {:?}", app_config_response);
                            app_config_response.e_tag = get_header(&headers, "X-Moesif-Config-Etag");
                        }
                        Err(e) => {
                            log::error!("No valid AppConfigResponse: {:?}", e);
                        }
                    }
                } else {
                    log::warn!("Config Response body: None");
                }
            }),
        );
    }

    fn request_rules_api(&self) {
        self.dispatch_http_request(
            "GET",
            "/v1/rules",
            Vec::new(),
            Box::new(|headers, body| {
                let e_tag = get_header(&headers, "X-Moesif-Config-Etag");
                let status = get_header(&headers, ":status");
                log::info!("Rules Response status {:?} e_tag {:?}", status, e_tag);
                if let Some(body) = body {
                    let rules = serde_json::from_slice::<Vec<GovernanceRule>>(&body).unwrap_or_default();
                    let rules_response = GovernanceRulesResponse { rules, e_tag };
                    log::info!("Rules Response rules_response: {:?}", rules_response);
                    for rule in rules_response.rules {
                        if let (Some(body), Some(variables)) = (rule.response.body, rule.variables) {
                            let variables: HashMap<String, String> = variables
                                .into_iter()
                                .map(|variable| (variable.name, variable.path))
                                .collect();
                            let templated_body = template(&body.0, &variables);
                            log::info!("Rule templated_body: {:?}", templated_body);
                        }
                    }
                } else {
                    log::warn!("Rules Response body: None");
                }
            }),
        );
    }

    fn dispatch_http_request(
        &self,
        method: &str,
        path: &str,
        body: Vec<u8>,
        callback: Handler,
    ) -> u32 {
        let content_length = body.len().to_string();
        let application_id = self.config.env.moesif_application_id.clone();
        let headers = vec![
            (":method".to_string(), method.to_string()),
            (":path".to_string(), path.to_string()),
            (":authority".to_string(), self.config.env.base_uri.clone()),
            ("accept".to_string(), "*/*".to_string()),
            ("content-type".to_string(), "application/json".to_string()),
            ("content-length".to_string(), content_length),
            ("x-moesif-application-id".to_string(), application_id),
        ];
        let trailers = vec![];
        let timeout = Duration::from_millis(self.config.env.connection_timeout as u64);
        let bodystr = std::str::from_utf8(&body).unwrap_or_default();
        log::info!(
            "Dispatching {} request to {} with body {}",
            method, path, bodystr
        );

        // Simulate an HTTP call and execute the callback
        let token_id = self.simulate_http_call(headers, body, trailers, timeout);
        self.http_manager.lock().unwrap().register_handler(token_id, callback); // Access via Mutex
        token_id
    }

    // Simulates dispatching an HTTP call (this should be replaced with actual HTTP client logic)
    fn simulate_http_call(
        &self,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
        _trailers: Vec<(String, String)>,
        _timeout: Duration,
    ) -> u32 {
        log::info!("Simulated HTTP call with headers: {:?}, body: {:?}", headers, body);
        // Simulate a token ID (in real usage, this would be handled by the HTTP client)
        1
    }

    // Simulate dequeue_shared_queue (this needs to be replaced with actual queue logic)
    fn dequeue_shared_queue(&self, _queue_id: u32) -> Result<Option<Vec<u8>>, ()> {
        // Simulate fetching from a queue
        Ok(Some(vec![])) // Example, returning empty vector as event bytes
    }
}

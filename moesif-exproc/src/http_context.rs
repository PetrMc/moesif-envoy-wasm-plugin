use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;

use base64::Engine as _;
use chrono::Utc;
use crate::config::Config;
use crate::event::{Event, ResponseInfo};

#[derive(Default)]
pub struct EventHttpContext {
    pub config: Arc<Config>,
    pub event: Event,
    pub request_body: Vec<u8>,
    pub response_body: Vec<u8>,
}

impl EventHttpContext {
    pub fn new(config: Arc<Config>) -> Self {
        EventHttpContext {
            config,
            event: Event::default(),
            request_body: Vec::new(),
            response_body: Vec::new(),
        }
    }

    pub fn on_http_request_headers(&mut self, headers: Vec<(String, String)>) {
        self.event.direction = "Incoming".to_string();
        self.event.request.time = Utc::now().to_rfc3339();
        self.event.request.headers = EventHttpContext::header_list_to_map(headers);
        self.event.request.uri = self.event.request.headers.get(":path").unwrap_or(&"".into()).clone();
        self.event.request.verb = self.event.request.headers.get(":method").unwrap_or(&"GET".into()).clone();

        self.event.request.headers.retain(|k, _| !k.starts_with(":"));
        self.event.request.ip_address = EventHttpContext::get_client_ip(&self.event.request.headers);
        self.event.request.api_version = self.event.request.headers.get("x-api-version").cloned();
        self.event.request.transfer_encoding = self.event.request.headers.get("transfer-encoding").cloned();

        if let Some(user_id_header) = &self.config.env.user_id_header {
            self.event.user_id = self.event.request.headers.get(user_id_header).cloned();
        }
        if let Some(company_id_header) = &self.config.env.company_id_header {
            self.event.company_id = self.event.request.headers.get(company_id_header).cloned();
        }
    }

    pub fn on_http_request_body(&mut self, body: Vec<u8>, end_of_stream: bool) {
        self.request_body.extend(body);

        if end_of_stream {
            let body = std::mem::take(&mut self.request_body);
            let content_type = self.event.request.headers.get("content-type").cloned();
            self.event.request.body = EventHttpContext::body_bytes_to_value(body, content_type.as_ref());
        }
    }

    pub fn on_http_response_headers(&mut self, headers: Vec<(String, String)>) {
        let status_str = self.event.request.headers.get(":status").unwrap_or(&"0".to_string()).clone();
        let mut response = ResponseInfo {
            time: Utc::now().to_rfc3339(),
            status: status_str.parse::<usize>().unwrap_or(0),
            headers: EventHttpContext::header_list_to_map(headers),
            ip_address: self.event.request.headers.get("x-forwarded-for").cloned(),
            body: serde_json::Value::Null,
        };
        response.headers.retain(|k, _| !k.starts_with(":"));
        self.event.response = Some(response);
    }

    pub fn on_http_response_body(&mut self, body: Vec<u8>, end_of_stream: bool) {
        self.response_body.extend(body);

        if end_of_stream {
            if let Some(response) = self.event.response.as_mut() {
                let body = std::mem::take(&mut self.response_body);
                let content_type = response.headers.get("content-type").cloned();
                response.body = EventHttpContext::body_bytes_to_value(body, content_type.as_ref());
            }
        }
    }

    pub fn on_log(&self) {
        let json = serde_json::to_string(&self.event).unwrap();
        log::info!("Request & Response Data: {}", json);
        self.enqueue_event();
    }

    fn enqueue_event(&self) {
        let event_bytes = serde_json::to_vec(&self.event).unwrap();
        // Simulate enqueueing event to a shared queue or logging it
        log::info!("Enqueued event to shared queue: {:?}", event_bytes);
    }

    fn header_list_to_map(headers: Vec<(String, String)>) -> HashMap<String, String> {
        headers.into_iter().map(|(k, v)| (k.to_lowercase(), v)).collect::<HashMap<_, _>>()
    }

    fn get_client_ip(headers: &HashMap<String, String>) -> Option<String> {
        let possible_headers = vec![
            "x-client-ip", "x-forwarded-for", "cf-connecting-ip", "fastly-client-ip",
            "true-client-ip", "x-real-ip", "x-cluster-client-ip", "x-forwarded",
            "forwarded-for", "forwarded", "x-appengine-user-ip", "cf-pseudo-ipv4",
        ];

        for header in possible_headers {
            if let Some(value) = headers.get(header) {
                let ips: Vec<&str> = value.split(',').collect();
                for ip in ips {
                    if IpAddr::from_str(ip.trim()).is_ok() {
                        return Some(ip.trim().to_string());
                    }
                }
            }
        }
        None
    }

    fn body_bytes_to_value(body: Vec<u8>, content_type: Option<&String>) -> serde_json::Value {
        if body.is_empty() {
            return serde_json::Value::Null;
        }

        if let Some(content_type) = content_type {
            if content_type.as_str() == "application/json" {
                return match serde_json::from_slice::<serde_json::Value>(&body) {
                    Ok(json) => json,
                    Err(_) => {
                        let encoded = base64::engine::general_purpose::STANDARD.encode(&body);
                        serde_json::Value::String(encoded)
                    }
                };
            }
        }

        let body_str = String::from_utf8_lossy(&body).into_owned();
        serde_json::Value::String(body_str)
    }
}

use tonic::{Request, Response, Status, Streaming};
use tokio_stream::wrappers::ReceiverStream;
use crate::protobuf_mods::envoy::service::ext_proc::v3::{external_processor_server::ExternalProcessor, ProcessingRequest, ProcessingResponse};
use crate::protobuf_mods::envoy::config::core::v3::HeaderMap;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use bytes::Bytes;
use chrono::Utc;
use futures_util::StreamExt;
use base64::Engine;
use crate::config::{Config, AppConfigResponse};
use crate::event::{Event, ResponseInfo};
use crate::http_callback::{get_header, Handler, HttpCallbackManager};
use crate::rules::{template, GovernanceRule, GovernanceRulesResponse};

const EVENT_QUEUE: &str = "moesif_event_queue";

#[derive(Default)]
pub struct MoesifGlooExtProcGrpcService {
    event_context: Arc<Mutex<EventRootContext>>,
}

#[tonic::async_trait]
impl ExternalProcessor for MoesifGlooExtProcGrpcService {
    type ProcessStream = ReceiverStream<Result<ProcessingResponse, Status>>;

    async fn process(
        &self,
        mut request: Request<Streaming<ProcessingRequest>>,
    ) -> Result<Response<Self::ProcessStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn({
            let event_context = Arc::clone(&self.event_context);
            async move {
                while let Some(message) = request.get_mut().next().await {
                    match message {
                        Ok(mut msg) => {
                            let mut event = Event::default();

                            // Handle request headers
                            if let Some(envoy_service_ext_proc_v3::processing_request::Request::RequestHeaders(headers_msg)) = &msg.request {
                                let headers = headers_msg.headers.as_ref();

                                event.direction = "Incoming".to_string();
                                event.request.time = Utc::now().to_rfc3339();
                                event.request.headers = header_list_to_map(headers.clone().cloned());
                                event.request.uri = event.request.headers.get(":path").unwrap_or(&"".into()).clone();
                                event.request.verb = event.request.headers.get(":method").unwrap_or(&"GET".into()).clone();
                                event.request.headers.retain(|k, _| !k.starts_with(":"));
                                event.request.ip_address = get_client_ip(&event.request.headers);
                                event.request.api_version = event.request.headers.get("x-api-version").cloned();
                                event.request.transfer_encoding = event.request.headers.get("transfer-encoding").cloned();
                            }

                            // Handle response headers
                            if let Some(envoy_service_ext_proc_v3::processing_request::Request::ResponseHeaders(response_headers_msg)) = &msg.request {
                                let status_str = response_headers_msg
                                    .headers
                                    .as_ref()
                                    .and_then(|header_map| {
                                        header_map.headers.iter().find(|header| header.key == ":status").map(|header| header.value.clone())
                                    })
                                    .unwrap_or_else(|| "0".to_string());

                                let mut response = ResponseInfo {
                                    time: Utc::now().to_rfc3339(),
                                    status: status_str.parse::<usize>().unwrap_or(0),
                                    headers: header_list_to_map(response_headers_msg.headers.clone()),
                                    ip_address: None,
                                    body: serde_json::Value::Null,
                                };
                                response.headers.retain(|k, _| !k.starts_with(":"));
                                event.response = Some(response);
                            }

                            // Log the request and response
                            log_event(&event);

                            // Add the event to the EventRootContext for batching and sending
                            {
                                let mut event_context = event_context.lock().unwrap();
                                event_context.add_event(serialize_event_to_bytes(&event));
                            }

                            // Send the simplified response
                            let response = simplified_response();
                            if let Err(e) = tx.send(Ok(response)).await {
                                println!("Error sending response: {:?}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            println!("Error receiving message: {:?}", e);
                            if tx.send(Err(Status::internal("Error processing request"))).await.is_err() {
                                println!("Error sending internal error response: {:?}", e);
                                break;
                            }
                        }
                    }
                }
                println!("Stream processing complete.");
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

fn header_list_to_map(header_map: Option<HeaderMap>) -> HashMap<String, String> {
    let mut map = HashMap::new();

    if let Some(header_map) = header_map {
        for header in header_map.headers {
            let key = header.key.to_lowercase();
            let value = header.value.clone();
            map.insert(key, value);
        }
    }

    map
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

fn log_event(event: &Event) {
    let json = serde_json::to_string(event).unwrap();
    log::info!("Request & Response Data: {}", json);
}

fn serialize_event_to_bytes(event: &Event) -> Bytes {
    Bytes::from(serde_json::to_vec(event).unwrap())
}

fn simplified_response() -> ProcessingResponse {
    let headers_response = envoy_service_ext_proc_v3::HeadersResponse {
        response: None,
    };

    ProcessingResponse {
        dynamic_metadata: None,
        mode_override: None,
        override_message_timeout: None,
        response: Some(envoy_service_ext_proc_v3::processing_response::Response::RequestHeaders(headers_response)),
    }
}

#[derive(Default)]
pub struct EventRootContext {
    context_id: String,
    config: Arc<Config>,
    is_start: bool,
    event_byte_buffer: Arc<Mutex<Vec<Bytes>>>,
    http_manager: HttpCallbackManager,
}

impl EventRootContext {
    fn add_event(&self, event_bytes: Bytes) {
        let mut buffer: MutexGuard<Vec<Bytes>> = self.event_byte_buffer.lock().unwrap();
        buffer.push(event_bytes);
    }

    fn drain_and_send(&self, drain_at_least: usize) {
        let mut buffer: MutexGuard<Vec<Bytes>> = self.event_byte_buffer.lock().unwrap();
        while buffer.len() >= drain_at_least {
            let end = std::cmp::min(buffer.len(), self.config.env.batch_max_size);
            let body = self.write_events_json(buffer.drain(..end).collect());
            self.dispatch_http_request(
                "POST",
                "/v1/events/batch",
                body,
                Box::new(|headers, _| {
                    let config_etag = get_header(&headers, "X-Moesif-Config-Etag");
                    let rules_etag = get_header(&headers, "X-Moesif-Rules-Etag");
                    log::info!(
                        "Event Response eTags: config={:?} rules={:?}",
                        config_etag,
                        rules_etag
                    );
                }),
            );
        }
    }

    fn write_events_json(&self, events: Vec<Bytes>) -> Bytes {
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

        event_json_array.into()
    }

    fn request_config_api(&self) {
        self.dispatch_http_request(
            "GET",
            "/v1/config",
            Bytes::new(),
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
            Bytes::new(),
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
        body: Bytes,
        callback: Handler,
    ) -> u32 {
        let content_length = body.len().to_string();
        let application_id = self.config.env.moesif_application_id.clone();
        let headers = vec![
            (":method", method.to_string()),
            (":path", path.to_string()),
            (":authority", self.config.env.base_uri.clone()),
            ("accept", "*/*".to_string()),
            ("content-type", "application/json".to_string()),
            ("content-length", content_length),
            ("x-moesif-application-id", application_id),
        ];

        let trailers: Vec<(String, String)> = vec![];
        let timeout = Duration::from_millis(self.config.env.connection_timeout as u64);
        let bodystr = std::str::from_utf8(&body).unwrap_or_default();
        log::info!(
            "Dispatching {} upstream {} request to {} with body {}",
            &self.config.env.upstream,
            method,
            path,
            bodystr
        );

        // Replace dispatch_http_call with actual HTTP request using an appropriate crate like `reqwest`
        // For example, you could implement this method using `reqwest::Client`.
        
        0 // Return a dummy token ID
    }
}

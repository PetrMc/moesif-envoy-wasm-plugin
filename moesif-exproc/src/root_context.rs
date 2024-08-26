use tonic::{Request, Response, Status, Streaming};
use tokio_stream::wrappers::ReceiverStream;
use envoy_ext_proc_proto::envoy::service::ext_proc::v3::{
    external_processor_server::ExternalProcessor, ProcessingRequest, ProcessingResponse,
    processing_request, processing_response, HeadersResponse,
};

use log::LevelFilter;
use crate::config::{Config, EnvConfig};
use envoy_ext_proc_proto::envoy::config::core::v3::HeaderMap;
use reqwest::{Client, Method};
use reqwest::header::{HeaderMap as ReqwestHeaderMap, HeaderName, HeaderValue};
use crate::http_callback::{get_header, HttpCallbackManager};

use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use bytes::Bytes;
use chrono::Utc;
use futures_util::{StreamExt, TryFutureExt};
use base64::Engine;
use crate::event::{Event, ResponseInfo};
use tokio::sync::{Mutex, TryLockError};

#[derive(Default)]
pub struct MoesifGlooExtProcGrpcService {
    config: Arc<Config>, // Store the config in the service
    event_context: Arc<Mutex<EventRootContext>>,
}

impl MoesifGlooExtProcGrpcService {
    pub fn new(config: Config) -> Result<Self, String> {
        // Set the log level based on the config
        if config.env.debug {
            log::set_max_level(LevelFilter::Debug);
        } else {
            log::set_max_level(LevelFilter::Warn);
        }

        // Initialize EventRootContext with the loaded configuration
        let root_context = EventRootContext::new(config.clone());

        // Return the instance wrapped in Ok
        Ok(MoesifGlooExtProcGrpcService {
            config: Arc::new(config),
            event_context: Arc::new(Mutex::new(root_context)),
        })
    }
}

#[tonic::async_trait]
impl ExternalProcessor for MoesifGlooExtProcGrpcService {
    type ProcessStream = ReceiverStream<Result<ProcessingResponse, Status>>;

    async fn process(
        &self,
        mut request: Request<Streaming<ProcessingRequest>>,
    ) -> Result<Response<Self::ProcessStream>, Status> {
        log::info!("Processing new gRPC request...");
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn({
            let event_context = Arc::clone(&self.event_context);
            async move {
                while let Some(message) = request.get_mut().next().await {
                    match message {
                        Ok(msg) => {
                            log::info!("Received message: {:?}", msg);

                            let mut event = Event::default();
                            event.request.time = Utc::now().to_rfc3339();
                            log::info!("Generated request time: {}", event.request.time);

                            // Handle request headers
                            if let Some(processing_request::Request::RequestHeaders(headers_msg)) =
                                &msg.request
                            {
                                log::info!("Processing request headers...");

                                let headers = headers_msg.headers.as_ref();
                                if headers.is_none() {
                                    log::warn!("No headers found in request.");
                                } else {
                                    log::info!("Headers found: {:?}", headers);
                                }

                                event.direction = "Incoming".to_string();

                                // Check each header
                                event.request.headers =
                                    header_list_to_map(headers.clone().cloned());
                                log::info!("Parsed headers: {:?}", event.request.headers);

                                event.request.uri = event
                                    .request
                                    .headers
                                    .get(":path")
                                    .unwrap_or(&"".into())
                                    .clone();
                                log::info!("Parsed URI: {}", event.request.uri);

                                event.request.verb = event
                                    .request
                                    .headers
                                    .get(":method")
                                    .unwrap_or(&"GET".into())
                                    .clone();
                                log::info!("Parsed method: {}", event.request.verb);

                                event.request.headers.retain(|k, _| !k.starts_with(":"));
                                log::info!("Filtered headers: {:?}", event.request.headers);

                                event.request.ip_address =
                                    get_client_ip(&event.request.headers);
                                log::info!("Client IP: {:?}", event.request.ip_address);

                                event.request.api_version = event
                                    .request
                                    .headers
                                    .get("x-api-version")
                                    .cloned();
                                log::info!("API Version: {:?}", event.request.api_version);

                                event.request.transfer_encoding = event
                                    .request
                                    .headers
                                    .get("transfer-encoding")
                                    .cloned();
                                log::info!(
                                    "Transfer Encoding: {:?}",
                                    event.request.transfer_encoding
                                );
                            }

                            // Handle response headers
                            if let Some(
                                processing_request::Request::ResponseHeaders(
                                    response_headers_msg,
                                ),
                            ) = &msg.request
                            {
                                log::info!("Processing response headers...");
                                let status_str = response_headers_msg
                                    .headers
                                    .as_ref()
                                    .and_then(|header_map| {
                                        header_map
                                            .headers
                                            .iter()
                                            .find(|header| header.key == ":status")
                                            .map(|header| header.value.clone())
                                    })
                                    .unwrap_or_else(|| "0".to_string());

                                let mut response = ResponseInfo {
                                    time: Utc::now().to_rfc3339(),
                                    status: status_str.parse::<usize>().unwrap_or(0),
                                    headers: header_list_to_map(
                                        response_headers_msg.headers.clone(),
                                    ),
                                    ip_address: None,
                                    body: serde_json::Value::Null,
                                };
                                response.headers.retain(|k, _| !k.starts_with(":"));
                                event.response = Some(response);
                            }

                            // Log the request and response
                            log_event(&event);

                            // Add the event to the EventRootContext for batching and sending
                            log::info!("Attempting to acquire lock on EventRootContext...");
                            {
                                let mut event_context = event_context.lock().await;

                                log::info!("Lock acquired. Adding event to EventRootContext...");
                                event_context.add_event(serialize_event_to_bytes(&event)).await;
                                log::info!("Event added to EventRootContext.");
                            }

                            // Send the simplified response
                            let response = simplified_response();
                            log::info!("Attempting to send response...");
                            if let Err(e) = tx.send(Ok(response)).await {
                                log::error!("Error sending response: {:?}", e);
                                break;
                            } else {
                                log::info!("Response sent successfully.");
                            }
                        }
                        Err(e) => {
                            log::error!("Error receiving message: {:?}", e);
                            if tx.send(Err(Status::internal("Error processing request"))).await.is_err() {
                                log::error!("Error sending internal error response: {:?}", e);
                                break;
                            }
                        }
                    }
                }
                log::info!("Stream processing complete.");
            }
        });

        log::info!("Returning gRPC response stream.");
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
        "x-client-ip",
        "x-forwarded-for",
        "cf-connecting-ip",
        "fastly-client-ip",
        "true-client-ip",
        "x-real-ip",
        "x-cluster-client-ip",
        "x-forwarded",
        "forwarded-for",
        "forwarded",
        "x-appengine-user-ip",
        "cf-pseudo-ipv4",
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
    let headers_response = HeadersResponse { response: None };

    ProcessingResponse {
        dynamic_metadata: None,
        mode_override: None,
        override_message_timeout: None,
        response: Some(processing_response::Response::RequestHeaders(headers_response)),
    }
}

#[derive(Default)]
pub struct EventRootContext {
    pub config: Config,
    pub event_byte_buffer: Mutex<Vec<Bytes>>,
    context_id: String,
    is_start: bool,
}

impl EventRootContext {
    pub fn new(config: Config) -> Self {
        EventRootContext {
            config, // No need for Arc if single-threaded or managed differently
            event_byte_buffer: Mutex::new(Vec::new()),
            context_id: String::new(),
            is_start: true,
        }
    }

    async fn write_events_json(&self, events: Vec<Bytes>) -> Bytes {
        log::info!("Entering write_events_json with {} events.", events.len());

        let total_size: usize = events.iter().map(|event_bytes| event_bytes.len()).sum();

        let json_array_size = if events.len() > 0 {
            total_size + events.len() - 1 + 2
        } else {
            2 // Just for the empty array '[]'
        };
        let mut event_json_array = Vec::with_capacity(json_array_size);

        event_json_array.push(b'[');
        for (i, event_bytes) in events.iter().enumerate() {
            if i > 0 {
                event_json_array.push(b',');
            }
            event_json_array.extend(event_bytes);

            log::info!("Adding event to JSON array: {:?}", std::str::from_utf8(event_bytes).unwrap_or("Invalid UTF-8"));
        }
        event_json_array.push(b']');

        let final_json = std::str::from_utf8(&event_json_array).unwrap_or("Invalid UTF-8");
        log::info!("Final JSON array being sent: {}", final_json);
        log::info!(
            "Exiting write_events_json with JSON array size {} bytes.",
            event_json_array.len()
        );
        event_json_array.into() // Return as Bytes
    }

    pub async fn add_event(&self, event_bytes: Bytes) {
        log::info!("Entering add_event.");

        {
            let mut buffer = self.event_byte_buffer.lock().await;
            log::info!(
                "Acquired lock on event_byte_buffer. Current buffer size: {}",
                buffer.len()
            );

            buffer.push(event_bytes);
            log::info!("Event added to buffer. New buffer size: {}", buffer.len());
        }

        self.drain_and_send(2).await;
    }

    async fn drain_and_send(&self, drain_at_least: usize) {
        log::info!(
            "Entering drain_and_send with drain_at_least size: {}",
            drain_at_least
        );

        let mut attempts = 0;
        loop {
            match self.event_byte_buffer.try_lock() {
                Ok(mut buffer) => {
                    log::info!(
                        "Acquired lock on event_byte_buffer for draining after {} attempts. Current buffer size: {}",
                        attempts, buffer.len()
                    );

                    while buffer.len() >= drain_at_least {
                        log::info!("Buffer size {} >= {}. Draining and sending events.", buffer.len(), drain_at_least);

                        log::info!("Config batch_max_size: {}", self.config.env.batch_max_size);
                        let end = std::cmp::min(buffer.len(), self.config.env.batch_max_size);
                        log::info!("Calculated end for draining: {}", end);

                        let events_to_send: Vec<Bytes> = buffer.drain(..end).collect();
                        log::info!("Drained {} events from buffer for sending.", events_to_send.len());
                        log::info!("Buffer size after draining: {}", buffer.len());

                        let body = self.write_events_json(events_to_send).await;

                        log::info!("Dispatching HTTP request with {} events.", end);

                        if let Err(e) = self.dispatch_http_request(
                            "POST",
                            "/v1/events/batch",
                            body,
                            Box::new(|headers, _| {
                                let config_etag = get_header(&headers, "X-Moesif-Config-Etag");
                                let rules_etag = get_header(&headers, "X-Moesif-Rules-Etag");
                                log::info!("Event Response eTags: config={:?} rules={:?}", config_etag, rules_etag);
                            }),
                        ).await {
                            log::error!("Failed to dispatch HTTP request: {:?}", e);
                        }

                        log::info!("Events drained and sent. Current buffer size: {}", buffer.len());
                    }

                    log::info!("Exiting drain_and_send. Current buffer size: {}", buffer.len());
                    break;
                }
                Err(_) => {  // Handle the TryLockError generically
                    attempts += 1;
                    log::warn!("Failed to acquire lock on event_byte_buffer; will retry after a short delay (attempt: {}).", attempts);
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }
    }

    async fn dispatch_http_request(
        &self,
        method: &str,
        path: &str,
        body: Bytes,
        callback: Box<dyn Fn(Vec<(String, String)>, Option<Vec<u8>>) + Send>,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        log::info!("Entering dispatch_http_request.");

        let client = Client::new();
        let url = format!("{}{}", self.config.env.base_uri, path);

        let method = Method::from_bytes(method.as_bytes())?;
        log::info!("Using method: {} and URL: {}", method, url);

        let mut headers = ReqwestHeaderMap::new();
        headers.insert(HeaderName::from_static("content-type"), HeaderValue::from_static("application/json"));
        headers.insert(HeaderName::from_static("x-moesif-application-id"), HeaderValue::from_str(&self.config.env.moesif_application_id)?);

        let curl_cmd = generate_curl_command(method.as_str(), &url, &headers, Some(&body));
        log::info!("Equivalent curl command:\n{}", curl_cmd);

        log::info!(
            "Dispatching {} request to {} with headers: {:?} and body: {}",
            method,
            url,
            headers,
            std::str::from_utf8(&body).unwrap_or_default()
        );

        let response = client
            .request(method, &url)
            .headers(headers)
            .body(body)
            .send()
            .await?;

        let status = response.status();
        log::info!("Received response with status: {}", status);

        let headers: Vec<(String, String)> = response.headers().iter().map(|(k, v)| {
            (k.to_string(), v.to_str().unwrap_or_default().to_string())
        }).collect();

        let body = response.bytes().await.ok();

        // Call the provided callback with the headers and response body
        callback(headers, body.map(|b| b.to_vec()));

        log::info!("Exiting dispatch_http_request.");

        Ok(12345) // Replace with actual token or ID logic if needed
    }
}

fn generate_curl_command(
    method: &str,
    url: &str,
    headers: &ReqwestHeaderMap,
    body: Option<&Bytes>,
) -> String {
    let mut curl_cmd = format!("curl -v -X {} '{}'", method, url);

    // Add headers to the curl command
    for (key, value) in headers {
        let header_value = value.to_str().unwrap_or("");
        curl_cmd.push_str(&format!(" -H '{}: {}'", key, header_value));
    }

    // Add body to the curl command
    if let Some(body) = body {
        let body_str = std::str::from_utf8(body).unwrap_or("");
        curl_cmd.push_str(&format!(" --data '{}'", body_str));
    }

    curl_cmd
}

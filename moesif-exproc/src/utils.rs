use envoy_ext_proc_proto::envoy::service::ext_proc::v3::{
    processing_response, CommonResponse, HeaderMutation, HeadersResponse, HttpHeaders,
    ProcessingResponse,
};
use tonic::Status;

use envoy_ext_proc_proto::envoy::config::core::v3::{
    header_value_option::HeaderAppendAction, HeaderMap, HeaderValue as EnvoyHeaderValue,
    HeaderValueOption,
};

use crate::config::Config;
use crate::root_context::EventRootContext;
use reqwest::header::HeaderMap as ReqwestHeaderMap;
use uuid::Uuid;

use crate::event::{Event, ResponseInfo};
use bytes::Bytes;
use chrono::Utc;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

type Headers = Vec<(String, String)>;

// Handle request headers
pub async fn process_request_headers(
    event_context: &Arc<Mutex<EventRootContext>>,
    config: &Arc<Config>,
    event: &mut Event,
    headers_msg: &HttpHeaders,
    response_headers: &mut HashMap<String, String>,
) {
    log::trace!("Processing request headers...");

    let headers = headers_msg.headers.as_ref();
    if headers.is_none() {
        log::warn!("No headers found in request.");
    } else {
        log::info!("Headers found: {:?}", headers);
    }

    event.direction = "Incoming".to_string();

    // Check each header
    event.request.headers = header_list_to_map(headers.cloned());
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

    event.request.ip_address = get_client_ip(&event.request.headers);
    log::info!("Client IP: {:?}", event.request.ip_address);

    event.request.api_version = event.request.headers.get("x-api-version").cloned();
    log::info!("API Version: {:?}", event.request.api_version);

    event.request.transfer_encoding = event.request.headers.get("transfer-encoding").cloned();
    log::info!("Transfer Encoding: {:?}", event.request.transfer_encoding);

    let moesif_gloo_id = generate_moesif_gloo_id(&mut event.request.headers);
    event.moesif_gloo_id = moesif_gloo_id.clone();
    response_headers.insert("X-Moesif-Gloo-ID".to_string(), moesif_gloo_id);

    add_env_headers_to_event(config, event, response_headers).await;

    log_event(event);
    {
        let ctx = event_context.lock().await;
        ctx.store_event(event.moesif_gloo_id.clone(), event.clone())
            .await;
    }
}

// Handle response headers
pub async fn process_response_headers(
    event_context: &Arc<Mutex<EventRootContext>>,
    response_headers_msg: &HttpHeaders,
) {
    log::trace!("Processing response headers...");
    log::info!("Received Response Headers: {:?}", response_headers_msg);
    if let Some(header_map) = &response_headers_msg.headers {
        for header in &header_map.headers {
            log::info!(
                "Response Header Key: {}, Value: {}",
                header.key,
                header.value
            );
        }
    } else {
        log::warn!("No headers found in response.");
    }

    let (status_str, moesif_gloo_id) = extract_status_and_id(response_headers_msg);

    if moesif_gloo_id.is_empty() {
        log::warn!("Moesif Gloo ID is empty in the response. Skipping matching.");
    } else {
        log::info!(
            "Matching response to request with Moesif Gloo ID: {} and status: {}",
            moesif_gloo_id,
            status_str
        );

        let response = ResponseInfo {
            time: Utc::now().to_rfc3339(),
            status: status_str.parse::<usize>().unwrap_or(0),
            headers: header_list_to_map(response_headers_msg.headers.clone()),
            ip_address: None,
            body: serde_json::Value::Null,
        };

        match_and_store_response(event_context, moesif_gloo_id, response).await;

        // Call the check_and_flush_buffer function to determine if the buffer needs to be flushed
        let mut event_context = event_context.lock().await;
        event_context.check_and_flush_buffer().await;
    }
}

pub async fn send_grpc_response(
    tx: tokio::sync::mpsc::Sender<Result<ProcessingResponse, Status>>,
    response: ProcessingResponse,
) {
    log::trace!("Attempting to send response...");
    if let Err(e) = tx.send(Ok(response)).await {
        log::error!("Error sending response: {:?}", e);
    } else {
        log::info!("Response sent successfully.");
    }
}

pub fn generate_moesif_gloo_id(headers: &mut HashMap<String, String>) -> String {
    let moesif_gloo_id = Uuid::new_v4().to_string();

    if let Some(existing_id) = headers.get("X-Moesif-Gloo-ID").cloned() {
        log::warn!(
            "Duplicate Moesif Gloo ID detected: {}. Generating a new ID.",
            existing_id
        );
    }

    headers.insert("X-Moesif-Gloo-ID".to_string(), moesif_gloo_id.clone());
    log::info!("Generated or retrieved Moesif Gloo ID: {}", moesif_gloo_id);

    moesif_gloo_id
}

pub async fn add_env_headers_to_event(
    config: &Arc<Config>,
    event: &mut Event,
    response_headers: &mut HashMap<String, String>,
) {
    // Log the pre-loaded values from EnvConfig
    log::trace!("Config USER_ID_HEADER: {:?}", config.env.user_id_header);
    log::trace!(
        "Config COMPANY_ID_HEADER: {:?}",
        config.env.company_id_header
    );

    // Log the values directly from the environment
    log::trace!("Env USER_ID_HEADER: {:?}", std::env::var("USER_ID_HEADER"));
    log::trace!(
        "Env COMPANY_ID_HEADER: {:?}",
        std::env::var("COMPANY_ID_HEADER")
    );

    if let Some(user_id_header) = &config.env.user_id_header {
        event.user_id = event.request.headers.get(user_id_header).cloned();
        if event.user_id.is_none() {
            log::trace!(
                "User ID header '{}' not found in the request, setting default value.",
                user_id_header
            );
            event.user_id = Some("default_user_id".to_string());
        }
        event.request.headers.insert(
            .clone(),
            event.user_id.clone().unwrap_or_default(),
        );
        response_headers.insert(
            user_id_header.clone(),
            event.user_id.clone().unwrap_or_default(),
        );
    }

    if let Some(company_id_header) = &config.env.company_id_header {
        event.company_id = event.request.headers.get(company_id_header).cloned();
        if event.company_id.is_none() {
            log::trace!(
                "Company ID header '{}' not found in the request, setting default value.",
                company_id_header
            );
            event.company_id = Some("default_company_id".to_string());
        }
        event.request.headers.insert(
            company_id_header.clone(),
            event.company_id.clone().unwrap_or_default(),
        );
        response_headers.insert(
            company_id_header.clone(),
            event.company_id.clone().unwrap_or_default(),
        );
    }
}

pub async fn match_and_store_response(
    event_context: &Arc<Mutex<EventRootContext>>,
    moesif_gloo_id: String,
    response: ResponseInfo,
) {
    let ctx = event_context.lock().await;
    let matched = ctx
        .match_and_store_response(Some(moesif_gloo_id.clone()), response)
        .await;
    if matched {
        log::info!("Response successfully matched with request.");
    } else {
        log::warn!("Failed to match response with any request.");
    }
}

pub fn extract_status_and_id(headers_msg: &HttpHeaders) -> (String, String) {
    if let Some(header_map) = &headers_msg.headers {
        for header in &header_map.headers {
            log::info!("Header Key: {}, Value: {}", header.key, header.value);
        }
    }

    let status_str = headers_msg
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

    let moesif_gloo_id = headers_msg
        .headers
        .as_ref()
        .and_then(|header_map| {
            header_map
                .headers
                .iter()
                .find(|header| header.key == "X-Moesif-Gloo-ID")
                .map(|header| header.value.clone())
        })
        .unwrap_or_default();

    (status_str, moesif_gloo_id)
}

pub fn header_list_to_map(header_map: Option<HeaderMap>) -> HashMap<String, String> {
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

pub fn get_client_ip(headers: &HashMap<String, String>) -> Option<String> {
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

pub fn log_event(event: &Event) {
    let json = serde_json::to_string(event).unwrap();
    log::info!("Request & Response Data: {}", json);
}

pub fn serialize_event_to_bytes(event: &Event) -> Bytes {
    Bytes::from(serde_json::to_vec(event).unwrap())
}

pub fn simplified_response() -> ProcessingResponse {
    let headers_response = HeadersResponse { response: None };

    ProcessingResponse {
        dynamic_metadata: None,
        mode_override: None,
        override_message_timeout: None,
        response: Some(processing_response::Response::RequestHeaders(
            headers_response,
        )),
    }
}

pub fn response_with_headers(headers: HashMap<String, String>) -> ProcessingResponse {
    if headers.is_empty() {
        // If the moesif_gloo_id is empty, return a simplified response
        return simplified_response();
    }

    // Create a list of HeaderValueOption for each header in the HashMap
    let mut header_options = Vec::new();

    for (key, value) in headers {
        let envoy_header = EnvoyHeaderValue {
            key,
            value,
            raw_value: Bytes::new(), // Empty as we're not using raw bytes
        };

        let header_value_option = HeaderValueOption {
            header: Some(envoy_header),
            append: Some(false), // Deprecated but required for current Envoy compatibility.
            append_action: HeaderAppendAction::AddIfAbsent.into(), // Only add if absent
            keep_empty_value: true, // Keep the header if its value is empty
        };

        header_options.push(header_value_option);
    }

    // Create the HeaderMutation with the list of HeaderValueOption
    let header_mutation = HeaderMutation {
        set_headers: header_options,
        remove_headers: vec![], // No headers to remove
    };

    // Wrap the HeaderMutation in a CommonResponse
    let common_response = CommonResponse {
        header_mutation: Some(header_mutation),
        ..Default::default()
    };

    // Construct the HeadersResponse
    let headers_response = HeadersResponse {
        response: Some(common_response),
    };

    // Construct and return the ProcessingResponse
    ProcessingResponse {
        dynamic_metadata: None,
        mode_override: None,
        override_message_timeout: None,
        response: Some(processing_response::Response::RequestHeaders(
            headers_response,
        )),
    }
}

pub fn generate_curl_command(
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

pub fn get_header(headers: &Headers, name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
        .map(|(_, header_value)| header_value.to_owned())
}

// Print the current log level
pub fn display_log_level() {
    match log::max_level() {
        log::LevelFilter::Error => println!("Logging level set to: ERROR"),
        log::LevelFilter::Warn => println!("Logging level set to: WARN"),
        log::LevelFilter::Info => println!("Logging level set to: INFO"),
        log::LevelFilter::Debug => println!("Logging level set to: DEBUG"),
        log::LevelFilter::Trace => println!("Logging level set to: TRACE"),
        log::LevelFilter::Off => println!("Logging is turned OFF"),
    }
}

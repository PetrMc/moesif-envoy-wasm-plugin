use super::envoy_service_ext_proc_v3::{ProcessingRequest, processing_request};
use serde_json::{Value, Map};

/// Convert a ProcessingRequest to a serde_json::Value for sending to the Moesif API.
pub fn processing_request_to_json(request: &ProcessingRequest) -> Value {
    let mut map = Map::new();

    // Process request headers
    if let Some(processing_request::Request::RequestHeaders(ref request_headers)) = request.request {
        let headers_map: Map<String, Value> = request_headers
            .headers
            .as_ref()
            .map(|h| {
                h.headers
                    .iter()
                    .map(|header| (header.key.clone(), Value::String(String::from_utf8_lossy(&header.raw_value).into())))
                    .collect()
            })
            .unwrap_or_default();
        map.insert("request_headers".to_string(), Value::Object(headers_map));
    }

    // Process response headers
    if let Some(processing_request::Request::ResponseHeaders(ref response_headers)) = request.request {
        let headers_map: Map<String, Value> = response_headers
            .headers
            .as_ref()
            .map(|h| {
                h.headers
                    .iter()
                    .map(|header| (header.key.clone(), Value::String(String::from_utf8_lossy(&header.raw_value).into())))
                    .collect()
            })
            .unwrap_or_default();
        map.insert("response_headers".to_string(), Value::Object(headers_map));
    }

    // Remove the incorrect dynamic_metadata field handling since it's not available in ProcessingRequest
    // Add other fields if necessary

    if let Some(request_headers) = &request.request {
        match request_headers {
            processing_request::Request::RequestHeaders(headers) => {
                if let Some(path) = headers.headers.as_ref().and_then(|h| h.headers.iter().find(|&hdr| hdr.key == ":path")) {
                    map.insert("path".to_string(), Value::String(String::from_utf8_lossy(&path.raw_value).into()));
                }
                if let Some(method) = headers.headers.as_ref().and_then(|h| h.headers.iter().find(|&hdr| hdr.key == ":method")) {
                    map.insert("method".to_string(), Value::String(String::from_utf8_lossy(&method.raw_value).into()));
                }
            }
            _ => {}
        }
    }

    Value::Object(map)
}

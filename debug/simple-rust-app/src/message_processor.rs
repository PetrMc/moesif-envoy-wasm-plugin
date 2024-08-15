use super::envoy_service_ext_proc_v3::{ProcessingRequest, processing_request};

pub fn process_message(msg: &mut ProcessingRequest) {
    // Access the `request` field inside `ProcessingRequest`
    if let Some(processing_request) = msg.request.as_mut() {
        // Match on the specific type of request (e.g., RequestHeaders)
        if let processing_request::Request::RequestHeaders(ref mut request_headers) = processing_request {
            // Access the `headers` field within `RequestHeaders`, which is a `HeaderMap`
            if let Some(header_map) = request_headers.headers.as_mut() {
                // Iterate over the `headers` inside the `HeaderMap`
                for header in &mut header_map.headers {
                    // If the `value` field is empty, decode `raw_value`
                    if header.value.is_empty() {
                        if let Ok(decoded_value) = String::from_utf8(header.raw_value.clone()) {
                            header.value = decoded_value;
                        } else {
                            println!("Failed to decode raw value for header: {}", header.key);
                        }
                    }
                }
            }
        }
    }
}

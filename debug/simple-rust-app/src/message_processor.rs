use chrono::Local; // For timestamp
use super::envoy_service_ext_proc_v3::{ProcessingRequest, processing_request};

pub fn process_message(msg: &mut ProcessingRequest) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Process request headers (Client-to-Server)
    if let Some(processing_request::Request::RequestHeaders(ref mut request_headers)) = msg.request {
        println!("[{}] Processing Client-to-Server request headers:", timestamp);
        if let Some(header_map) = request_headers.headers.as_mut() {
            for header in &mut header_map.headers {
                if header.value.is_empty() {
                    if let Ok(decoded_value) = String::from_utf8(header.raw_value.clone()) {
                        header.value = decoded_value;
                    } else {
                        println!("Failed to decode raw value for header: {}", header.key);
                    }
                }
                // Print each header after processing
                println!("Header: {} -> Value: {}", header.key, header.value);
            }
        } else {
            println!("[{}] No headers found in request headers.", timestamp);
        }
    } else {
        println!("[{}] No RequestHeaders found in message.", timestamp);
    }

    // Process response headers (Server-to-Client)
    if let Some(processing_request::Request::ResponseHeaders(ref mut response_headers)) = msg.request {
        println!("[{}] Processing Server-to-Client response headers:", timestamp);
        if let Some(header_map) = response_headers.headers.as_mut() {
            for header in &mut header_map.headers {
                if header.value.is_empty() {
                    if let Ok(decoded_value) = String::from_utf8(header.raw_value.clone()) {
                        header.value = decoded_value;
                    } else {
                        println!("Failed to decode raw value for header: {}", header.key);
                    }
                }
                // Print each header after processing
                println!("Header: {} -> Value: {}", header.key, header.value);
            }
        } else {
            println!("[{}] No headers found in response headers.", timestamp);
        }
    } else {
        println!("[{}] No ResponseHeaders found in message.", timestamp);
    }

    // Process request body (Client-to-Server)
    if let Some(processing_request::Request::RequestBody(ref mut request_body)) = msg.request {
        println!("[{}] Processing Client-to-Server request body: {:?}", timestamp, request_body);
    }

    // Process response body (Server-to-Client)
    if let Some(processing_request::Request::ResponseBody(ref mut response_body)) = msg.request {
        println!("[{}] Processing Server-to-Client response body: {:?}", timestamp, response_body);
    }
}

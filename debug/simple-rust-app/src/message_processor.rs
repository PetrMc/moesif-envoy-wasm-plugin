use super::envoy_service_ext_proc_v3::{ProcessingRequest, processing_request};
use crate::utils::{print_headers_if_debug};

pub fn process_message(msg: &mut ProcessingRequest) {
    // Process request headers (Client-to-Server)
    if let Some(processing_request::Request::RequestHeaders(ref mut request_headers)) = msg.request {
        let mut headers = Vec::new();
        if let Some(header_map) = request_headers.headers.as_mut() {
            for header in &mut header_map.headers {
                headers.push((header.key.clone(), header.raw_value.clone()));
            }
        }
        print_headers_if_debug("Client-to-Server", &headers);
    }

    // Process response headers (Server-to-Client)
    if let Some(processing_request::Request::ResponseHeaders(ref mut response_headers)) = msg.request {
        let mut headers = Vec::new();
        if let Some(header_map) = response_headers.headers.as_mut() {
            for header in &mut header_map.headers {
                headers.push((header.key.clone(), header.raw_value.clone()));
            }
        }
        print_headers_if_debug("Server-to-Client", &headers);
    }
}

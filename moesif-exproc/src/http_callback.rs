use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Define types for HTTP headers and body content
type Headers = Vec<(String, String)>;
type Body = Vec<u8>;

// Handler type for the callback function that is called when an HTTP call response is received
pub type Handler = Box<dyn Fn(Headers, Option<Body>) + Send>;

#[derive(Default)]
pub struct HttpCallbackManager {
    // Mapping from token_id to callback response handler function
    handlers: Arc<Mutex<HashMap<u32, Handler>>>,
}

impl HttpCallbackManager {
    // Register a new handler for a specific token_id
    pub fn register_handler(&self, token_id: u32, handler: Handler) {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.insert(token_id, handler);
    }

    // Handle an HTTP response by executing the corresponding handler
    pub fn handle_response(&self, token_id: u32, headers: Headers, body: Option<Body>) {
        let mut handlers = self.handlers.lock().unwrap();

        if let Some(handler) = handlers.remove(&token_id) {
            handler(headers, body);
        } else {
            // Log an error if the token_id does not exist, which should not happen in normal operation
            log::error!("Received response for non-existent token: {}", token_id);
        }
    }
}

// Utility function to get a specific header from the Headers
pub fn get_header(headers: &Headers, name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
        .map(|(_, header_value)| header_value.to_owned())
}

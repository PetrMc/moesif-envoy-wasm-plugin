use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// Struct to hold information about an HTTP request
#[derive(Default, Serialize, Deserialize)]
pub struct RequestInfo {
    pub time: String,
    pub verb: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub transfer_encoding: Option<String>,
    pub api_version: Option<String>,
    pub ip_address: Option<String>,
    pub body: serde_json::Value,
}

// Struct to hold information about an HTTP response
#[derive(Default, Serialize, Deserialize)]
pub struct ResponseInfo {
    pub time: String,
    pub status: usize,
    pub headers: HashMap<String, String>,
    pub ip_address: Option<String>,
    pub body: serde_json::Value,
}

// Struct to represent an event, which includes request and response information
#[derive(Default, Serialize, Deserialize)]
pub struct Event {
    pub request: RequestInfo,
    pub response: Option<ResponseInfo>,
    pub user_id: Option<String>,
    pub company_id: Option<String>,
    pub metadata: serde_json::Value,
    pub direction: String,
    pub session_token: Option<String>,
    pub blocked_by: Option<String>,
}

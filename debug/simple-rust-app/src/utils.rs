use std::collections::HashMap;
use std::env;
use chrono::Local; // For timestamp

/// Retrieve all environment variables and store them in a HashMap.
pub fn retrieve_env_variables() -> HashMap<String, String> {
    let mut env_vars = HashMap::new();
    for (key, value) in env::vars() {
        env_vars.insert(key, value);
    }
    env_vars
}

/// Print debug information if DEBUG environment variable is set to true.
pub fn print_if_debug(label: &str, message: Option<&impl std::fmt::Debug>, env_vars: Option<&HashMap<String, String>>) {
    let debug = env::var("DEBUG").unwrap_or_else(|_| "false".to_string()).to_lowercase() == "true";
    if debug {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        println!("[{}] DEBUG: true", timestamp);
        if let Some(msg) = message {
            println!("{} message: {:?}", label, msg);
        }
        if let Some(vars) = env_vars {
            println!("{} environment variables:", label);
            for (key, value) in vars {
                println!("{}: {}", key, value);
            }
        }
    }
}

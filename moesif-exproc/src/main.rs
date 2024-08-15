mod config;
mod event;
mod http_context;
mod root_context;
mod http_callback;
mod rules;
mod update_manager;

use root_context::EventRootContext; // Ensure this matches your module structure
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use log::LevelFilter;
use std::convert::Infallible; // Import Infallible here
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::runtime;

async fn handle_request(
    req: Request<Body>,
    root_context: Arc<Mutex<EventRootContext>>,
) -> Result<Response<Body>, Infallible> {
    log::info!("Received HTTP request: {:?}", req);

    // Lock the root_context to safely access it
    let mut context = root_context.lock().unwrap();

    // Call the tick method or any other processing method
    context.tick();

    // Respond back to the caller
    Ok(Response::new(Body::from("Processed request using existing logic")))
}

fn main() {
    // Initialize logging based on the DEBUG environment variable
    let debug = env::var("DEBUG").unwrap_or_else(|_| "false".to_string()) == "true";
    if debug {
        log::set_max_level(LevelFilter::Debug);
    } else {
        log::set_max_level(LevelFilter::Info);
    }

    // Fetch configuration from environment variables
    let moesif_application_id = env::var("MOESIF_APPLICATION_ID").expect("MOESIF_APPLICATION_ID must be set");
    let user_id_header = env::var("USER_ID_HEADER").unwrap_or_else(|_| "X-User-Example-Header".to_string());
    let company_id_header = env::var("COMPANY_ID_HEADER").ok();
    let upstream = env::var("UPSTREAM").unwrap_or_else(|_| "outbound|443||api.moesif.net".to_string());

    // Create the root context with the configuration
    let root_context = Arc::new(Mutex::new(EventRootContext::new(
        moesif_application_id,
        user_id_header,
        company_id_header,
        upstream,
        debug,
    )));

    // Define the address to listen on
    let addr = SocketAddr::from(([0, 0, 0, 0], 18082));

    // Create the Tokio runtime
    let rt = runtime::Runtime::new().expect("Failed to create Tokio runtime");

    // Run the HTTP server
    rt.block_on(async move {
        let make_svc = make_service_fn(|_conn| {
            let root_context = Arc::clone(&root_context); // Clone the Arc, not the content
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    handle_request(req, Arc::clone(&root_context))
                }))
            }
        });

        let server = Server::bind(&addr).serve(make_svc);

        println!("Listening on http://{}", addr);

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    });
}

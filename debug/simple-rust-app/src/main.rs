mod grpc_service;
mod utils;

use utils::{retrieve_env_variables, print_if_debug};

use tonic::transport::Server;

pub mod envoy_service_ext_proc_v3 {
    include!("proto/envoy.service.ext_proc.v3.rs");
}

pub mod envoy_config_core_v3 {
    include!("proto/envoy.config.core.v3.rs");
}

pub mod envoy_extensions_filters_http_ext_proc_v3 {
    include!("proto/envoy.extensions.filters.http.ext_proc.v3.rs");
}

pub mod envoy_type_v3 {
    include!("proto/envoy.r#type.v3.rs");
}

pub mod xds_core_v3 {
    include!("proto/xds.core.v3.rs");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the tokio runtime to run your async code
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    // Retrieve all environment variables and store them in a map
    let env_vars = retrieve_env_variables();

    // Print environment variables if debug is true
    print_if_debug("Startup", None::<&()>, Some(&env_vars));

    let addr = "0.0.0.0:50051".parse()?;  // Bind to all interfaces, IPv4 and IPv6
    let grpc_service = grpc_service::MoesifGlooExtProcGrpcService::default();

    println!("Starting Moesif ExtProc gRPC server for Solo.io Gloo Gateway on {}", addr);

    Server::builder()
        .add_service(grpc_service::envoy_service_ext_proc_v3::external_processor_server::ExternalProcessorServer::new(grpc_service))
        .serve(addr)
        .await?;

    Ok(())
}

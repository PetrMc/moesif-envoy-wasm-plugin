mod config;
mod event;
mod root_context;
mod http_callback;
mod rules;
mod update_manager;

use tonic::transport::Server;
use envoy_ext_proc_proto::envoy::service::ext_proc::v3::external_processor_server::ExternalProcessorServer as ProcessorServer;
use crate::root_context::MoesifGlooExtProcGrpcService;

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {

    let addr = "0.0.0.0:50051".parse()?;
    let grpc_service = MoesifGlooExtProcGrpcService::default();

    println!("Starting Moesif ExtProc gRPC server for Solo.io Gloo Gateway on {}", addr);

    Server::builder()
        .add_service(ProcessorServer::new(grpc_service))
        .serve(addr)
        .await?;

    Ok(())
}

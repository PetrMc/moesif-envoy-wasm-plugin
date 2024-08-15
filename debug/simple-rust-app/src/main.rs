use tonic::{transport::Server, Request, Response, Status, Streaming};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

mod envoy_service_ext_proc_v3 {
    include!("proto/envoy.service.ext_proc.v3.rs");
}

mod envoy_config_core_v3 {
    include!("proto/envoy.config.core.v3.rs");
}

mod envoy_extensions_filters_http_ext_proc_v3 {
    include!("proto/envoy.extensions.filters.http.ext_proc.v3.rs");
}

mod envoy_type_v3 {
    include!("proto/envoy.r#type.v3.rs");
}

mod xds_core_v3 {
    include!("proto/xds.core.v3.rs");
}

// gRPC service definition
#[derive(Debug, Default)]
pub struct MyGrpcService;

// Implement the gRPC service
#[tonic::async_trait]
impl envoy_service_ext_proc_v3::external_processor_server::ExternalProcessor for MyGrpcService {
    type ProcessStream = ReceiverStream<Result<envoy_service_ext_proc_v3::ProcessingResponse, Status>>;

    async fn process(
        &self,
        mut request: Request<Streaming<envoy_service_ext_proc_v3::ProcessingRequest>>,
    ) -> Result<Response<Self::ProcessStream>, Status> {
        println!("Received gRPC request: {:?}", request);

        // Stream the incoming messages
        while let Some(message) = request.get_mut().next().await {
            match message {
                Ok(msg) => {
                    // Log the content of the received message
                    println!("Received message: {:?}", msg);
                }
                Err(e) => {
                    println!("Error receiving message: {:?}", e);
                    return Err(Status::internal("Error processing request"));
                }
            }
        }

        let (tx, rx) = tokio::sync::mpsc::channel(4);
        tx.send(Ok(envoy_service_ext_proc_v3::ProcessingResponse::default())).await.unwrap();

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the tokio runtime to run your async code
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;  // Bind to all interfaces, IPv4 and IPv6
    let grpc_service = MyGrpcService::default();

    println!("Starting gRPC server on {}", addr);

    Server::builder()
        .add_service(envoy_service_ext_proc_v3::external_processor_server::ExternalProcessorServer::new(grpc_service))
        .serve(addr)
        .await?;

    Ok(())
}

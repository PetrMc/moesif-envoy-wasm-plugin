#[path = "message_processor.rs"]
mod message_processor;

use crate::utils::print_if_debug;
use tonic::{Request, Response, Status, Streaming};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

pub use super::envoy_service_ext_proc_v3;

#[derive(Debug, Default)]
pub struct MoesifGlooExtProcGrpcService;

#[tonic::async_trait]
impl envoy_service_ext_proc_v3::external_processor_server::ExternalProcessor for MoesifGlooExtProcGrpcService {
    type ProcessStream = ReceiverStream<Result<envoy_service_ext_proc_v3::ProcessingResponse, Status>>;

    async fn process(
        &self,
        mut request: Request<Streaming<envoy_service_ext_proc_v3::ProcessingRequest>>,
    ) -> Result<Response<Self::ProcessStream>, Status> {

        println!("Received gRPC request: {:?}", request);

        // Stream the incoming messages
        while let Some(message) = request.get_mut().next().await {
            match message {
                Ok(mut msg) => {
                    // Print received message before processing
                    print_if_debug("Received", Some(&msg), None);

                    // Process the message (convert raw_value to value)
                    message_processor::process_message(&mut msg);

                    // Print processed message
                    print_if_debug("Processed", Some(&msg), None);
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

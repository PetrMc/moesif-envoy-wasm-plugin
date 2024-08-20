#[path = "message_processor.rs"]
mod message_processor;

use message_processor::process_message;
use tonic::{Request, Response, Status, Streaming};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

pub use super::envoy_service_ext_proc_v3;

#[derive(Debug, Default)]
pub struct MoesifGlooExtProcGrpcService;

// Simplified "Continue" response to match the service on port 18080
fn simplified_response() -> envoy_service_ext_proc_v3::ProcessingResponse {
    let headers_response = envoy_service_ext_proc_v3::HeadersResponse {
        response: None, // No additional response fields
    };

    envoy_service_ext_proc_v3::ProcessingResponse {
        dynamic_metadata: None, // No dynamic metadata
        mode_override: None,    // No mode overrides
        override_message_timeout: None,
        response: Some(envoy_service_ext_proc_v3::processing_response::Response::RequestHeaders(headers_response)),
    }
}

#[tonic::async_trait]
impl envoy_service_ext_proc_v3::external_processor_server::ExternalProcessor for MoesifGlooExtProcGrpcService {
    type ProcessStream = ReceiverStream<Result<envoy_service_ext_proc_v3::ProcessingResponse, Status>>;

    async fn process(
        &self,
        mut request: Request<Streaming<envoy_service_ext_proc_v3::ProcessingRequest>>,
    ) -> Result<Response<Self::ProcessStream>, Status> {

        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            // Stream the incoming messages
            while let Some(message) = request.get_mut().next().await {
                match message {
                    Ok(mut msg) => {
                        // Use process_message to handle the message and log it
                        process_message(&mut msg);

                        // Send the simplified response
                        let response = simplified_response();
                        println!("Sending simplified gRPC response: {:?}", response);

                        if let Err(e) = tx.send(Ok(response)).await {
                            println!("Error sending response: {:?}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        println!("Error receiving message: {:?}", e);
                        if tx.send(Err(Status::internal("Error processing request"))).await.is_err() {
                            println!("Error sending internal error response: {:?}", e);
                            break;
                        }
                    }
                }
            }
            println!("Stream processing complete.");
        });

        // Return the simplified response stream
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

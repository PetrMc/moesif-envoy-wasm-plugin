#[path = "message_processor.rs"]
mod message_processor;

use crate::utils::print_if_debug;
use tonic::{Request, Response, Status, Streaming};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

pub use super::envoy_service_ext_proc_v3;
pub use crate::envoy_extensions_filters_http_ext_proc_v3::{ProcessingMode, HeaderSendMode, BodySendMode};

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
                        // Log incoming headers for debugging
                        if let Some(envoy_service_ext_proc_v3::processing_request::Request::RequestHeaders(ref headers)) = msg.request {
                            print_if_debug("Client-to-Server Request Headers", Some(&headers.headers), None);
                        }

                        if let Some(envoy_service_ext_proc_v3::processing_request::Request::ResponseHeaders(ref headers)) = msg.request {
                            print_if_debug("Server-to-Client Response Headers", Some(&headers.headers), None);
                        }

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

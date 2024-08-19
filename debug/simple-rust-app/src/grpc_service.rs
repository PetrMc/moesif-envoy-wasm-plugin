#[path = "message_processor.rs"]
mod message_processor;

use crate::utils::print_if_debug;
use tonic::{Request, Response, Status, Streaming};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tokio::time::{timeout, Duration};

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

        let (tx, rx) = tokio::sync::mpsc::channel(4);
        let timeout_duration = Duration::from_secs(15); // Set your desired timeout here

        // Stream the incoming messages with a timeout
        while let Ok(Some(message)) = timeout(timeout_duration, request.get_mut().next()).await {
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
                        return Err(Status::internal(format!("Error sending response: {:?}", e)));
                    }
                }
                Err(e) => {
                    println!("Error receiving message: {:?}", e);
                    return Err(Status::internal("Error processing request"));
                }
            }
        }

        // Return the simplified response stream
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

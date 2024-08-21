cessinguse crate::message_processor::process_message;
use crate::converter::processing_request_to_json;
use crate::root_context::EventRootContext; // Import EventRootContext
use tonic::{Request, Response, Status, Streaming};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use std::sync::Arc;
use tokio::sync::Mutex;

pub use super::envoy_service_ext_proc_v3;

pub struct MoesifGlooExtProcGrpcService {
    event_context: Arc<Mutex<EventRootContext>>, // Add EventRootContext to the service
}

impl MoesifGlooExtProcGrpcService {
    pub fn new(event_context: Arc<Mutex<EventRootContext>>) -> Self {
        MoesifGlooExtProcGrpcService { event_context }
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

        tokio::spawn({
            let event_context = Arc::clone(&self.event_context);
            async move {
                // Stream the incoming messages
                while let Some(message) = request.get_mut().next().await {
                    match message {
                        Ok(mut msg) => {
                            // Process the message using the configuration
                            process_message(&mut msg);

                            // Convert the message to JSON for sending to Moesif
                            let data = processing_request_to_json(&msg);

                            // Add the event to the EventRootContext for batching and sending
                            let event_context = event_context.lock().await;
                            event_context.add_event(data); // Remove .await

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
            }
        });

        // Return the simplified response stream
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

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

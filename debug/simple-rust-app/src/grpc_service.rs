#[path = "message_processor.rs"]
mod message_processor;

use crate::utils::print_if_debug;
use tonic::{Request, Response, Status, Streaming};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use std::collections::BTreeMap;
use prost_types::{Struct, Value};

pub use super::envoy_service_ext_proc_v3;
pub use crate::envoy_extensions_filters_http_ext_proc_v3::{ProcessingMode, HeaderSendMode, BodySendMode};

#[derive(Debug, Default)]
pub struct MoesifGlooExtProcGrpcService;

// Static "Continue" response
fn static_response() -> envoy_service_ext_proc_v3::ProcessingResponse {
    let common_response = envoy_service_ext_proc_v3::CommonResponse {
        status: envoy_service_ext_proc_v3::common_response::ResponseStatus::Continue as i32,
        ..Default::default()
    };

    let headers_response = envoy_service_ext_proc_v3::HeadersResponse {
        response: Some(common_response),
    };

    let metadata_map = {
        let mut map = BTreeMap::new();
        map.insert("static_key".to_string(), Value {
            kind: Some(prost_types::value::Kind::StringValue("static_value".to_string())),
        });
        map
    };

    envoy_service_ext_proc_v3::ProcessingResponse {
        dynamic_metadata: Some(Struct {
            fields: metadata_map,
        }),
        mode_override: Some(ProcessingMode {
            request_header_mode: HeaderSendMode::Send as i32,
            response_header_mode: HeaderSendMode::Send as i32,
            request_body_mode: BodySendMode::Buffered as i32,
            response_body_mode: BodySendMode::Buffered as i32,
            request_trailer_mode: HeaderSendMode::Send as i32,
            response_trailer_mode: HeaderSendMode::Send as i32,
        }),
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

        // Stream the incoming messages (but ignore them since we return a static response)
        while let Some(message) = request.get_mut().next().await {
            match message {
                Ok(_msg) => {
                    // Log the received message if needed
                    println!("Received gRPC message but sending static response.");

                    // Send the static response
                    let response = static_response();
                    println!("Sending static gRPC response: {:?}", response);

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

        // Return the static response stream
        println!("Closing the stream after sending static responses");
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

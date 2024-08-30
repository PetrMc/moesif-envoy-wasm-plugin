use envoy_ext_proc_proto::envoy::service::ext_proc::v3::{
    external_processor_server::ExternalProcessor, processing_request, ProcessingRequest,
    ProcessingResponse,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

use crate::config::Config;
use crate::root_context::EventRootContext;
use crate::utils::*;
use log::LevelFilter;

use crate::event::Event;
use chrono::Utc;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct MoesifGlooExtProcGrpcService {
    config: Arc<Config>, // Store the config in the service
    event_context: Arc<Mutex<EventRootContext>>,
}

impl MoesifGlooExtProcGrpcService {
    pub fn new(config: Config) -> Result<Self, String> {
        // Set the log level based on the config
        if config.env.debug {
            log::set_max_level(LevelFilter::Debug);
        } else {
            log::set_max_level(LevelFilter::Warn);
        }

        // Initialize EventRootContext with the loaded configuration
        let root_context = EventRootContext::new(config.clone());

        // Create the service instance
        let service = MoesifGlooExtProcGrpcService {
            config: Arc::new(config),
            event_context: Arc::new(Mutex::new(root_context)),
        };

        // Start periodic sending in the background
        service.start_periodic_sending();

        Ok(service)
    }

    fn start_periodic_sending(&self) {
        let event_context = Arc::clone(&self.event_context);
        let batch_max_wait = Duration::from_millis(self.config.env.batch_max_wait as u64);

        log::trace!(
            "Starting periodic sending with batch_max_wait: {:?}",
            batch_max_wait
        );

        tokio::spawn(async move {
            loop {
                log::trace!("Waiting for batch_max_wait period: {:?}", batch_max_wait);
                tokio::time::sleep(batch_max_wait).await;

                log::trace!(
                    "Periodic sending triggered after waiting for: {:?}",
                    batch_max_wait
                );
                let event_context = event_context.lock().await;

                log::trace!("Draining and sending events from the main buffer...");
                event_context.drain_and_send(1).await;

                log::trace!("Cleaning up and moving stale events from the temporary buffer...");
                event_context.cleanup_temporary_buffer(batch_max_wait).await;

                log::trace!("Periodic sending cycle complete.");
            }
        });
    }
}

#[tonic::async_trait]
impl ExternalProcessor for MoesifGlooExtProcGrpcService {
    type ProcessStream = ReceiverStream<Result<ProcessingResponse, Status>>;

    async fn process(
        &self,
        mut request: Request<Streaming<ProcessingRequest>>,
    ) -> Result<Response<Self::ProcessStream>, Status> {
        log::info!("Processing new gRPC request...");
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        let config = Arc::clone(&self.config);

        tokio::spawn({
            let event_context = Arc::clone(&self.event_context);
            async move {
                while let Some(message) = request.get_mut().next().await {
                    match message {
                        Ok(msg) => {
                            log::info!("Received message: {:?}", msg);

                            let mut event = Event::default();
                            event.request.time = Utc::now().to_rfc3339();
                            log::trace!("Generated request time: {}", event.request.time);

                            let mut response_headers = HashMap::new();

                            if let Some(processing_request::Request::RequestHeaders(headers_msg)) =
                                &msg.request
                            {
                                process_request_headers(
                                    &event_context,
                                    &config,
                                    &mut event,
                                    headers_msg,
                                    &mut response_headers,
                                )
                                .await;
                            }

                            if let Some(processing_request::Request::ResponseHeaders(
                                response_headers_msg,
                            )) = &msg.request
                            {
                                process_response_headers(&event_context, response_headers_msg)
                                    .await;
                            }
                            log::info!(
                                "Sending gRPC response with headers: {:?}",
                                response_headers
                            );
                            send_grpc_response(tx.clone(), response_with_headers(response_headers))
                                .await;
                        }

                        Err(e) => {
                            log::error!("Error receiving message: {:?}", e);
                            if tx
                                .send(Err(Status::internal("Error processing request")))
                                .await
                                .is_err()
                            {
                                log::error!("Error sending internal error response: {:?}", e);
                                break;
                            }
                        }
                    }
                }
                log::info!("Stream processing complete.");
            }
        });

        log::info!("Returning gRPC response stream.");
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

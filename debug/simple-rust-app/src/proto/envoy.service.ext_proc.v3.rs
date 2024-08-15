/// This represents the different types of messages that Envoy can send
/// to an external processing server.
/// [#next-free-field: 11]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProcessingRequest {
    /// Dynamic metadata associated with the request.
    #[prost(message, optional, tag = "8")]
    pub metadata_context: ::core::option::Option<Metadata>,
    /// The values of properties selected by the ``request_attributes``
    /// or ``response_attributes`` list in the configuration. Each entry
    /// in the list is populated from the standard
    /// :ref:`attributes <arch_overview_attributes>` supported across Envoy.
    #[prost(map = "string, message", tag = "9")]
    pub attributes:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost_types::Struct>,
    /// Specify whether the filter that sent this request is running in :ref:`observability_mode
    /// <envoy_v3_api_field_extensions.filters.http.ext_proc.v3.ExternalProcessor.observability_mode>`
    /// and defaults to false.
    ///
    /// * A value of ``false`` indicates that the server must respond
    ///   to this message by either sending back a matching ProcessingResponse message,
    ///   or by closing the stream.
    /// * A value of ``true`` indicates that the server should not respond to this message, as any
    ///   responses will be ignored. However, it may still close the stream to indicate that no more messages
    ///   are needed.
    ///
    #[prost(bool, tag = "10")]
    pub observability_mode: bool,
    /// Each request message will include one of the following sub-messages. Which
    /// ones are set for a particular HTTP request/response depend on the
    /// processing mode.
    #[prost(oneof = "processing_request::Request", tags = "2, 3, 4, 5, 6, 7")]
    pub request: ::core::option::Option<processing_request::Request>,
}
/// Nested message and enum types in `ProcessingRequest`.
pub mod processing_request {
    /// Each request message will include one of the following sub-messages. Which
    /// ones are set for a particular HTTP request/response depend on the
    /// processing mode.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Request {
        /// Information about the HTTP request headers, as well as peer info and additional
        /// properties. Unless ``observability_mode`` is ``true``, the server must send back a
        /// HeaderResponse message, an ImmediateResponse message, or close the stream.
        #[prost(message, tag = "2")]
        RequestHeaders(super::HttpHeaders),
        /// Information about the HTTP response headers, as well as peer info and additional
        /// properties. Unless ``observability_mode`` is ``true``, the server must send back a
        /// HeaderResponse message or close the stream.
        #[prost(message, tag = "3")]
        ResponseHeaders(super::HttpHeaders),
        /// A chunk of the HTTP request body. Unless ``observability_mode`` is true, the server must send back
        /// a BodyResponse message, an ImmediateResponse message, or close the stream.
        #[prost(message, tag = "4")]
        RequestBody(super::HttpBody),
        /// A chunk of the HTTP response body. Unless ``observability_mode`` is ``true``, the server must send back
        /// a BodyResponse message or close the stream.
        #[prost(message, tag = "5")]
        ResponseBody(super::HttpBody),
        /// The HTTP trailers for the request path. Unless ``observability_mode`` is ``true``, the server
        /// must send back a TrailerResponse message or close the stream.
        ///
        /// This message is only sent if the trailers processing mode is set to ``SEND`` and
        /// the original downstream request has trailers.
        #[prost(message, tag = "6")]
        RequestTrailers(super::HttpTrailers),
        /// The HTTP trailers for the response path. Unless ``observability_mode`` is ``true``, the server
        /// must send back a TrailerResponse message or close the stream.
        ///
        /// This message is only sent if the trailers processing mode is set to ``SEND`` and
        /// the original upstream response has trailers.
        #[prost(message, tag = "7")]
        ResponseTrailers(super::HttpTrailers),
    }
}
/// For every ProcessingRequest received by the server with the ``observability_mode`` field
/// set to false, the server must send back exactly one ProcessingResponse message.
/// [#next-free-field: 11]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProcessingResponse {
    /// Optional metadata that will be emitted as dynamic metadata to be consumed by
    /// following filters. This metadata will be placed in the namespace(s) specified by the top-level
    /// field name(s) of the struct.
    #[prost(message, optional, tag = "8")]
    pub dynamic_metadata: ::core::option::Option<::prost_types::Struct>,
    /// Override how parts of the HTTP request and response are processed
    /// for the duration of this particular request/response only. Servers
    /// may use this to intelligently control how requests are processed
    /// based on the headers and other metadata that they see.
    /// This field is only applicable when servers responding to the header requests.
    /// If it is set in the response to the body or trailer requests, it will be ignored by Envoy.
    /// It is also ignored by Envoy when the ext_proc filter config
    /// :ref:`allow_mode_override
    /// <envoy_v3_api_field_extensions.filters.http.ext_proc.v3.ExternalProcessor.allow_mode_override>`
    /// is set to false.
    #[prost(message, optional, tag = "9")]
    pub mode_override:
        ::core::option::Option<crate::envoy_extensions_filters_http_ext_proc_v3::ProcessingMode>,
    /// When ext_proc server receives a request message, in case it needs more
    /// time to process the message, it sends back a ProcessingResponse message
    /// with a new timeout value. When Envoy receives this response message,
    /// it ignores other fields in the response, just stop the original timer,
    /// which has the timeout value specified in
    /// :ref:`message_timeout
    /// <envoy_v3_api_field_extensions.filters.http.ext_proc.v3.ExternalProcessor.message_timeout>`
    /// and start a new timer with this ``override_message_timeout`` value and keep the
    /// Envoy ext_proc filter state machine intact.
    /// Has to be >= 1ms and <=
    /// :ref:`max_message_timeout <envoy_v3_api_field_extensions.filters.http.ext_proc.v3.ExternalProcessor.max_message_timeout>`
    /// Such message can be sent at most once in a particular Envoy ext_proc filter processing state.
    /// To enable this API, one has to set ``max_message_timeout`` to a number >= 1ms.
    #[prost(message, optional, tag = "10")]
    pub override_message_timeout: ::core::option::Option<::prost_types::Duration>,
    #[prost(oneof = "processing_response::Response", tags = "1, 2, 3, 4, 5, 6, 7")]
    pub response: ::core::option::Option<processing_response::Response>,
}
/// Nested message and enum types in `ProcessingResponse`.
pub mod processing_response {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Response {
        /// The server must send back this message in response to a message with the
        /// ``request_headers`` field set.
        #[prost(message, tag = "1")]
        RequestHeaders(super::HeadersResponse),
        /// The server must send back this message in response to a message with the
        /// ``response_headers`` field set.
        #[prost(message, tag = "2")]
        ResponseHeaders(super::HeadersResponse),
        /// The server must send back this message in response to a message with
        /// the ``request_body`` field set.
        #[prost(message, tag = "3")]
        RequestBody(super::BodyResponse),
        /// The server must send back this message in response to a message with
        /// the ``response_body`` field set.
        #[prost(message, tag = "4")]
        ResponseBody(super::BodyResponse),
        /// The server must send back this message in response to a message with
        /// the ``request_trailers`` field set.
        #[prost(message, tag = "5")]
        RequestTrailers(super::TrailersResponse),
        /// The server must send back this message in response to a message with
        /// the ``response_trailers`` field set.
        #[prost(message, tag = "6")]
        ResponseTrailers(super::TrailersResponse),
        /// If specified, attempt to create a locally generated response, send it
        /// downstream, and stop processing additional filters and ignore any
        /// additional messages received from the remote server for this request or
        /// response. If a response has already started -- for example, if this
        /// message is sent response to a ``response_body`` message -- then
        /// this will either ship the reply directly to the downstream codec,
        /// or reset the stream.
        #[prost(message, tag = "7")]
        ImmediateResponse(super::ImmediateResponse),
    }
}
// The following are messages that are sent to the server.

/// This message is sent to the external server when the HTTP request and responses
/// are first received.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HttpHeaders {
    /// The HTTP request headers. All header keys will be
    /// lower-cased, because HTTP header keys are case-insensitive.
    /// The header value is encoded in the
    /// :ref:`raw_value <envoy_v3_api_field_config.core.v3.HeaderValue.raw_value>` field.
    #[prost(message, optional, tag = "1")]
    pub headers: ::core::option::Option<crate::envoy_config_core_v3::HeaderMap>,
    /// [#not-implemented-hide:]
    /// This field is deprecated and not implemented. Attributes will be sent in
    /// the  top-level :ref:`attributes <envoy_v3_api_field_service.ext_proc.v3.ProcessingRequest.attributes`
    /// field.
    #[prost(map = "string, message", tag = "2")]
    pub attributes:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost_types::Struct>,
    /// If true, then there is no message body associated with this
    /// request or response.
    #[prost(bool, tag = "3")]
    pub end_of_stream: bool,
}
/// This message contains the message body that Envoy sends to the external server.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HttpBody {
    #[prost(bytes = "vec", tag = "1")]
    pub body: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag = "2")]
    pub end_of_stream: bool,
}
/// This message contains the trailers.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HttpTrailers {
    /// The header value is encoded in the
    /// :ref:`raw_value <envoy_v3_api_field_config.core.v3.HeaderValue.raw_value>` field.
    #[prost(message, optional, tag = "1")]
    pub trailers: ::core::option::Option<crate::envoy_config_core_v3::HeaderMap>,
}
// The following are messages that may be sent back by the server.

/// This message must be sent in response to an HttpHeaders message.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeadersResponse {
    #[prost(message, optional, tag = "1")]
    pub response: ::core::option::Option<CommonResponse>,
}
/// This message must be sent in response to an HttpTrailers message.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TrailersResponse {
    /// Instructions on how to manipulate the trailers
    #[prost(message, optional, tag = "1")]
    pub header_mutation: ::core::option::Option<HeaderMutation>,
}
/// This message must be sent in response to an HttpBody message.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BodyResponse {
    #[prost(message, optional, tag = "1")]
    pub response: ::core::option::Option<CommonResponse>,
}
/// This message contains common fields between header and body responses.
/// [#next-free-field: 6]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommonResponse {
    /// If set, provide additional direction on how the Envoy proxy should
    /// handle the rest of the HTTP filter chain.
    #[prost(enumeration = "common_response::ResponseStatus", tag = "1")]
    pub status: i32,
    /// Instructions on how to manipulate the headers. When responding to an
    /// HttpBody request, header mutations will only take effect if
    /// the current processing mode for the body is BUFFERED.
    #[prost(message, optional, tag = "2")]
    pub header_mutation: ::core::option::Option<HeaderMutation>,
    /// Replace the body of the last message sent to the remote server on this
    /// stream. If responding to an HttpBody request, simply replace or clear
    /// the body chunk that was sent with that request. Body mutations may take
    /// effect in response either to ``header`` or ``body`` messages. When it is
    /// in response to ``header`` messages, it only take effect if the
    /// :ref:`status <envoy_v3_api_field_service.ext_proc.v3.CommonResponse.status>`
    /// is set to CONTINUE_AND_REPLACE.
    #[prost(message, optional, tag = "3")]
    pub body_mutation: ::core::option::Option<BodyMutation>,
    /// [#not-implemented-hide:]
    /// Add new trailers to the message. This may be used when responding to either a
    /// HttpHeaders or HttpBody message, but only if this message is returned
    /// along with the CONTINUE_AND_REPLACE status.
    /// The header value is encoded in the
    /// :ref:`raw_value <envoy_v3_api_field_config.core.v3.HeaderValue.raw_value>` field.
    #[prost(message, optional, tag = "4")]
    pub trailers: ::core::option::Option<crate::envoy_config_core_v3::HeaderMap>,
    /// Clear the route cache for the current client request. This is necessary
    /// if the remote server modified headers that are used to calculate the route.
    /// This field is ignored in the response direction. This field is also ignored
    /// if the Envoy ext_proc filter is in the upstream filter chain.
    #[prost(bool, tag = "5")]
    pub clear_route_cache: bool,
}
/// Nested message and enum types in `CommonResponse`.
pub mod common_response {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum ResponseStatus {
        /// Apply the mutation instructions in this message to the
        /// request or response, and then continue processing the filter
        /// stream as normal. This is the default.
        Continue = 0,
        /// Apply the specified header mutation, replace the body with the body
        /// specified in the body mutation (if present), and do not send any
        /// further messages for this request or response even if the processing
        /// mode is configured to do so.
        ///
        /// When used in response to a request_headers or response_headers message,
        /// this status makes it possible to either completely replace the body
        /// while discarding the original body, or to add a body to a message that
        /// formerly did not have one.
        ///
        /// In other words, this response makes it possible to turn an HTTP GET
        /// into a POST, PUT, or PATCH.
        ContinueAndReplace = 1,
    }
}
/// This message causes the filter to attempt to create a locally
/// generated response, send it  downstream, stop processing
/// additional filters, and ignore any additional messages received
/// from the remote server for this request or response. If a response
/// has already started, then  this will either ship the reply directly
/// to the downstream codec, or reset the stream.
/// [#next-free-field: 6]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImmediateResponse {
    /// The response code to return
    #[prost(message, optional, tag = "1")]
    pub status: ::core::option::Option<crate::envoy_type_v3::HttpStatus>,
    /// Apply changes to the default headers, which will include content-type.
    #[prost(message, optional, tag = "2")]
    pub headers: ::core::option::Option<HeaderMutation>,
    /// The message body to return with the response which is sent using the
    /// text/plain content type, or encoded in the grpc-message header.
    #[prost(bytes = "vec", tag = "3")]
    pub body: ::prost::alloc::vec::Vec<u8>,
    /// If set, then include a gRPC status trailer.
    #[prost(message, optional, tag = "4")]
    pub grpc_status: ::core::option::Option<GrpcStatus>,
    /// A string detailing why this local reply was sent, which may be included
    /// in log and debug output (e.g. this populates the %RESPONSE_CODE_DETAILS%
    /// command operator field for use in access logging).
    #[prost(string, tag = "5")]
    pub details: ::prost::alloc::string::String,
}
/// This message specifies a gRPC status for an ImmediateResponse message.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GrpcStatus {
    /// The actual gRPC status
    #[prost(uint32, tag = "1")]
    pub status: u32,
}
/// Change HTTP headers or trailers by appending, replacing, or removing
/// headers.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeaderMutation {
    /// Add or replace HTTP headers. Attempts to set the value of
    /// any ``x-envoy`` header, and attempts to set the ``:method``,
    /// ``:authority``, ``:scheme``, or ``host`` headers will be ignored.
    /// The header value is encoded in the
    /// :ref:`raw_value <envoy_v3_api_field_config.core.v3.HeaderValue.raw_value>` field.
    #[prost(message, repeated, tag = "1")]
    pub set_headers: ::prost::alloc::vec::Vec<crate::envoy_config_core_v3::HeaderValueOption>,
    /// Remove these HTTP headers. Attempts to remove system headers --
    /// any header starting with ``:``, plus ``host`` -- will be ignored.
    #[prost(string, repeated, tag = "2")]
    pub remove_headers: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Replace the entire message body chunk received in the corresponding
/// HttpBody message with this new body, or clear the body.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BodyMutation {
    #[prost(oneof = "body_mutation::Mutation", tags = "1, 2")]
    pub mutation: ::core::option::Option<body_mutation::Mutation>,
}
/// Nested message and enum types in `BodyMutation`.
pub mod body_mutation {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Mutation {
        /// The entire body to replace
        #[prost(bytes, tag = "1")]
        Body(::prost::alloc::vec::Vec<u8>),
        /// Clear the corresponding body chunk
        #[prost(bool, tag = "2")]
        ClearBody(bool),
    }
}
#[doc = r" Generated client implementations."]
pub mod external_processor_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct ExternalProcessorClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ExternalProcessorClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> ExternalProcessorClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + Send + Sync + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ExternalProcessorClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            ExternalProcessorClient::new(InterceptedService::new(inner, interceptor))
        }
        #[doc = r" Compress requests with `gzip`."]
        #[doc = r""]
        #[doc = r" This requires the server to support it otherwise it might respond with an"]
        #[doc = r" error."]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        #[doc = r" Enable decompressing responses with `gzip`."]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        #[doc = " This begins the bidirectional stream that Envoy will use to"]
        #[doc = " give the server control over what the filter does. The actual"]
        #[doc = " protocol is described by the ProcessingRequest and ProcessingResponse"]
        #[doc = " messages below."]
        pub async fn process(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::ProcessingRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::ProcessingResponse>>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/envoy.service.ext_proc.v3.ExternalProcessor/Process",
            );
            self.inner
                .streaming(request.into_streaming_request(), path, codec)
                .await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod external_processor_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with ExternalProcessorServer."]
    #[async_trait]
    pub trait ExternalProcessor: Send + Sync + 'static {
        #[doc = "Server streaming response type for the Process method."]
        type ProcessStream: futures_core::Stream<Item = Result<super::ProcessingResponse, tonic::Status>>
            + Send
            + Sync
            + 'static;
        #[doc = " This begins the bidirectional stream that Envoy will use to"]
        #[doc = " give the server control over what the filter does. The actual"]
        #[doc = " protocol is described by the ProcessingRequest and ProcessingResponse"]
        #[doc = " messages below."]
        async fn process(
            &self,
            request: tonic::Request<tonic::Streaming<super::ProcessingRequest>>,
        ) -> Result<tonic::Response<Self::ProcessStream>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ExternalProcessorServer<T: ExternalProcessor> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: ExternalProcessor> ExternalProcessorServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ExternalProcessorServer<T>
    where
        T: ExternalProcessor,
        B: Body + Send + Sync + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/envoy.service.ext_proc.v3.ExternalProcessor/Process" => {
                    #[allow(non_camel_case_types)]
                    struct ProcessSvc<T: ExternalProcessor>(pub Arc<T>);
                    impl<T: ExternalProcessor>
                        tonic::server::StreamingService<super::ProcessingRequest>
                        for ProcessSvc<T>
                    {
                        type Response = super::ProcessingResponse;
                        type ResponseStream = T::ProcessStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<tonic::Streaming<super::ProcessingRequest>>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).process(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ProcessSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: ExternalProcessor> Clone for ExternalProcessorServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: ExternalProcessor> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ExternalProcessor> tonic::transport::NamedService for ExternalProcessorServer<T> {
        const NAME: &'static str = "envoy.service.ext_proc.v3.ExternalProcessor";
    }
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Metadata {
    #[prost(map="string, message", tag="1")]
    pub filter_metadata: ::std::collections::HashMap<String, ::prost_types::Struct>,
    
    #[prost(map="string, message", tag="2")]
    pub typed_filter_metadata: ::std::collections::HashMap<String, ::prost_types::Any>,
}

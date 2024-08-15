#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProcessingMode {
    #[prost(enumeration = "HeaderSendMode", tag = "1")]
    pub request_header_mode: i32, // or another type that matches your needs
    #[prost(enumeration = "HeaderSendMode", tag = "2")]
    pub response_header_mode: i32,
    #[prost(enumeration = "BodySendMode", tag = "3")]
    pub request_body_mode: i32,
    #[prost(enumeration = "BodySendMode", tag = "4")]
    pub response_body_mode: i32,
    #[prost(enumeration = "HeaderSendMode", tag = "5")]
    pub request_trailer_mode: i32,
    #[prost(enumeration = "HeaderSendMode", tag = "6")]
    pub response_trailer_mode: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ::prost::Enumeration)]
#[repr(i32)]
pub enum HeaderSendMode {
    Default = 0,
    Send = 1,
    Skip = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ::prost::Enumeration)]
#[repr(i32)]
pub enum BodySendMode {
    None = 0,
    Streamed = 1,
    Buffered = 2,
    BufferedPartial = 3,
}

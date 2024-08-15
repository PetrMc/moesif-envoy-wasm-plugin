#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ContextParams {
    #[prost(map = "string, string", tag = "1")]
    pub params: ::std::collections::HashMap<String, String>,
}


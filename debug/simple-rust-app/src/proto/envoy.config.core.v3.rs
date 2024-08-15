// [#protodoc-title: Extension configuration]

/// Message type for extension configuration.
/// [#next-major-version: revisit all existing typed_config that doesn't use this wrapper.].
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TypedExtensionConfig {
    /// The name of an extension. This is not used to select the extension, instead
    /// it serves the role of an opaque identifier.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// The typed config for the extension. The type URL will be used to identify
    /// the extension. In the case that the type URL is ``xds.type.v3.TypedStruct``
    /// (or, for historical reasons, ``udpa.type.v1.TypedStruct``), the inner type
    /// URL of ``TypedStruct`` will be utilized. See the
    /// :ref:`extension configuration overview
    /// <config_overview_extension_configuration>` for further details.
    #[prost(message, optional, tag = "2")]
    pub typed_config: ::core::option::Option<::prost_types::Any>,
}
// [#protodoc-title: Socket option]

/// Generic socket option message. This would be used to set socket options that
/// might not exist in upstream kernels or precompiled Envoy binaries.
///
/// For example:
///
/// .. code-block:: json
///
///  {
///    "description": "support tcp keep alive",
///    "state": 0,
///    "level": 1,
///    "name": 9,
///    "int_value": 1,
///  }
///
/// 1 means SOL_SOCKET and 9 means SO_KEEPALIVE on Linux.
/// With the above configuration, `TCP Keep-Alives <https://www.freesoft.org/CIE/RFC/1122/114.htm>`_
/// can be enabled in socket with Linux, which can be used in
/// :ref:`listener's<envoy_v3_api_field_config.listener.v3.Listener.socket_options>` or
/// :ref:`admin's <envoy_v3_api_field_config.bootstrap.v3.Admin.socket_options>` socket_options etc.
///
/// It should be noted that the name or level may have different values on different platforms.
/// [#next-free-field: 8]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SocketOption {
    /// An optional name to give this socket option for debugging, etc.
    /// Uniqueness is not required and no special meaning is assumed.
    #[prost(string, tag = "1")]
    pub description: ::prost::alloc::string::String,
    /// Corresponding to the level value passed to setsockopt, such as IPPROTO_TCP
    #[prost(int64, tag = "2")]
    pub level: i64,
    /// The numeric name as passed to setsockopt
    #[prost(int64, tag = "3")]
    pub name: i64,
    /// The state in which the option will be applied. When used in BindConfig
    /// STATE_PREBIND is currently the only valid value.
    #[prost(enumeration = "socket_option::SocketState", tag = "6")]
    pub state: i32,
    /// Apply the socket option to the specified `socket type <https://linux.die.net/man/2/socket>`_.
    /// If not specified, the socket option will be applied to all socket types.
    #[prost(message, optional, tag = "7")]
    pub r#type: ::core::option::Option<socket_option::SocketType>,
    #[prost(oneof = "socket_option::Value", tags = "4, 5")]
    pub value: ::core::option::Option<socket_option::Value>,
}
/// Nested message and enum types in `SocketOption`.
pub mod socket_option {
    /// The `socket type <https://linux.die.net/man/2/socket>`_ to apply the socket option to.
    /// Only one field should be set. If multiple fields are set, the precedence order will determine
    /// the selected one. If none of the fields is set, the socket option will be applied to all socket types.
    ///
    /// For example:
    /// If :ref:`stream <envoy_v3_api_field_config.core.v3.SocketOption.SocketType.stream>` is set,
    /// it takes precedence over :ref:`datagram <envoy_v3_api_field_config.core.v3.SocketOption.SocketType.datagram>`.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct SocketType {
        /// Apply the socket option to the stream socket type.
        #[prost(message, optional, tag = "1")]
        pub stream: ::core::option::Option<socket_type::Stream>,
        /// Apply the socket option to the datagram socket type.
        #[prost(message, optional, tag = "2")]
        pub datagram: ::core::option::Option<socket_type::Datagram>,
    }
    /// Nested message and enum types in `SocketType`.
    pub mod socket_type {
        /// The stream socket type.
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Stream {}
        /// The datagram socket type.
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Datagram {}
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum SocketState {
        /// Socket options are applied after socket creation but before binding the socket to a port
        StatePrebind = 0,
        /// Socket options are applied after binding the socket to a port but before calling listen()
        StateBound = 1,
        /// Socket options are applied after calling listen()
        StateListening = 2,
    }
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        /// Because many sockopts take an int value.
        #[prost(int64, tag = "4")]
        IntValue(i64),
        /// Otherwise it's a byte buffer.
        #[prost(bytes, tag = "5")]
        BufValue(::prost::alloc::vec::Vec<u8>),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SocketOptionsOverride {
    #[prost(message, repeated, tag = "1")]
    pub socket_options: ::prost::alloc::vec::Vec<SocketOption>,
}
// [#protodoc-title: Network addresses]

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pipe {
    /// Unix Domain Socket path. On Linux, paths starting with '@' will use the
    /// abstract namespace. The starting '@' is replaced by a null byte by Envoy.
    /// Paths starting with '@' will result in an error in environments other than
    /// Linux.
    #[prost(string, tag = "1")]
    pub path: ::prost::alloc::string::String,
    /// The mode for the Pipe. Not applicable for abstract sockets.
    #[prost(uint32, tag = "2")]
    pub mode: u32,
}
/// The address represents an envoy internal listener.
/// [#comment: TODO(asraa): When address available, remove workaround from test/server/server_fuzz_test.cc:30.]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EnvoyInternalAddress {
    /// Specifies an endpoint identifier to distinguish between multiple endpoints for the same internal listener in a
    /// single upstream pool. Only used in the upstream addresses for tracking changes to individual endpoints. This, for
    /// example, may be set to the final destination IP for the target internal listener.
    #[prost(string, tag = "2")]
    pub endpoint_id: ::prost::alloc::string::String,
    #[prost(oneof = "envoy_internal_address::AddressNameSpecifier", tags = "1")]
    pub address_name_specifier:
        ::core::option::Option<envoy_internal_address::AddressNameSpecifier>,
}
/// Nested message and enum types in `EnvoyInternalAddress`.
pub mod envoy_internal_address {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum AddressNameSpecifier {
        /// Specifies the :ref:`name <envoy_v3_api_field_config.listener.v3.Listener.name>` of the
        /// internal listener.
        #[prost(string, tag = "1")]
        ServerListenerName(::prost::alloc::string::String),
    }
}
/// [#next-free-field: 7]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SocketAddress {
    #[prost(enumeration = "socket_address::Protocol", tag = "1")]
    pub protocol: i32,
    /// The address for this socket. :ref:`Listeners <config_listeners>` will bind
    /// to the address. An empty address is not allowed. Specify ``0.0.0.0`` or ``::``
    /// to bind to any address. [#comment:TODO(zuercher) reinstate when implemented:
    /// It is possible to distinguish a Listener address via the prefix/suffix matching
    /// in :ref:`FilterChainMatch <envoy_v3_api_msg_config.listener.v3.FilterChainMatch>`.] When used
    /// within an upstream :ref:`BindConfig <envoy_v3_api_msg_config.core.v3.BindConfig>`, the address
    /// controls the source address of outbound connections. For :ref:`clusters
    /// <envoy_v3_api_msg_config.cluster.v3.Cluster>`, the cluster type determines whether the
    /// address must be an IP (``STATIC`` or ``EDS`` clusters) or a hostname resolved by DNS
    /// (``STRICT_DNS`` or ``LOGICAL_DNS`` clusters). Address resolution can be customized
    /// via :ref:`resolver_name <envoy_v3_api_field_config.core.v3.SocketAddress.resolver_name>`.
    #[prost(string, tag = "2")]
    pub address: ::prost::alloc::string::String,
    /// The name of the custom resolver. This must have been registered with Envoy. If
    /// this is empty, a context dependent default applies. If the address is a concrete
    /// IP address, no resolution will occur. If address is a hostname this
    /// should be set for resolution other than DNS. Specifying a custom resolver with
    /// ``STRICT_DNS`` or ``LOGICAL_DNS`` will generate an error at runtime.
    #[prost(string, tag = "5")]
    pub resolver_name: ::prost::alloc::string::String,
    /// When binding to an IPv6 address above, this enables `IPv4 compatibility
    /// <https://tools.ietf.org/html/rfc3493#page-11>`_. Binding to ``::`` will
    /// allow both IPv4 and IPv6 connections, with peer IPv4 addresses mapped into
    /// IPv6 space as ``::FFFF:<IPv4-address>``.
    #[prost(bool, tag = "6")]
    pub ipv4_compat: bool,
    #[prost(oneof = "socket_address::PortSpecifier", tags = "3, 4")]
    pub port_specifier: ::core::option::Option<socket_address::PortSpecifier>,
}
/// Nested message and enum types in `SocketAddress`.
pub mod socket_address {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Protocol {
        Tcp = 0,
        Udp = 1,
    }
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum PortSpecifier {
        #[prost(uint32, tag = "3")]
        PortValue(u32),
        /// This is only valid if :ref:`resolver_name
        /// <envoy_v3_api_field_config.core.v3.SocketAddress.resolver_name>` is specified below and the
        /// named resolver is capable of named port resolution.
        #[prost(string, tag = "4")]
        NamedPort(::prost::alloc::string::String),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TcpKeepalive {
    /// Maximum number of keepalive probes to send without response before deciding
    /// the connection is dead. Default is to use the OS level configuration (unless
    /// overridden, Linux defaults to 9.)
    #[prost(message, optional, tag = "1")]
    pub keepalive_probes: ::core::option::Option<u32>,
    /// The number of seconds a connection needs to be idle before keep-alive probes
    /// start being sent. Default is to use the OS level configuration (unless
    /// overridden, Linux defaults to 7200s (i.e., 2 hours.)
    #[prost(message, optional, tag = "2")]
    pub keepalive_time: ::core::option::Option<u32>,
    /// The number of seconds between keep-alive probes. Default is to use the OS
    /// level configuration (unless overridden, Linux defaults to 75s.)
    #[prost(message, optional, tag = "3")]
    pub keepalive_interval: ::core::option::Option<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExtraSourceAddress {
    /// The additional address to bind.
    #[prost(message, optional, tag = "1")]
    pub address: ::core::option::Option<SocketAddress>,
    /// Additional socket options that may not be present in Envoy source code or
    /// precompiled binaries. If specified, this will override the
    /// :ref:`socket_options <envoy_v3_api_field_config.core.v3.BindConfig.socket_options>`
    /// in the BindConfig. If specified with no
    /// :ref:`socket_options <envoy_v3_api_field_config.core.v3.SocketOptionsOverride.socket_options>`
    /// or an empty list of :ref:`socket_options <envoy_v3_api_field_config.core.v3.SocketOptionsOverride.socket_options>`,
    /// it means no socket option will apply.
    #[prost(message, optional, tag = "2")]
    pub socket_options: ::core::option::Option<SocketOptionsOverride>,
}
/// [#next-free-field: 7]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BindConfig {
    /// The address to bind to when creating a socket.
    #[prost(message, optional, tag = "1")]
    pub source_address: ::core::option::Option<SocketAddress>,
    /// Whether to set the ``IP_FREEBIND`` option when creating the socket. When this
    /// flag is set to true, allows the :ref:`source_address
    /// <envoy_v3_api_field_config.core.v3.BindConfig.source_address>` to be an IP address
    /// that is not configured on the system running Envoy. When this flag is set
    /// to false, the option ``IP_FREEBIND`` is disabled on the socket. When this
    /// flag is not set (default), the socket is not modified, i.e. the option is
    /// neither enabled nor disabled.
    #[prost(message, optional, tag = "2")]
    pub freebind: ::core::option::Option<bool>,
    /// Additional socket options that may not be present in Envoy source code or
    /// precompiled binaries.
    #[prost(message, repeated, tag = "3")]
    pub socket_options: ::prost::alloc::vec::Vec<SocketOption>,
    /// Extra source addresses appended to the address specified in the ``source_address``
    /// field. This enables to specify multiple source addresses.
    /// The source address selection is determined by :ref:`local_address_selector
    /// <envoy_v3_api_field_config.core.v3.BindConfig.local_address_selector>`.
    #[prost(message, repeated, tag = "5")]
    pub extra_source_addresses: ::prost::alloc::vec::Vec<ExtraSourceAddress>,
    /// Deprecated by
    /// :ref:`extra_source_addresses <envoy_v3_api_field_config.core.v3.BindConfig.extra_source_addresses>`
    #[deprecated]
    #[prost(message, repeated, tag = "4")]
    pub additional_source_addresses: ::prost::alloc::vec::Vec<SocketAddress>,
    /// Custom local address selector to override the default (i.e.
    /// :ref:`DefaultLocalAddressSelector
    /// <envoy_v3_api_msg_config.upstream.local_address_selector.v3.DefaultLocalAddressSelector>`).
    /// [#extension-category: envoy.upstream.local_address_selector]
    #[prost(message, optional, tag = "6")]
    pub local_address_selector: ::core::option::Option<TypedExtensionConfig>,
}
/// Addresses specify either a logical or physical address and port, which are
/// used to tell Envoy where to bind/listen, connect to upstream and find
/// management servers.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Address {
    #[prost(oneof = "address::Address", tags = "1, 2, 3")]
    pub address: ::core::option::Option<address::Address>,
}
/// Nested message and enum types in `Address`.
pub mod address {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Address {
        #[prost(message, tag = "1")]
        SocketAddress(super::SocketAddress),
        #[prost(message, tag = "2")]
        Pipe(super::Pipe),
        /// Specifies a user-space address handled by :ref:`internal listeners
        /// <envoy_v3_api_field_config.listener.v3.Listener.internal_listener>`.
        #[prost(message, tag = "3")]
        EnvoyInternalAddress(super::EnvoyInternalAddress),
    }
}
/// CidrRange specifies an IP Address and a prefix length to construct
/// the subnet mask for a `CIDR <https://tools.ietf.org/html/rfc4632>`_ range.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CidrRange {
    /// IPv4 or IPv6 address, e.g. ``192.0.0.0`` or ``2001:db8::``.
    #[prost(string, tag = "1")]
    pub address_prefix: ::prost::alloc::string::String,
    /// Length of prefix, e.g. 0, 32. Defaults to 0 when unset.
    #[prost(message, optional, tag = "2")]
    pub prefix_len: ::core::option::Option<u32>,
}
// [#protodoc-title: Backoff strategy]

/// Configuration defining a jittered exponential back off strategy.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BackoffStrategy {
    /// The base interval to be used for the next back off computation. It should
    /// be greater than zero and less than or equal to :ref:`max_interval
    /// <envoy_v3_api_field_config.core.v3.BackoffStrategy.max_interval>`.
    #[prost(message, optional, tag = "1")]
    pub base_interval: ::core::option::Option<::prost_types::Duration>,
    /// Specifies the maximum interval between retries. This parameter is optional,
    /// but must be greater than or equal to the :ref:`base_interval
    /// <envoy_v3_api_field_config.core.v3.BackoffStrategy.base_interval>` if set. The default
    /// is 10 times the :ref:`base_interval
    /// <envoy_v3_api_field_config.core.v3.BackoffStrategy.base_interval>`.
    #[prost(message, optional, tag = "2")]
    pub max_interval: ::core::option::Option<::prost_types::Duration>,
}
// [#protodoc-title: HTTP service URI ]

/// Envoy external URI descriptor
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HttpUri {
    /// The HTTP server URI. It should be a full FQDN with protocol, host and path.
    ///
    /// Example:
    ///
    /// .. code-block:: yaml
    ///
    ///    uri: https://www.googleapis.com/oauth2/v1/certs
    ///
    #[prost(string, tag = "1")]
    pub uri: ::prost::alloc::string::String,
    /// Sets the maximum duration in milliseconds that a response can take to arrive upon request.
    #[prost(message, optional, tag = "3")]
    pub timeout: ::core::option::Option<::prost_types::Duration>,
    /// Specify how ``uri`` is to be fetched. Today, this requires an explicit
    /// cluster, but in the future we may support dynamic cluster creation or
    /// inline DNS resolution. See `issue
    /// <https://github.com/envoyproxy/envoy/issues/1606>`_.
    #[prost(oneof = "http_uri::HttpUpstreamType", tags = "2")]
    pub http_upstream_type: ::core::option::Option<http_uri::HttpUpstreamType>,
}
/// Nested message and enum types in `HttpUri`.
pub mod http_uri {
    /// Specify how ``uri`` is to be fetched. Today, this requires an explicit
    /// cluster, but in the future we may support dynamic cluster creation or
    /// inline DNS resolution. See `issue
    /// <https://github.com/envoyproxy/envoy/issues/1606>`_.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum HttpUpstreamType {
        /// A cluster is created in the Envoy "cluster_manager" config
        /// section. This field specifies the cluster name.
        ///
        /// Example:
        ///
        /// .. code-block:: yaml
        ///
        ///    cluster: jwks_cluster
        ///
        #[prost(string, tag = "2")]
        Cluster(::prost::alloc::string::String),
    }
}
/// Identifies location of where either Envoy runs or where upstream hosts run.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Locality {
    /// Region this :ref:`zone <envoy_v3_api_field_config.core.v3.Locality.zone>` belongs to.
    #[prost(string, tag = "1")]
    pub region: ::prost::alloc::string::String,
    /// Defines the local service zone where Envoy is running. Though optional, it
    /// should be set if discovery service routing is used and the discovery
    /// service exposes :ref:`zone data <envoy_v3_api_field_config.endpoint.v3.LocalityLbEndpoints.locality>`,
    /// either in this message or via :option:`--service-zone`. The meaning of zone
    /// is context dependent, e.g. `Availability Zone (AZ)
    /// <https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-regions-availability-zones.html>`_
    /// on AWS, `Zone <https://cloud.google.com/compute/docs/regions-zones/>`_ on
    /// GCP, etc.
    #[prost(string, tag = "2")]
    pub zone: ::prost::alloc::string::String,
    /// When used for locality of upstream hosts, this field further splits zone
    /// into smaller chunks of sub-zones so they can be load balanced
    /// independently.
    #[prost(string, tag = "3")]
    pub sub_zone: ::prost::alloc::string::String,
}
/// BuildVersion combines SemVer version of extension with free-form build information
/// (i.e. 'alpha', 'private-build') as a set of strings.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BuildVersion {
    /// SemVer version of extension.
    #[prost(message, optional, tag = "1")]
    pub version: ::core::option::Option<crate::envoy_type_v3::SemanticVersion>,
    /// Free-form build information.
    /// Envoy defines several well known keys in the source/common/version/version.h file
    #[prost(message, optional, tag = "2")]
    pub metadata: ::core::option::Option<::prost_types::Struct>,
}
/// Version and identification for an Envoy extension.
/// [#next-free-field: 7]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Extension {
    /// This is the name of the Envoy filter as specified in the Envoy
    /// configuration, e.g. envoy.filters.http.router, com.acme.widget.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Category of the extension.
    /// Extension category names use reverse DNS notation. For instance "envoy.filters.listener"
    /// for Envoy's built-in listener filters or "com.acme.filters.http" for HTTP filters from
    /// acme.com vendor.
    /// [#comment:TODO(yanavlasov): Link to the doc with existing envoy category names.]
    #[prost(string, tag = "2")]
    pub category: ::prost::alloc::string::String,
    /// [#not-implemented-hide:] Type descriptor of extension configuration proto.
    /// [#comment:TODO(yanavlasov): Link to the doc with existing configuration protos.]
    /// [#comment:TODO(yanavlasov): Add tests when PR #9391 lands.]
    #[deprecated]
    #[prost(string, tag = "3")]
    pub type_descriptor: ::prost::alloc::string::String,
    /// The version is a property of the extension and maintained independently
    /// of other extensions and the Envoy API.
    /// This field is not set when extension did not provide version information.
    #[prost(message, optional, tag = "4")]
    pub version: ::core::option::Option<BuildVersion>,
    /// Indicates that the extension is present but was disabled via dynamic configuration.
    #[prost(bool, tag = "5")]
    pub disabled: bool,
    /// Type URLs of extension configuration protos.
    #[prost(string, repeated, tag = "6")]
    pub type_urls: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Identifies a specific Envoy instance. The node identifier is presented to the
/// management server, which may use this identifier to distinguish per Envoy
/// configuration for serving.
/// [#next-free-field: 13]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Node {
    /// An opaque node identifier for the Envoy node. This also provides the local
    /// service node name. It should be set if any of the following features are
    /// used: :ref:`statsd <arch_overview_statistics>`, :ref:`CDS
    /// <config_cluster_manager_cds>`, and :ref:`HTTP tracing
    /// <arch_overview_tracing>`, either in this message or via
    /// :option:`--service-node`.
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    /// Defines the local service cluster name where Envoy is running. Though
    /// optional, it should be set if any of the following features are used:
    /// :ref:`statsd <arch_overview_statistics>`, :ref:`health check cluster
    /// verification
    /// <envoy_v3_api_field_config.core.v3.HealthCheck.HttpHealthCheck.service_name_matcher>`,
    /// :ref:`runtime override directory <envoy_v3_api_msg_config.bootstrap.v3.Runtime>`,
    /// :ref:`user agent addition
    /// <envoy_v3_api_field_extensions.filters.network.http_connection_manager.v3.HttpConnectionManager.add_user_agent>`,
    /// :ref:`HTTP global rate limiting <config_http_filters_rate_limit>`,
    /// :ref:`CDS <config_cluster_manager_cds>`, and :ref:`HTTP tracing
    /// <arch_overview_tracing>`, either in this message or via
    /// :option:`--service-cluster`.
    #[prost(string, tag = "2")]
    pub cluster: ::prost::alloc::string::String,
    /// Opaque metadata extending the node identifier. Envoy will pass this
    /// directly to the management server.
    #[prost(message, optional, tag = "3")]
    pub metadata: ::core::option::Option<::prost_types::Struct>,
    /// Map from xDS resource type URL to dynamic context parameters. These may vary at runtime (unlike
    /// other fields in this message). For example, the xDS client may have a shard identifier that
    /// changes during the lifetime of the xDS client. In Envoy, this would be achieved by updating the
    /// dynamic context on the Server::Instance's LocalInfo context provider. The shard ID dynamic
    /// parameter then appears in this field during future discovery requests.
    #[prost(map = "string, message", tag = "12")]
    pub dynamic_parameters: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        crate::xds_core_v3::ContextParams,
    >,
    /// Locality specifying where the Envoy instance is running.
    #[prost(message, optional, tag = "4")]
    pub locality: ::core::option::Option<Locality>,
    /// Free-form string that identifies the entity requesting config.
    /// E.g. "envoy" or "grpc"
    #[prost(string, tag = "6")]
    pub user_agent_name: ::prost::alloc::string::String,
    /// List of extensions and their versions supported by the node.
    #[prost(message, repeated, tag = "9")]
    pub extensions: ::prost::alloc::vec::Vec<Extension>,
    /// Client feature support list. These are well known features described
    /// in the Envoy API repository for a given major version of an API. Client features
    /// use reverse DNS naming scheme, for example ``com.acme.feature``.
    /// See :ref:`the list of features <client_features>` that xDS client may
    /// support.
    #[prost(string, repeated, tag = "10")]
    pub client_features: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Known listening ports on the node as a generic hint to the management server
    /// for filtering :ref:`listeners <config_listeners>` to be returned. For example,
    /// if there is a listener bound to port 80, the list can optionally contain the
    /// SocketAddress ``(0.0.0.0,80)``. The field is optional and just a hint.
    #[deprecated]
    #[prost(message, repeated, tag = "11")]
    pub listening_addresses: ::prost::alloc::vec::Vec<Address>,
    #[prost(oneof = "node::UserAgentVersionType", tags = "7, 8")]
    pub user_agent_version_type: ::core::option::Option<node::UserAgentVersionType>,
}
/// Nested message and enum types in `Node`.
pub mod node {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum UserAgentVersionType {
        /// Free-form string that identifies the version of the entity requesting config.
        /// E.g. "1.12.2" or "abcd1234", or "SpecialEnvoyBuild"
        #[prost(string, tag = "7")]
        UserAgentVersion(::prost::alloc::string::String),
        /// Structured version of the entity requesting config.
        #[prost(message, tag = "8")]
        UserAgentBuildVersion(super::BuildVersion),
    }
}
/// Runtime derived uint32 with a default when not specified.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RuntimeUInt32 {
    /// Default value if runtime value is not available.
    #[prost(uint32, tag = "2")]
    pub default_value: u32,
    /// Runtime key to get value for comparison. This value is used if defined.
    #[prost(string, tag = "3")]
    pub runtime_key: ::prost::alloc::string::String,
}
/// Runtime derived percentage with a default when not specified.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RuntimePercent {
    /// Default value if runtime value is not available.
    #[prost(message, optional, tag = "1")]
    pub default_value: ::core::option::Option<crate::envoy_type_v3::Percent>,
    /// Runtime key to get value for comparison. This value is used if defined.
    #[prost(string, tag = "2")]
    pub runtime_key: ::prost::alloc::string::String,
}
/// Runtime derived double with a default when not specified.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RuntimeDouble {
    /// Default value if runtime value is not available.
    #[prost(double, tag = "1")]
    pub default_value: f64,
    /// Runtime key to get value for comparison. This value is used if defined.
    #[prost(string, tag = "2")]
    pub runtime_key: ::prost::alloc::string::String,
}
/// Runtime derived bool with a default when not specified.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RuntimeFeatureFlag {
    /// Default value if runtime value is not available.
    #[prost(message, optional, tag = "1")]
    pub default_value: ::core::option::Option<bool>,
    /// Runtime key to get value for comparison. This value is used if defined. The boolean value must
    /// be represented via its
    /// `canonical JSON encoding <https://developers.google.com/protocol-buffers/docs/proto3#json>`_.
    #[prost(string, tag = "2")]
    pub runtime_key: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyValue {
    /// The key of the key/value pair.
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    /// The value of the key/value pair.
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
/// Key/value pair plus option to control append behavior. This is used to specify
/// key/value pairs that should be appended to a set of existing key/value pairs.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyValueAppend {
    /// Key/value pair entry that this option to append or overwrite.
    #[prost(message, optional, tag = "1")]
    pub entry: ::core::option::Option<KeyValue>,
    /// Describes the action taken to append/overwrite the given value for an existing
    /// key or to only add this key if it's absent.
    #[prost(enumeration = "key_value_append::KeyValueAppendAction", tag = "2")]
    pub action: i32,
}
/// Nested message and enum types in `KeyValueAppend`.
pub mod key_value_append {
    /// Describes the supported actions types for key/value pair append action.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum KeyValueAppendAction {
        /// If the key already exists, this action will result in the following behavior:
        ///
        /// - Comma-concatenated value if multiple values are not allowed.
        /// - New value added to the list of values if multiple values are allowed.
        ///
        /// If the key doesn't exist then this will add pair with specified key and value.
        AppendIfExistsOrAdd = 0,
        /// This action will add the key/value pair if it doesn't already exist. If the
        /// key already exists then this will be a no-op.
        AddIfAbsent = 1,
        /// This action will overwrite the specified value by discarding any existing
        /// values if the key already exists. If the key doesn't exist then this will add
        /// the pair with specified key and value.
        OverwriteIfExistsOrAdd = 2,
        /// This action will overwrite the specified value by discarding any existing
        /// values if the key already exists. If the key doesn't exist then this will
        /// be no-op.
        OverwriteIfExists = 3,
    }
}
/// Key/value pair to append or remove.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyValueMutation {
    /// Key/value pair to append or overwrite. Only one of ``append`` or ``remove`` can be set.
    #[prost(message, optional, tag = "1")]
    pub append: ::core::option::Option<KeyValueAppend>,
    /// Key to remove. Only one of ``append`` or ``remove`` can be set.
    #[prost(string, tag = "2")]
    pub remove: ::prost::alloc::string::String,
}
/// Query parameter name/value pair.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParameter {
    /// The key of the query parameter. Case sensitive.
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    /// The value of the query parameter.
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
}
/// Header name/value pair.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeaderValue {
    /// Header name.
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    /// Header value.
    ///
    /// The same :ref:`format specifier <config_access_log_format>` as used for
    /// :ref:`HTTP access logging <config_access_log>` applies here, however
    /// unknown header values are replaced with the empty string instead of ``-``.
    /// Header value is encoded as string. This does not work for non-utf8 characters.
    /// Only one of ``value`` or ``raw_value`` can be set.
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
    /// Header value is encoded as bytes which can support non-utf8 characters.
    /// Only one of ``value`` or ``raw_value`` can be set.
    #[prost(bytes = "vec", tag = "3")]
    pub raw_value: ::prost::alloc::vec::Vec<u8>,
}
/// A directory that is watched for changes, e.g. by inotify on Linux. Move/rename
/// events inside this directory trigger the watch.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WatchedDirectory {
    /// Directory path to watch.
    #[prost(string, tag = "1")]
    pub path: ::prost::alloc::string::String,
}
/// Data source consisting of a file, an inline value, or an environment variable.
/// [#next-free-field: 6]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DataSource {
    /// Watched directory that is watched for file changes. If this is set explicitly, the file
    /// specified in the ``filename`` field will be reloaded when relevant file move events occur.
    ///
    /// .. note::
    ///   This field only makes sense when the ``filename`` field is set.
    ///
    /// .. note::
    ///   Envoy only updates when the file is replaced by a file move, and not when the file is
    ///   edited in place.
    ///
    /// .. note::
    ///   Not all use cases of ``DataSource`` support watching directories. It depends on the
    ///   specific usage of the ``DataSource``. See the documentation of the parent message for
    ///   details.
    #[prost(message, optional, tag = "5")]
    pub watched_directory: ::core::option::Option<WatchedDirectory>,
    #[prost(oneof = "data_source::Specifier", tags = "1, 2, 3, 4")]
    pub specifier: ::core::option::Option<data_source::Specifier>,
}
/// Nested message and enum types in `DataSource`.
pub mod data_source {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Specifier {
        /// Local filesystem data source.
        #[prost(string, tag = "1")]
        Filename(::prost::alloc::string::String),
        /// Bytes inlined in the configuration.
        #[prost(bytes, tag = "2")]
        InlineBytes(::prost::alloc::vec::Vec<u8>),
        /// String inlined in the configuration.
        #[prost(string, tag = "3")]
        InlineString(::prost::alloc::string::String),
        /// Environment variable data source.
        #[prost(string, tag = "4")]
        EnvironmentVariable(::prost::alloc::string::String),
    }
}
/// The message specifies the retry policy of remote data source when fetching fails.
/// [#next-free-field: 7]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RetryPolicy {
    /// Specifies parameters that control :ref:`retry backoff strategy <envoy_v3_api_msg_config.core.v3.BackoffStrategy>`.
    /// This parameter is optional, in which case the default base interval is 1000 milliseconds. The
    /// default maximum interval is 10 times the base interval.
    #[prost(message, optional, tag = "1")]
    pub retry_back_off: ::core::option::Option<BackoffStrategy>,
    /// Specifies the allowed number of retries. This parameter is optional and
    /// defaults to 1.
    #[prost(message, optional, tag = "2")]
    pub num_retries: ::core::option::Option<u32>,
    /// For details, see :ref:`retry_on <envoy_v3_api_field_config.route.v3.RetryPolicy.retry_on>`.
    #[prost(string, tag = "3")]
    pub retry_on: ::prost::alloc::string::String,
    /// For details, see :ref:`retry_priority <envoy_v3_api_field_config.route.v3.RetryPolicy.retry_priority>`.
    #[prost(message, optional, tag = "4")]
    pub retry_priority: ::core::option::Option<retry_policy::RetryPriority>,
    /// For details, see :ref:`RetryHostPredicate <envoy_v3_api_field_config.route.v3.RetryPolicy.retry_host_predicate>`.
    #[prost(message, repeated, tag = "5")]
    pub retry_host_predicate: ::prost::alloc::vec::Vec<retry_policy::RetryHostPredicate>,
    /// For details, see :ref:`host_selection_retry_max_attempts <envoy_v3_api_field_config.route.v3.RetryPolicy.host_selection_retry_max_attempts>`.
    #[prost(int64, tag = "6")]
    pub host_selection_retry_max_attempts: i64,
}
/// Nested message and enum types in `RetryPolicy`.
pub mod retry_policy {
    /// See :ref:`RetryPriority <envoy_v3_api_field_config.route.v3.RetryPolicy.retry_priority>`.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RetryPriority {
        #[prost(string, tag = "1")]
        pub name: ::prost::alloc::string::String,
        #[prost(oneof = "retry_priority::ConfigType", tags = "2")]
        pub config_type: ::core::option::Option<retry_priority::ConfigType>,
    }
    /// Nested message and enum types in `RetryPriority`.
    pub mod retry_priority {
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum ConfigType {
            #[prost(message, tag = "2")]
            TypedConfig(::prost_types::Any),
        }
    }
    /// See :ref:`RetryHostPredicate <envoy_v3_api_field_config.route.v3.RetryPolicy.retry_host_predicate>`.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RetryHostPredicate {
        #[prost(string, tag = "1")]
        pub name: ::prost::alloc::string::String,
        #[prost(oneof = "retry_host_predicate::ConfigType", tags = "2")]
        pub config_type: ::core::option::Option<retry_host_predicate::ConfigType>,
    }
    /// Nested message and enum types in `RetryHostPredicate`.
    pub mod retry_host_predicate {
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum ConfigType {
            #[prost(message, tag = "2")]
            TypedConfig(::prost_types::Any),
        }
    }
}
/// The message specifies how to fetch data from remote and how to verify it.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RemoteDataSource {
    /// The HTTP URI to fetch the remote data.
    #[prost(message, optional, tag = "1")]
    pub http_uri: ::core::option::Option<HttpUri>,
    /// SHA256 string for verifying data.
    #[prost(string, tag = "2")]
    pub sha256: ::prost::alloc::string::String,
    /// Retry policy for fetching remote data.
    #[prost(message, optional, tag = "3")]
    pub retry_policy: ::core::option::Option<RetryPolicy>,
}
/// Async data source which support async data fetch.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AsyncDataSource {
    #[prost(oneof = "async_data_source::Specifier", tags = "1, 2")]
    pub specifier: ::core::option::Option<async_data_source::Specifier>,
}
/// Nested message and enum types in `AsyncDataSource`.
pub mod async_data_source {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Specifier {
        /// Local async data source.
        #[prost(message, tag = "1")]
        Local(super::DataSource),
        /// Remote async data source.
        #[prost(message, tag = "2")]
        Remote(super::RemoteDataSource),
    }
}
/// Configuration for transport socket in :ref:`listeners <config_listeners>` and
/// :ref:`clusters <envoy_v3_api_msg_config.cluster.v3.Cluster>`. If the configuration is
/// empty, a default transport socket implementation and configuration will be
/// chosen based on the platform and existence of tls_context.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransportSocket {
    /// The name of the transport socket to instantiate. The name must match a supported transport
    /// socket implementation.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Implementation specific configuration which depends on the implementation being instantiated.
    /// See the supported transport socket implementations for further documentation.
    #[prost(oneof = "transport_socket::ConfigType", tags = "3")]
    pub config_type: ::core::option::Option<transport_socket::ConfigType>,
}
/// Nested message and enum types in `TransportSocket`.
pub mod transport_socket {
    /// Implementation specific configuration which depends on the implementation being instantiated.
    /// See the supported transport socket implementations for further documentation.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum ConfigType {
        #[prost(message, tag = "3")]
        TypedConfig(::prost_types::Any),
    }
}
/// Runtime derived FractionalPercent with defaults for when the numerator or denominator is not
/// specified via a runtime key.
///
/// .. note::
///
///   Parsing of the runtime key's data is implemented such that it may be represented as a
///   :ref:`FractionalPercent <envoy_v3_api_msg_type.v3.FractionalPercent>` proto represented as JSON/YAML
///   and may also be represented as an integer with the assumption that the value is an integral
///   percentage out of 100. For instance, a runtime key lookup returning the value "42" would parse
///   as a ``FractionalPercent`` whose numerator is 42 and denominator is HUNDRED.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RuntimeFractionalPercent {
    /// Default value if the runtime value's for the numerator/denominator keys are not available.
    #[prost(message, optional, tag = "1")]
    pub default_value: ::core::option::Option<crate::envoy_type_v3::FractionalPercent>,
    /// Runtime key for a YAML representation of a FractionalPercent.
    #[prost(string, tag = "2")]
    pub runtime_key: ::prost::alloc::string::String,
}
/// Identifies a specific ControlPlane instance that Envoy is connected to.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ControlPlane {
    /// An opaque control plane identifier that uniquely identifies an instance
    /// of control plane. This can be used to identify which control plane instance,
    /// the Envoy is connected to.
    #[prost(string, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
}
// [#protodoc-title: Common types]

/// Envoy supports :ref:`upstream priority routing
/// <arch_overview_http_routing_priority>` both at the route and the virtual
/// cluster level. The current priority implementation uses different connection
/// pool and circuit breaking settings for each priority level. This means that
/// even for HTTP/2 requests, two physical connections will be used to an
/// upstream host. In the future Envoy will likely support true HTTP/2 priority
/// over a single upstream connection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum RoutingPriority {
    Default = 0,
    High = 1,
}
/// HTTP request method.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum RequestMethod {
    MethodUnspecified = 0,
    Get = 1,
    Head = 2,
    Post = 3,
    Put = 4,
    Delete = 5,
    Connect = 6,
    Options = 7,
    Trace = 8,
    Patch = 9,
}
/// Identifies the direction of the traffic relative to the local Envoy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum TrafficDirection {
    /// Default option is unspecified.
    Unspecified = 0,
    /// The transport is used for incoming traffic.
    Inbound = 1,
    /// The transport is used for outgoing traffic.
    Outbound = 2,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeaderMap {
    /// Repeated field for the headers.
    #[prost(message, repeated, tag = "1")]
    pub headers: ::prost::alloc::vec::Vec<HeaderValue>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeaderValueOption {
    #[prost(enumeration = "header_value_option::HeaderAppendAction", tag = "3")]
    pub append_action: i32,

    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<HeaderValue>,

    #[prost(message, optional, tag = "2")]
    pub append: ::core::option::Option<bool>,

    #[prost(bool, tag = "4")]
    pub keep_empty_value: bool,
}

pub mod header_value_option {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum HeaderAppendAction {
        AppendIfExistsOrAdd = 0,
        AddIfAbsent = 1,
        OverwriteIfExistsOrAdd = 2,
        OverwriteIfExists = 3,
    }
}

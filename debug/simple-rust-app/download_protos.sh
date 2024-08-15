#!/bin/bash

# Set the base directory for proto files
BASE_DIR="proto"

# Create the necessary directory structure
mkdir -p $BASE_DIR/envoy/config/core/v3
mkdir -p $BASE_DIR/envoy/extensions/filters/http/ext_proc/v3
mkdir -p $BASE_DIR/envoy/service/ext_proc/v3
mkdir -p $BASE_DIR/envoy/type/v3
mkdir -p $BASE_DIR/envoy/annotations
mkdir -p $BASE_DIR/udpa/annotations
mkdir -p $BASE_DIR/xds/core/v3
mkdir -p $BASE_DIR/xds/annotations/v3
mkdir -p $BASE_DIR/validate

# Download files into the correct directories
curl -o $BASE_DIR/envoy/config/core/v3/base.proto https://raw.githubusercontent.com/envoyproxy/envoy/main/api/envoy/config/core/v3/base.proto
curl -o $BASE_DIR/envoy/config/core/v3/address.proto https://raw.githubusercontent.com/envoyproxy/envoy/main/api/envoy/config/core/v3/address.proto
curl -o $BASE_DIR/envoy/config/core/v3/backoff.proto https://raw.githubusercontent.com/envoyproxy/envoy/main/api/envoy/config/core/v3/backoff.proto
curl -o $BASE_DIR/envoy/config/core/v3/http_uri.proto https://raw.githubusercontent.com/envoyproxy/envoy/main/api/envoy/config/core/v3/http_uri.proto
curl -o $BASE_DIR/envoy/config/core/v3/extension.proto https://raw.githubusercontent.com/envoyproxy/envoy/main/api/envoy/config/core/v3/extension.proto
curl -o $BASE_DIR/envoy/config/core/v3/socket_option.proto https://raw.githubusercontent.com/envoyproxy/envoy/main/api/envoy/config/core/v3/socket_option.proto
curl -o $BASE_DIR/envoy/type/v3/percent.proto https://raw.githubusercontent.com/envoyproxy/envoy/main/api/envoy/type/v3/percent.proto
curl -o $BASE_DIR/envoy/type/v3/semantic_version.proto https://raw.githubusercontent.com/envoyproxy/envoy/main/api/envoy/type/v3/semantic_version.proto
curl -o $BASE_DIR/xds/core/v3/context_params.proto https://raw.githubusercontent.com/solo-io/gloo/main/projects/gloo/api/external/xds/core/v3/context_params.proto
curl -o $BASE_DIR/xds/annotations/v3/status.proto https://raw.githubusercontent.com/solo-io/gloo/main/projects/gloo/api/external/xds/annotations/v3/status.proto
curl -o $BASE_DIR/udpa/annotations/migrate.proto https://raw.githubusercontent.com/cncf/udpa/main/udpa/annotations/migrate.proto
curl -o $BASE_DIR/udpa/annotations/versioning.proto https://raw.githubusercontent.com/cncf/udpa/main/udpa/annotations/versioning.proto
curl -o $BASE_DIR/envoy/annotations/deprecation.proto https://raw.githubusercontent.com/envoyproxy/envoy/main/api/envoy/annotations/deprecation.proto
curl -o $BASE_DIR/envoy/type/v3/http_status.proto https://raw.githubusercontent.com/envoyproxy/envoy/main/api/envoy/type/v3/http_status.proto
curl -o $BASE_DIR/envoy/extensions/filters/http/ext_proc/v3/processing_mode.proto https://raw.githubusercontent.com/envoyproxy/envoy/main/api/envoy/extensions/filters/http/ext_proc/v3/processing_mode.proto
curl -o $BASE_DIR/validate/validate.proto https://raw.githubusercontent.com/envoyproxy/protoc-gen-validate/main/validate/validate.proto
curl -o $BASE_DIR/envoy/service/ext_proc/v3/external_processor.proto https://raw.githubusercontent.com/envoyproxy/envoy/main/api/envoy/service/ext_proc/v3/external_processor.proto

echo "Proto files have been downloaded into the appropriate directories."

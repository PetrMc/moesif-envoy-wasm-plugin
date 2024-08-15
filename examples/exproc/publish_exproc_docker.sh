#!/bin/bash -e

# Set default tag to 'latest' unless provided as an argument
TAG=${1:-latest}

# Determine build variant: debug or release
if [ "$2" == "debug" ]; then
    BUILD_VARIANT=debug
    BUILD_FLAGS=""
else
    BUILD_VARIANT=release
    BUILD_FLAGS="--release"
fi

# Docker image names
REPO=docker.io/moesif
TAG_ARTIFACT=$REPO/moesif-solo-exproc-plugin:$TAG

# Get the directory of this script to make sure we can run it from anywhere
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_DIR="$SCRIPT_DIR/../.."
SOURCE="$BASE_DIR/moesif-exproc"
OUTPUT="$SOURCE/target/$BUILD_VARIANT"

# Build the application in the builder Docker container
docker build \
 --build-arg USER_ID=$(id -u) --build-arg GROUP_ID=$(id -g) \
 -t moesif-solo-exproc-plugin-builder \
 -f $BASE_DIR/Cargo-exporc-build.dockerfile \
 $BASE_DIR

# Run the container to build the Rust project
docker run \
 -v $SOURCE:/usr/src/moesif-exproc \
 moesif-solo-exproc-plugin-builder \
 bash -c "cargo build $BUILD_FLAGS"

# Package the plugin into a Docker image for deployment
docker build \
 -t $TAG_ARTIFACT \
 -f $BASE_DIR/Cargo-exporc-build.dockerfile \
 $OUTPUT

# Uncomment the following lines to push the Docker image to the repository
# docker push $TAG_ARTIFACT

echo "Docker image $TAG_ARTIFACT built successfully."

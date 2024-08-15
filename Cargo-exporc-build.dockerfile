# Stage 1: Build the Rust application with musl
FROM rust:1.70.0 as builder

# Install musl tools
RUN rustup target add x86_64-unknown-linux-musl

# Set the working directory inside the container
WORKDIR /usr/src/moesif-exproc

# Copy the source code and Cargo files into the container
COPY moesif-exproc/Cargo.toml .
COPY moesif-exproc/src ./src

# Build the application in release mode with musl
RUN cargo build --release --target=x86_64-unknown-linux-musl

# Stage 2: Create a smaller final image with the statically linked binary
FROM debian:buster-slim

# Install necessary dependencies for the binary to run
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/moesif-exproc/target/x86_64-unknown-linux-musl/release/moesif_solo_exproc_plugin /usr/local/bin/moesif_solo_exproc_plugin

# Set the default command to run your application
CMD ["moesif_solo_exproc_plugin"]

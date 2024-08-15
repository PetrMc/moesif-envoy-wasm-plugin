# Stage 1: Build the Rust application
FROM rust:1.70.0 as builder

# Set the working directory inside the container
WORKDIR /usr/src/simple-rust-app

# Copy the source code and Cargo files into the container
COPY Cargo.toml .
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Stage 2: Create a smaller final image with the built binary
FROM debian:bullseye-slim

# Install necessary dependencies for the binary to run
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/simple-rust-app/target/release/simple-rust-app /usr/local/bin/simple-rust-app

# Set the default command to run your application
CMD ["simple-rust-app"]

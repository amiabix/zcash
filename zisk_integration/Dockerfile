# ZisK zkVM Runner Dockerfile
# Builds and runs the ZisK zkVM integration

FROM ubuntu:22.04

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive
ENV RUST_VERSION=1.82.0

# Install system dependencies
RUN apt-get update && \
    apt-get install -y \
        curl ca-certificates build-essential \
        python3 python3-pip \
        pkg-config libssl-dev libffi-dev \
        libgmp-dev libmpfr-dev libmpc-dev \
        nasm make && \
    rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain $RUST_VERSION
ENV PATH="/root/.cargo/bin:${PATH}"

# Set working directory
WORKDIR /app

# Copy only the demo source and Cargo.toml
COPY src/bin/simple_demo.rs ./src/bin/
COPY Cargo-demo.toml ./Cargo.toml

# Build the simple demo
RUN cargo build --release --bin simple_demo

# Create startup script
RUN echo '#!/bin/bash\n\
set -e\n\
\n\
echo "🌐 Starting Simple Zcash Validator..."\n\
echo "📡 This is a simplified version for testing Docker setup"\n\
echo "🔧 Running without ZisK dependencies for ARM64 compatibility"\n\
\n\
# Start the simple validator\n\
exec cargo run --release --bin simple_demo\n\
' > /usr/local/bin/start.sh && chmod +x /usr/local/bin/start.sh

# Expose RPC port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Start zkVM runner
CMD ["/usr/local/bin/start.sh"]

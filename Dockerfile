# Zcash Node with ZisK Integration Dockerfile
# Builds the modified Zcash node with zkVM integration

FROM ubuntu:22.04

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && \
    apt-get install -y \
        wget gnupg build-essential libtool autotools-dev automake pkg-config \
        libssl-dev libevent-dev bsdmainutils git curl unzip \
        libboost-system-dev libboost-filesystem-dev libboost-chrono-dev \
        libboost-test-dev libboost-thread-dev \
        libdb-dev libdb++-dev libminiupnpc-dev \
        libzmq3-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the entire project (including zisk_integration)
COPY . /zcash

# Set working directory
WORKDIR /zcash

# Build Zcash from source
RUN ./zcutil/build.sh -j$(nproc)

# Create Zcash configuration
RUN mkdir -p /root/.zcash && \
    echo "regtest=1" > /root/.zcash/zcash.conf && \
    echo "rpcuser=test" >> /root/.zcash/zcash.conf && \
    echo "rpcpassword=test" >> /root/.zcash/zcash.conf && \
    echo "rpcallowip=0.0.0.0/0" >> /root/.zcash/zcash.conf && \
    echo "rpcbind=0.0.0.0" >> /root/.zcash/zcash.conf && \
    echo "server=1" >> /root/.zcash/zcash.conf && \
    echo "daemon=0" >> /root/.zcash/zcash.conf && \
    echo "printtoconsole=1" >> /root/.zcash/zcash.conf && \
    echo "txindex=1" >> /root/.zcash/zcash.conf && \
    echo "listen=1" >> /root/.zcash/zcash.conf

# Expose ports
EXPOSE 8232 18232

# Health check
HEALTHCHECK --interval=10s --timeout=5s --start-period=30s --retries=5 \
    CMD /zcash/src/zcash-cli -regtest -rpcuser=test -rpcpassword=test getblockcount || exit 1

# Start Zcash node
CMD ["/zcash/src/zcashd", "-regtest", "-rpcuser=test", "-rpcpassword=test", "-rpcallowip=0.0.0.0/0", "-rpcbind=0.0.0.0:8232", "-txindex=1", "-listen=1", "-printtoconsole"]

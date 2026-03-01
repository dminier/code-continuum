# Use official Rust image with latest stable version
FROM mcr.microsoft.com/devcontainers/rust:2.0.3-1

RUN update-ca-certificates
# Set working directory
WORKDIR /workspace
 
# Build tools needed for tree-sitter wasm (wasmtime) build scripts
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        cmake \
        build-essential \
        pkg-config \
        python3 \
        python3-venv \
        python3-pip \
        curl \
    && rm -rf /var/lib/apt/lists/*



# Install uv (provides uvx) as non-root user and expose it on PATH
USER vscode
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
RUN curl -fsSL https://claude.ai/install.sh | bash
ENV PATH="/home/vscode/.local/bin:${PATH}"
ENV UV_NATIVE_TLS=true

# Port MCP HTTP (mode --mcp)
EXPOSE 8001

# Default command
CMD ["/bin/bash"]

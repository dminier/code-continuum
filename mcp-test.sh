#!/usr/bin/env bash

set -euo pipefail

# URLs from .mcp.json
NEO4J_URL="http://mcp-neo4j:8000/api/mcp/"
CODE_CONTINUUM_URL="http://localhost:8001/api/mcp/"

# Function to test MCP endpoint
test_mcp_endpoint() {
    local url=$1
    local name=$2

    echo "=================================================="
    echo "🚀 Testing: $name"
    echo "URL: $url"
    echo "Date: $(date -Iseconds)"
    echo "=================================================="
    echo

    curl -X POST "$url" \
      -H "Content-Type: application/json" \
      -H "Accept: application/json, text/event-stream" \
      -d '{
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
      }' \
      --include \
      --verbose \
      --connect-timeout 5 \
      --max-time 20 \
      --write-out "\n
================ CURL METRICS ================
HTTP Status Code: %{http_code}
Remote IP: %{remote_ip}
Remote Port: %{remote_port}
Total Time: %{time_total}s
DNS Lookup Time: %{time_namelookup}s
Connect Time: %{time_connect}s
TLS Handshake: %{time_appconnect}s
Start Transfer: %{time_starttransfer}s
Download Size: %{size_download} bytes
Upload Size: %{size_upload} bytes
Speed Download: %{speed_download} bytes/sec
==============================================\n" || true

    echo
}

# Test both endpoints
test_mcp_endpoint "$NEO4J_URL" "Neo4j MCP Server"
test_mcp_endpoint "$CODE_CONTINUUM_URL" "Code-Continuum MCP Server"

echo "✅ Tests completed"
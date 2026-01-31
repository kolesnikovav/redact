#!/usr/bin/env bash
#
# Benchmark Redact vs Microsoft Presidio using Oha
#
# Uses Oha (https://github.com/hatoo/oha) for objective HTTP benchmarking
# with proper statistical analysis and histograms.
#
# Requirements:
#   - oha (cargo install oha)
#   - Docker (for Presidio)
#   - Rust toolchain (for Redact)
#   - jq
#
# Usage:
#   ./scripts/benchmark-comparison.sh
#   ./scripts/benchmark-comparison.sh --requests 500
#   ./scripts/benchmark-comparison.sh --duration 30s
#

set -euo pipefail

# Defaults
REQUESTS=200
CONCURRENCY=1
PRESIDIO_PORT=5001
REDACT_PORT=8080
RESULTS_DIR="docs/benchmarks"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Parse args
while [[ $# -gt 0 ]]; do
    case $1 in
        -n|--requests) REQUESTS="$2"; shift 2 ;;
        -c|--concurrency) CONCURRENCY="$2"; shift 2 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

# Test payload
TEST_TEXT="Contact john.doe@example.com or call (555) 123-4567. SSN: 123-45-6789."
PAYLOAD=$(jq -n --arg t "$TEST_TEXT" '{"text":$t,"language":"en"}')

info() { echo "[*] $1"; }
ok() { echo "[✓] $1"; }
err() { echo "[!] $1" >&2; exit 1; }

cleanup() {
    pkill -f "redact-api" 2>/dev/null || true
    docker rm -f presidio-bench 2>/dev/null || true
}
trap cleanup EXIT

# Check dependencies
command -v oha >/dev/null || err "oha not found. Install: cargo install oha"
command -v docker >/dev/null || err "docker not found"
command -v jq >/dev/null || err "jq not found"

# Build Redact
info "Building Redact (release)..."
cargo build --release --bin redact-api 2>/dev/null
ok "Built"

# Start services
info "Starting Presidio (Docker)..."
docker rm -f presidio-bench 2>/dev/null || true
docker run -d --name presidio-bench -p ${PRESIDIO_PORT}:3000 \
    mcr.microsoft.com/presidio-analyzer:latest >/dev/null

info "Starting Redact..."
./target/release/redact-api &

# Wait for services
info "Waiting for services..."
for i in {1..30}; do
    p=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:${PRESIDIO_PORT}/health" 2>/dev/null || echo "0")
    r=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:${REDACT_PORT}/health" 2>/dev/null || echo "0")
    [[ "$p" == "200" && "$r" == "200" ]] && break
    sleep 1
done
ok "Services ready"

mkdir -p "$RESULTS_DIR"

echo ""
echo "========================================"
echo "  HTTP Benchmark (oha)"
echo "  Requests: $REQUESTS | Concurrency: $CONCURRENCY"
echo "========================================"
echo ""

# Benchmark Redact
info "Benchmarking Redact..."
echo ""
oha -n "$REQUESTS" -c "$CONCURRENCY" \
    -m POST \
    -H "Content-Type: application/json" \
    -d "$PAYLOAD" \
    "http://localhost:${REDACT_PORT}/api/v1/analyze" \
    --json > "${RESULTS_DIR}/redact-${TIMESTAMP}.json"

oha -n "$REQUESTS" -c "$CONCURRENCY" \
    -m POST \
    -H "Content-Type: application/json" \
    -d "$PAYLOAD" \
    "http://localhost:${REDACT_PORT}/api/v1/analyze"

echo ""

# Benchmark Presidio
info "Benchmarking Presidio..."
echo ""
oha -n "$REQUESTS" -c "$CONCURRENCY" \
    -m POST \
    -H "Content-Type: application/json" \
    -d "$PAYLOAD" \
    "http://localhost:${PRESIDIO_PORT}/analyze" \
    --json > "${RESULTS_DIR}/presidio-${TIMESTAMP}.json"

oha -n "$REQUESTS" -c "$CONCURRENCY" \
    -m POST \
    -H "Content-Type: application/json" \
    -d "$PAYLOAD" \
    "http://localhost:${PRESIDIO_PORT}/analyze"

# Generate comparison report
REDACT_P50=$(jq -r '.latencyPercentiles.p50' "${RESULTS_DIR}/redact-${TIMESTAMP}.json")
REDACT_P99=$(jq -r '.latencyPercentiles.p99' "${RESULTS_DIR}/redact-${TIMESTAMP}.json")
REDACT_RPS=$(jq -r '.summary.requestsPerSec' "${RESULTS_DIR}/redact-${TIMESTAMP}.json")

PRESIDIO_P50=$(jq -r '.latencyPercentiles.p50' "${RESULTS_DIR}/presidio-${TIMESTAMP}.json")
PRESIDIO_P99=$(jq -r '.latencyPercentiles.p99' "${RESULTS_DIR}/presidio-${TIMESTAMP}.json")
PRESIDIO_RPS=$(jq -r '.summary.requestsPerSec' "${RESULTS_DIR}/presidio-${TIMESTAMP}.json")

cat > "${RESULTS_DIR}/results-${TIMESTAMP}.md" << EOF
# Benchmark Results

**Date:** $(date +%Y-%m-%d)  
**Tool:** [oha](https://github.com/hatoo/oha)  
**Requests:** $REQUESTS | **Concurrency:** $CONCURRENCY  
**Platform:** $(uname -s) $(uname -m)

## Summary

| Metric | Redact | Presidio | Speedup |
|--------|--------|----------|---------|
| p50 Latency | ${REDACT_P50}s | ${PRESIDIO_P50}s | $(echo "scale=1; $PRESIDIO_P50 / $REDACT_P50" | bc 2>/dev/null || echo "-")x |
| p99 Latency | ${REDACT_P99}s | ${PRESIDIO_P99}s | $(echo "scale=1; $PRESIDIO_P99 / $REDACT_P99" | bc 2>/dev/null || echo "-")x |
| Requests/sec | ${REDACT_RPS} | ${PRESIDIO_RPS} | $(echo "scale=1; $REDACT_RPS / $PRESIDIO_RPS" | bc 2>/dev/null || echo "-")x |

## Test Payload

\`\`\`json
$PAYLOAD
\`\`\`

## Raw Data

- [redact-${TIMESTAMP}.json](redact-${TIMESTAMP}.json)
- [presidio-${TIMESTAMP}.json](presidio-${TIMESTAMP}.json)

## Reproduce

\`\`\`bash
./scripts/benchmark-comparison.sh --requests $REQUESTS --concurrency $CONCURRENCY
\`\`\`
EOF

echo ""
echo "========================================"
echo "  Summary"
echo "========================================"
echo ""
echo "Redact:   p50=${REDACT_P50}s  p99=${REDACT_P99}s  rps=${REDACT_RPS}"
echo "Presidio: p50=${PRESIDIO_P50}s  p99=${PRESIDIO_P99}s  rps=${PRESIDIO_RPS}"
echo ""
ok "Results saved to ${RESULTS_DIR}/results-${TIMESTAMP}.md"

#!/usr/bin/env bash
#
# Benchmark Redact vs Microsoft Presidio (REST API comparison)
#
# This is the fairest comparison - both tools running as HTTP services
# with identical inputs and measuring end-to-end latency.
#
# Requirements:
#   - Docker (for Presidio)
#   - Rust toolchain (for Redact)
#   - curl, jq, bc
#
# Usage:
#   ./scripts/benchmark-comparison.sh
#   ./scripts/benchmark-comparison.sh --iterations 100
#

set -euo pipefail

ITERATIONS="${1:-50}"
PRESIDIO_PORT=5001
REDACT_PORT=8080
RESULTS_DIR="docs/benchmarks"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Test texts - representative PII samples
TEXTS=(
    "Contact john.doe@example.com or call (555) 123-4567."
    "SSN: 123-45-6789, Card: 4532-1234-5678-9010."
    "Patient MRN-12345678, DOB 1990-05-15, NYC 10001."
)

info() { echo "[*] $1"; }
ok() { echo "[✓] $1"; }
err() { echo "[!] $1" >&2; }

cleanup() {
    pkill -f "redact-api" 2>/dev/null || true
    docker rm -f presidio-bench 2>/dev/null || true
}
trap cleanup EXIT

# Build Redact
info "Building Redact..."
cargo build --release --bin redact-api 2>/dev/null
ok "Redact built"

# Start Presidio
info "Starting Presidio (Docker)..."
docker rm -f presidio-bench 2>/dev/null || true
docker run -d --name presidio-bench -p ${PRESIDIO_PORT}:3000 \
    mcr.microsoft.com/presidio-analyzer:latest >/dev/null

# Start Redact
info "Starting Redact..."
./target/release/redact-api &
REDACT_PID=$!

# Wait for services
info "Waiting for services..."
for i in {1..30}; do
    presidio_up=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:${PRESIDIO_PORT}/health" 2>/dev/null || echo "000")
    redact_up=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:${REDACT_PORT}/health" 2>/dev/null || echo "000")
    [[ "$presidio_up" == "200" && "$redact_up" == "200" ]] && break
    sleep 1
done
ok "Services ready"

# Benchmark function
bench() {
    local url="$1"
    local payload="$2"
    local total=0
    
    for ((i=0; i<ITERATIONS; i++)); do
        ms=$(curl -s -o /dev/null -w "%{time_total}" -X POST "$url" \
            -H "Content-Type: application/json" -d "$payload" 2>/dev/null)
        total=$(echo "$total + $ms * 1000" | bc)
    done
    
    echo "scale=1; $total / $ITERATIONS" | bc
}

# Run benchmarks
echo ""
echo "========================================"
echo "  REST API Benchmark ($ITERATIONS iterations)"
echo "========================================"
echo ""

mkdir -p "$RESULTS_DIR"
RESULTS_FILE="${RESULTS_DIR}/results-${TIMESTAMP}.md"

cat > "$RESULTS_FILE" << EOF
# Benchmark Results

**Date:** $(date +%Y-%m-%d)  
**Iterations:** $ITERATIONS  
**Platform:** $(uname -s) $(uname -m)

## REST API Latency (ms)

| Test | Redact | Presidio | Speedup |
|------|--------|----------|---------|
EOF

test_num=1
for text in "${TEXTS[@]}"; do
    redact_payload=$(jq -n --arg t "$text" '{"text":$t,"language":"en"}')
    presidio_payload=$(jq -n --arg t "$text" '{"text":$t,"language":"en"}')
    
    redact_ms=$(bench "http://localhost:${REDACT_PORT}/api/v1/analyze" "$redact_payload")
    presidio_ms=$(bench "http://localhost:${PRESIDIO_PORT}/analyze" "$presidio_payload")
    
    speedup=$(echo "scale=1; $presidio_ms / $redact_ms" | bc 2>/dev/null || echo "-")
    
    printf "Test %d: Redact=%.1fms  Presidio=%.1fms  (%.1fx faster)\n" \
        "$test_num" "$redact_ms" "$presidio_ms" "$speedup"
    
    echo "| $test_num | $redact_ms | $presidio_ms | ${speedup}x |" >> "$RESULTS_FILE"
    ((test_num++))
done

cat >> "$RESULTS_FILE" << 'EOF'

## Test Cases

1. Email + phone number
2. SSN + credit card
3. Healthcare PII (MRN, DOB, ZIP)

## Methodology

Both services run locally - Presidio in Docker, Redact as native binary.
Measurements are end-to-end HTTP latency (includes serialization overhead).
EOF

echo ""
ok "Results saved to $RESULTS_FILE"

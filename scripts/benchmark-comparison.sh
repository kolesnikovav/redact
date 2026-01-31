#!/usr/bin/env bash
#
# Benchmark Redact vs Microsoft Presidio
#
# This script compares PII detection performance between Redact (Rust) and
# Presidio (Python via Docker). Results are saved to docs/benchmarks/.
#
# Requirements:
#   - Docker (for Presidio)
#   - Rust toolchain (for Redact)
#   - curl, jq
#
# Usage:
#   ./scripts/benchmark-comparison.sh
#   ./scripts/benchmark-comparison.sh --iterations 100
#   ./scripts/benchmark-comparison.sh --skip-presidio  # Redact-only benchmarks
#

set -euo pipefail

# Configuration
ITERATIONS="${ITERATIONS:-50}"
PRESIDIO_IMAGE="mcr.microsoft.com/presidio-analyzer:latest"
PRESIDIO_PORT=5001
REDACT_PORT=8080
RESULTS_DIR="docs/benchmarks"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
RESULTS_FILE="${RESULTS_DIR}/results-${TIMESTAMP}.md"

# Test data - representative PII samples
declare -a TEST_TEXTS=(
    "Contact John Doe at john.doe@example.com or call (555) 123-4567."
    "My SSN is 123-45-6789 and my credit card is 4532-1234-5678-9010."
    "Send payment to IBAN GB82WEST12345698765432. Reference: invoice-2024."
    "Patient ID: MRN-12345678, DOB: 1990-05-15, Address: 123 Main St, NYC 10001."
    "Server IP: 192.168.1.100, MAC: 00:1A:2B:3C:4D:5E, API key: sk-abc123xyz."
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Parse arguments
SKIP_PRESIDIO=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --iterations)
            ITERATIONS="$2"
            shift 2
            ;;
        --skip-presidio)
            SKIP_PRESIDIO=true
            shift
            ;;
        --help)
            echo "Usage: $0 [--iterations N] [--skip-presidio]"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    if ! command -v cargo &> /dev/null; then
        log_error "cargo not found. Install Rust: https://rustup.rs"
        exit 1
    fi
    
    if ! command -v jq &> /dev/null; then
        log_error "jq not found. Install with: brew install jq"
        exit 1
    fi
    
    if [[ "$SKIP_PRESIDIO" == "false" ]]; then
        if ! command -v docker &> /dev/null; then
            log_error "docker not found. Install Docker or use --skip-presidio"
            exit 1
        fi
    fi
    
    log_success "Dependencies OK"
}

# Build Redact
build_redact() {
    log_info "Building Redact (release mode)..."
    cargo build --release --bin redact-api --bin redact-cli 2>/dev/null
    log_success "Redact built"
}

# Start Presidio container
start_presidio() {
    if [[ "$SKIP_PRESIDIO" == "true" ]]; then
        return
    fi
    
    log_info "Starting Presidio container..."
    
    # Stop existing container if running
    docker rm -f presidio-benchmark 2>/dev/null || true
    
    # Pull latest image
    docker pull "$PRESIDIO_IMAGE" 2>/dev/null
    
    # Start container
    docker run -d --name presidio-benchmark \
        -p "${PRESIDIO_PORT}:3000" \
        "$PRESIDIO_IMAGE" > /dev/null
    
    # Wait for startup
    log_info "Waiting for Presidio to start..."
    for i in {1..30}; do
        if curl -s "http://localhost:${PRESIDIO_PORT}/health" > /dev/null 2>&1; then
            log_success "Presidio ready"
            return
        fi
        sleep 1
    done
    
    log_error "Presidio failed to start"
    exit 1
}

# Start Redact server
start_redact() {
    log_info "Starting Redact server..."
    
    # Kill existing process if running
    pkill -f "redact-api" 2>/dev/null || true
    sleep 1
    
    # Start server in background
    ./target/release/redact-api &
    REDACT_PID=$!
    
    # Wait for startup
    for i in {1..10}; do
        if curl -s "http://localhost:${REDACT_PORT}/health" > /dev/null 2>&1; then
            log_success "Redact ready (PID: $REDACT_PID)"
            return
        fi
        sleep 0.5
    done
    
    log_error "Redact failed to start"
    exit 1
}

# Benchmark function - measures time for N iterations
benchmark_api() {
    local name="$1"
    local url="$2"
    local payload="$3"
    local iterations="$4"
    
    local total_ms=0
    local success=0
    local failed=0
    
    for ((i=1; i<=iterations; i++)); do
        local start_ns=$(date +%s%N)
        
        local response
        response=$(curl -s -w "\n%{http_code}" -X POST "$url" \
            -H "Content-Type: application/json" \
            -d "$payload" 2>/dev/null)
        
        local end_ns=$(date +%s%N)
        local http_code=$(echo "$response" | tail -1)
        
        if [[ "$http_code" == "200" ]]; then
            local elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
            total_ms=$((total_ms + elapsed_ms))
            ((success++))
        else
            ((failed++))
        fi
    done
    
    if [[ $success -gt 0 ]]; then
        local avg_ms=$((total_ms / success))
        echo "$avg_ms"
    else
        echo "-1"
    fi
}

# Benchmark CLI (Redact only)
benchmark_cli() {
    local text="$1"
    local iterations="$2"
    
    local total_ms=0
    
    for ((i=1; i<=iterations; i++)); do
        local start_ns=$(date +%s%N)
        ./target/release/redact-cli analyze "$text" > /dev/null 2>&1
        local end_ns=$(date +%s%N)
        local elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
        total_ms=$((total_ms + elapsed_ms))
    done
    
    echo $((total_ms / iterations))
}

# Run Criterion benchmarks
run_criterion_benchmarks() {
    log_info "Running Criterion benchmarks..."
    cargo bench --package redact-core 2>/dev/null | tee "${RESULTS_DIR}/criterion-${TIMESTAMP}.txt"
    log_success "Criterion benchmarks complete"
}

# Cleanup
cleanup() {
    log_info "Cleaning up..."
    pkill -f "redact-api" 2>/dev/null || true
    docker rm -f presidio-benchmark 2>/dev/null || true
}

trap cleanup EXIT

# Generate results markdown
generate_report() {
    local redact_results=("$@")
    
    mkdir -p "$RESULTS_DIR"
    
    cat > "$RESULTS_FILE" << EOF
# Benchmark Results

**Date:** $(date +%Y-%m-%d)  
**Redact Version:** $(cargo metadata --format-version 1 2>/dev/null | jq -r '.packages[] | select(.name == "redact-core") | .version')  
**Presidio Version:** latest (Docker)  
**Iterations:** $ITERATIONS  
**Platform:** $(uname -s) $(uname -m)

## Summary

| Metric | Redact (Rust) | Presidio (Python) | Speedup |
|--------|---------------|-------------------|---------|
EOF
}

# Main execution
main() {
    echo ""
    echo "========================================"
    echo "  Redact vs Presidio Benchmark"
    echo "========================================"
    echo ""
    
    check_dependencies
    build_redact
    
    mkdir -p "$RESULTS_DIR"
    
    # Initialize results file
    cat > "$RESULTS_FILE" << EOF
# Benchmark Results

**Date:** $(date +%Y-%m-%d)  
**Redact Version:** $(cargo metadata --format-version 1 2>/dev/null | jq -r '.packages[] | select(.name == "redact-core") | .version' 2>/dev/null || echo "0.1.0")  
**Presidio Version:** latest (Docker)  
**Iterations per test:** $ITERATIONS  
**Platform:** $(uname -s) $(uname -m)

## API Latency Comparison

| Test Case | Redact (ms) | Presidio (ms) | Speedup |
|-----------|-------------|---------------|---------|
EOF

    # Start servers
    if [[ "$SKIP_PRESIDIO" == "false" ]]; then
        start_presidio
    fi
    start_redact
    
    sleep 2  # Allow servers to stabilize
    
    log_info "Running API benchmarks ($ITERATIONS iterations each)..."
    echo ""
    
    test_num=1
    for text in "${TEST_TEXTS[@]}"; do
        # Truncate text for display
        display_text="${text:0:50}..."
        
        # Redact payload
        redact_payload=$(jq -n --arg text "$text" '{"text": $text, "language": "en"}')
        
        # Presidio payload
        presidio_payload=$(jq -n --arg text "$text" '{"text": $text, "language": "en"}')
        
        # Benchmark Redact
        redact_ms=$(benchmark_api "Redact" \
            "http://localhost:${REDACT_PORT}/api/v1/analyze" \
            "$redact_payload" \
            "$ITERATIONS")
        
        # Benchmark Presidio
        if [[ "$SKIP_PRESIDIO" == "false" ]]; then
            presidio_ms=$(benchmark_api "Presidio" \
                "http://localhost:${PRESIDIO_PORT}/analyze" \
                "$presidio_payload" \
                "$ITERATIONS")
        else
            presidio_ms="-"
        fi
        
        # Calculate speedup
        if [[ "$presidio_ms" != "-" && "$presidio_ms" -gt 0 && "$redact_ms" -gt 0 ]]; then
            speedup=$(echo "scale=1; $presidio_ms / $redact_ms" | bc)
            speedup_display="${speedup}x"
        else
            speedup_display="-"
        fi
        
        # Output results
        printf "  Test %d: Redact=%dms" "$test_num" "$redact_ms"
        if [[ "$SKIP_PRESIDIO" == "false" ]]; then
            printf ", Presidio=%dms, Speedup=%s" "$presidio_ms" "$speedup_display"
        fi
        echo ""
        
        # Add to results file
        echo "| Test $test_num | $redact_ms | $presidio_ms | $speedup_display |" >> "$RESULTS_FILE"
        
        ((test_num++))
    done
    
    # Add CLI benchmarks
    cat >> "$RESULTS_FILE" << EOF

## CLI Performance (Redact only)

| Test Case | Latency (ms) |
|-----------|--------------|
EOF

    log_info "Running CLI benchmarks..."
    test_num=1
    for text in "${TEST_TEXTS[@]}"; do
        cli_ms=$(benchmark_cli "$text" 10)
        echo "| Test $test_num | $cli_ms |" >> "$RESULTS_FILE"
        printf "  CLI Test %d: %dms\n" "$test_num" "$cli_ms"
        ((test_num++))
    done
    
    # Add notes
    cat >> "$RESULTS_FILE" << EOF

## Test Cases

1. Email + Phone detection
2. SSN + Credit Card detection
3. IBAN + Reference detection
4. Healthcare PII (MRN, DOB, Address)
5. Technical identifiers (IP, MAC, API key)

## Methodology

- Both services run locally (Docker for Presidio, native for Redact)
- Each test runs $ITERATIONS iterations to calculate average latency
- Cold start time excluded (servers warmed up before benchmarking)
- Results include HTTP overhead for API benchmarks

## Running These Benchmarks

\`\`\`bash
# Full comparison (requires Docker)
./scripts/benchmark-comparison.sh

# Redact-only benchmarks
./scripts/benchmark-comparison.sh --skip-presidio

# More iterations for statistical significance
./scripts/benchmark-comparison.sh --iterations 100

# Criterion micro-benchmarks
cargo bench --package redact-core
\`\`\`
EOF

    echo ""
    log_success "Benchmark complete!"
    echo ""
    echo "Results saved to: $RESULTS_FILE"
    echo ""
    
    # Show summary
    cat "$RESULTS_FILE"
}

main "$@"

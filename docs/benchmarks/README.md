# Benchmarks

Performance benchmarks comparing Redact against Microsoft Presidio.

## Quick Start

```bash
# Full comparison (requires Docker for Presidio)
./scripts/benchmark-comparison.sh

# Redact-only benchmarks (no Docker required)
./scripts/benchmark-comparison.sh --skip-presidio

# Criterion micro-benchmarks
cargo bench --package redact-core
```

## Benchmark Types

### 1. Comparison Benchmarks (`benchmark-comparison.sh`)

Compares Redact API latency against Presidio API latency using identical test inputs.

**Requirements:**
- Docker (for Presidio) - or use `--skip-presidio`
- Rust toolchain
- `curl`, `jq`

**Options:**
```bash
--iterations N    # Number of iterations per test (default: 50)
--skip-presidio   # Skip Presidio benchmarks (Redact-only)
```

### 2. Criterion Micro-Benchmarks

Detailed Rust micro-benchmarks for Redact internals:

```bash
cargo bench --package redact-core
```

Benchmarks include:
- Single entity detection (email, SSN, phone, etc.)
- Multiple entity detection
- Text length scaling (100-5000 chars)
- Anonymization strategies (replace, mask, hash)
- Cold vs warm start performance
- Batch throughput

## Results

Benchmark results are saved to `docs/benchmarks/results-YYYYMMDD-HHMMSS.md`.

See the latest results in this directory, or run the benchmarks yourself for your specific hardware.

## Methodology

### Test Data

Representative PII samples covering:
1. Contact information (email, phone)
2. Government IDs (SSN, credit card)
3. Financial (IBAN)
4. Healthcare (MRN, DOB, address)
5. Technical (IP, MAC, API keys)

### Measurement

- **API benchmarks**: End-to-end HTTP latency including serialization
- **CLI benchmarks**: Process spawn + execution time
- **Criterion benchmarks**: Pure function execution time (no I/O)

### Environment

- Both services run locally
- Presidio runs in Docker container
- Redact runs as native binary (release build)
- Servers warmed up before measurement
- Multiple iterations for statistical significance

## Expected Results

Based on architecture differences:

| Metric | Redact | Presidio | Reason |
|--------|--------|----------|--------|
| Latency | ~2-5ms | ~20-100ms | Native vs interpreted |
| Memory | ~20-50MB | ~300MB+ | No Python runtime |
| Startup | ~50ms | ~2-5s | No model loading overhead |

Actual results vary by hardware and workload.

# Benchmarks

Performance benchmarks comparing Redact against Microsoft Presidio.

## Why REST API Comparison?

The **REST API benchmark is the fairest comparison** because:

1. **Real-world deployment** - Both tools are typically deployed as HTTP services
2. **Apples-to-apples** - Same protocol, same serialization overhead
3. **Language-agnostic** - Removes Python vs Rust runtime comparison noise
4. **User-relevant** - This is how most teams would actually use either tool

Other benchmark types are less meaningful:
- **Library comparison** - Different languages (Rust vs Python), not comparable
- **CLI comparison** - Presidio doesn't have a CLI equivalent

## Quick Start

```bash
# REST API comparison (requires Docker)
./scripts/benchmark-comparison.sh

# More iterations for statistical significance
./scripts/benchmark-comparison.sh 100

# Criterion micro-benchmarks (Redact internals)
cargo bench --package redact-core
```

## Requirements

- Docker (for Presidio container)
- Rust toolchain
- `curl`, `jq`, `bc`

## What Gets Measured

### REST API Benchmark

| Test Case | Description |
|-----------|-------------|
| 1 | Email + phone number |
| 2 | SSN + credit card |
| 3 | Healthcare PII (MRN, DOB, ZIP) |

Both services receive identical JSON payloads. Measurement is end-to-end HTTP latency.

### Criterion Micro-Benchmarks

Detailed Rust-level benchmarks for Redact internals:

- Single entity detection (email, SSN, phone, etc.)
- Multiple entity detection
- Text length scaling (100-5000 chars)
- Anonymization strategies (replace, mask, hash)
- Cold vs warm start performance
- Batch throughput

## Results

Results are saved to `docs/benchmarks/results-YYYYMMDD-HHMMSS.md`.

## Expected Performance

| Metric | Redact | Presidio | Why |
|--------|--------|----------|-----|
| API Latency | ~2-5ms | ~20-100ms | Native binary vs Python interpreter |
| Memory | ~20-50MB | ~300MB+ | No Python runtime overhead |
| Startup | ~50ms | ~2-5s | No model loading delay |

Actual results vary by hardware.

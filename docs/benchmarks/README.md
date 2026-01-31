# Benchmarks

Performance benchmarks comparing Redact against Microsoft Presidio.

## Tool

We use [**oha**](https://github.com/hatoo/oha) - a modern HTTP load testing tool written in Rust that provides:

- Proper statistical analysis (percentiles: p50, p90, p99)
- Latency histograms
- JSON output for programmatic analysis
- Consistent, reproducible measurements

## Why REST API Comparison?

The **REST API benchmark is the fairest comparison** because:

1. **Real-world deployment** - Both tools are typically deployed as HTTP services
2. **Apples-to-apples** - Same protocol, same serialization overhead
3. **Language-agnostic** - Removes Python vs Rust runtime comparison noise
4. **User-relevant** - This is how most teams would actually use either tool

## Quick Start

```bash
# Install oha
cargo install oha

# Run benchmark (requires Docker for Presidio)
./scripts/benchmark-comparison.sh

# Custom parameters
./scripts/benchmark-comparison.sh --requests 500 --concurrency 4
```

## Requirements

- [oha](https://github.com/hatoo/oha) (`cargo install oha`)
- Docker (for Presidio container)
- Rust toolchain
- `jq`

## Output

The benchmark produces:

1. **Console output** - oha's histogram and statistics for each service
2. **JSON files** - Raw data (`redact-*.json`, `presidio-*.json`)
3. **Markdown report** - Summary comparison (`results-*.md`)

## Criterion Micro-Benchmarks

For Redact-internal performance (no HTTP overhead):

```bash
cargo bench --package redact-core
```

Benchmarks include:
- Single entity detection (email, SSN, phone, etc.)
- Multiple entity detection
- Text length scaling (100-5000 chars)
- Anonymization strategies (replace, mask, hash)
- Cold vs warm start performance

## Latest Results (2026-01-31)

| Metric | Redact (Rust) | Presidio (Python) | Speedup |
|--------|---------------|-------------------|---------|
| p50 Latency | 0.20 ms | 6.84 ms | **34x** |
| p99 Latency | 0.78 ms | 12.38 ms | **16x** |
| Avg Latency | 0.23 ms | 7.16 ms | **31x** |
| Requests/sec | 4,321 | 139 | **31x** |

**Environment:** Darwin arm64, Docker containers, 500 requests

- **Redact:** Distroless container (37MB image)
- **Presidio:** Official Microsoft container

Results vary by hardware. Run `./scripts/benchmark-comparison.sh` to benchmark on your system.

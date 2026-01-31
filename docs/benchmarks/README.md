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

## Expected Performance

| Metric | Redact | Presidio | Why |
|--------|--------|----------|-----|
| p50 Latency | ~1-3ms | ~15-50ms | Native binary vs Python |
| p99 Latency | ~5-10ms | ~50-150ms | No GC pauses |
| Requests/sec | ~300-500 | ~20-50 | Lower overhead |

Actual results vary by hardware and payload.

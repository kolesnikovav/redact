# Redact

<div align="center">

[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Tests](https://github.com/censgate/redact/workflows/CI/badge.svg)](https://github.com/censgate/redact/actions)
[![Crates.io](https://img.shields.io/crates/v/redact-core.svg)](https://crates.io/crates/redact-core)

**High-performance PII detection and anonymization engine**

A production-ready, Rust-based solution designed as a drop-in replacement for Microsoft Presidio.

[Quick Start](#quick-start) · [Documentation](#documentation) · [Examples](#examples) · [Contributing](#contributing)

</div>

---

## Features

- **High Performance** — 10-100x faster than Python-based solutions with sub-millisecond inference
- **Memory Safe** — Rust's borrow checker eliminates entire classes of security vulnerabilities
- **Production Ready** — 36+ pattern-based entity types plus transformer-based NER
- **Multi-Platform** — Native server, WebAssembly (WASM), and CLI support
- **ML-Powered** — Full ONNX Runtime integration for transformer models (BERT, RoBERTa, DistilBERT)
- **Lightweight** — ~20-50MB memory footprint vs ~300MB for Presidio
- **Extensible** — Plugin architecture for custom recognizers and anonymization strategies

## Quick Start

### Install the CLI

```bash
cargo install redact-cli
redact --version
```

### Analyze Text for PII

```bash
redact analyze "Contact John Doe at john@example.com or call (555) 123-4567"
```

Output:

```
Detected 2 PII entities:

  EmailAddress at 21..37 (score: 0.80): john@example.com
  PhoneNumber at 46..60 (score: 0.70): (555) 123-4567

Processing time: 2ms
```

### Anonymize PII

```bash
# Replace with placeholders (default)
redact anonymize "My SSN is 123-45-6789"
# Output: My SSN is [US_SSN]

# Mask sensitive data
redact anonymize --strategy mask "Email: john@example.com"
# Output: Email: jo**@****le.com

# Hash for consistent pseudonymization
redact anonymize --strategy hash "Card: 4532-1234-5678-9010"
# Output: Card: [CREDIT_CARD_a1b2c3d4]
```

### Process Files

```bash
# Analyze a file
redact analyze -i sensitive_data.txt

# Pipe from stdin
cat document.txt | redact anonymize --strategy mask

# Output as JSON
redact analyze --format json "test@example.com" > results.json
```

### Filter by Entity Type

```bash
redact analyze --entities EmailAddress --entities UsSsn \
  "Email: test@example.com, SSN: 123-45-6789, Phone: (555) 123-4567"
# Only detects EmailAddress and UsSsn, ignores PhoneNumber
```

## Installation

### Using Cargo (Recommended)

```bash
cargo install redact-cli
```

### From Source

```bash
git clone https://github.com/censgate/redact.git
cd redact
cargo build --release
cargo test --workspace
```

### Using Docker

```bash
docker pull ghcr.io/censgate/redact:latest
docker run -p 8080:8080 ghcr.io/censgate/redact:latest
```

### Rust Version

This project requires Rust **1.93.0**. Use [Mise](https://mise.jdx.dev/) or [ASDF](https://asdf-vm.com/) for version management:

```bash
# Using Mise (recommended)
mise install rust@1.93.0

# Using ASDF
asdf install rust 1.93.0

# Using rustup
rustup install 1.93.0
rustup default 1.93.0
```

## Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
redact-core = "0.1"
redact-ner = "0.1"  # Optional: for ML-based NER
```

### Basic Pattern Detection

```rust
use redact_core::{AnalyzerEngine, AnonymizerConfig, AnonymizationStrategy};

fn main() -> anyhow::Result<()> {
    let engine = AnalyzerEngine::new();

    // Analyze text
    let text = "Contact John Doe at john@example.com or call (555) 123-4567";
    let result = engine.analyze(text, None)?;

    println!("Found {} PII entities", result.detected_entities.len());
    for entity in &result.detected_entities {
        println!(
            "  {:?}: {} (score: {:.2})",
            entity.entity_type,
            entity.text.as_deref().unwrap_or_default(),
            entity.score
        );
    }

    // Anonymize
    let config = AnonymizerConfig {
        strategy: AnonymizationStrategy::Replace,
        ..Default::default()
    };
    let anonymized = engine.anonymize(text, None, &config)?;
    println!("\nAnonymized: {}", anonymized.text);

    Ok(())
}
```

### ML-Powered NER

For detecting contextual entities like person names, organizations, and locations:

```rust
use redact_core::AnalyzerEngine;
use redact_ner::{NerRecognizer, NerConfig};
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    // Configure NER with ONNX model
    let ner_config = NerConfig {
        model_path: "models/bert-base-ner/model.onnx".to_string(),
        tokenizer_path: Some("models/bert-base-ner/tokenizer.json".to_string()),
        min_confidence: 0.7,
        ..Default::default()
    };

    let ner = NerRecognizer::from_config(ner_config)?;

    // Add NER to analyzer
    let mut engine = AnalyzerEngine::new();
    engine.recognizer_registry_mut().add_recognizer(Arc::new(ner));

    // Detect both pattern-based and contextual entities
    let text = "John Doe works at Acme Corp. Email: john@acme.com";
    let result = engine.analyze(text, None)?;

    for entity in &result.detected_entities {
        println!("{:?}: {}", entity.entity_type, entity.text.as_deref().unwrap_or_default());
    }
    // Output: PERSON: John Doe, ORGANIZATION: Acme Corp, EMAIL: john@acme.com

    Ok(())
}
```

## REST API

### Start the Server

```bash
cargo run --release --bin redact-api
# Server listening on http://0.0.0.0:8080
```

### Analyze Endpoint

```bash
curl -X POST http://localhost:8080/api/v1/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Email john@example.com, SSN 123-45-6789",
    "language": "en"
  }'
```

Response:

```json
{
  "results": [
    {
      "entity_type": "EMAIL_ADDRESS",
      "start": 6,
      "end": 22,
      "score": 0.8,
      "text": "john@example.com",
      "recognizer_name": "PatternRecognizer"
    },
    {
      "entity_type": "US_SSN",
      "start": 28,
      "end": 39,
      "score": 0.9,
      "text": "123-45-6789",
      "recognizer_name": "PatternRecognizer"
    }
  ],
  "metadata": {
    "recognizers_used": 1,
    "processing_time_ms": 2,
    "language": "en"
  }
}
```

### Anonymize Endpoint

```bash
curl -X POST http://localhost:8080/api/v1/anonymize \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Contact John at john@example.com",
    "config": {
      "strategy": "mask",
      "mask_char": "*",
      "mask_start_chars": 2,
      "mask_end_chars": 4
    }
  }'
```

## Supported Entity Types

### Pattern-Based (36+ types)

| Category | Entity Types |
|----------|-------------|
| **Contact** | `EMAIL_ADDRESS`, `PHONE_NUMBER`, `IP_ADDRESS`, `URL`, `DOMAIN_NAME` |
| **Financial** | `CREDIT_CARD`, `IBAN_CODE`, `US_BANK_NUMBER` |
| **US** | `US_SSN`, `US_DRIVER_LICENSE`, `US_PASSPORT`, `US_ZIP_CODE` |
| **UK** | `UK_NHS`, `UK_NINO`, `UK_POSTCODE`, `UK_PHONE_NUMBER`, `UK_MOBILE_NUMBER`, `UK_SORT_CODE`, `UK_DRIVER_LICENSE`, `UK_PASSPORT_NUMBER`, `UK_COMPANY_NUMBER` |
| **Healthcare** | `MEDICAL_LICENSE`, `MEDICAL_RECORD_NUMBER` |
| **Crypto** | `BTC_ADDRESS`, `ETH_ADDRESS` |
| **Technical** | `GUID`, `MAC_ADDRESS`, `MD5_HASH`, `SHA1_HASH`, `SHA256_HASH` |

### NER-Based (ML-Powered)

| Entity Type | Description |
|-------------|-------------|
| `PERSON` | Person names (e.g., "John Doe", "Marie Curie") |
| `ORGANIZATION` | Organization names (e.g., "Acme Corp", "Microsoft") |
| `LOCATION` | Location names (e.g., "New York", "London") |
| `DATE_TIME` | Date/time expressions in context |

*Requires ONNX model. See [ML-Powered NER](#ml-powered-ner-1) section.*

## Anonymization Strategies

| Strategy | Description | Example |
|----------|-------------|---------|
| **Replace** | Simple placeholder | `[EMAIL_ADDRESS]` |
| **Mask** | Partial masking | `jo**@****le.com` |
| **Hash** | Irreversible hashing | `[EMAIL_ADDRESS_a1b2c3d4]` |
| **Encrypt** | Reversible encryption | `<TOKEN_uuid>` |

```rust
use redact_core::anonymizers::{AnonymizerConfig, AnonymizationStrategy};

let config = AnonymizerConfig {
    strategy: AnonymizationStrategy::Mask,
    mask_char: '*',
    mask_start_chars: 2,
    mask_end_chars: 4,
    ..Default::default()
};
// "john@example.com" → "jo**@****le.com"
```

## ML-Powered NER

Redact includes full ONNX Runtime integration for transformer-based Named Entity Recognition.

### Setup

**1. Export a HuggingFace model to ONNX:**

```bash
pip install transformers optimum[exporters]
python scripts/export_ner_model.py \
    --model dslim/bert-base-NER \
    --output models/bert-base-ner
```

**2. Use in your code:**

```rust
use redact_ner::{NerRecognizer, NerConfig};
use redact_core::AnalyzerEngine;
use std::sync::Arc;

let config = NerConfig {
    model_path: "models/bert-base-ner/model.onnx".to_string(),
    tokenizer_path: Some("models/bert-base-ner/tokenizer.json".to_string()),
    min_confidence: 0.7,
    ..Default::default()
};

let ner = NerRecognizer::from_config(config)?;
let mut engine = AnalyzerEngine::new();
engine.recognizer_registry_mut().add_recognizer(Arc::new(ner));
```

### Recommended Models

| Model | Size | Use Case |
|-------|------|----------|
| `dslim/bert-base-NER` | ~420MB | Best accuracy/size balance |
| `dbmdz/bert-large-cased-finetuned-conll03-english` | ~1.2GB | Highest accuracy |
| `Davlan/distilbert-base-multilingual-cased-ner-hrl` | ~500MB | Multilingual support |

### Performance

- **Inference**: ~2-10ms per text (depending on model and text length)
- **Memory**: ~50-200MB (depending on model)
- **Startup**: ~100-500ms model load time
- **Concurrency**: Thread-safe via mutex-wrapped sessions

## Performance

### Benchmark Results (2026-01-31)

Measured using [oha](https://github.com/hatoo/oha) with both services running in Docker containers.

| Metric | Redact (Rust) | Presidio (Python) | Speedup |
|--------|---------------|-------------------|---------|
| p50 Latency | 0.21 ms | 6.90 ms | **33x** |
| p99 Latency | 1.05 ms | 21.46 ms | **20x** |
| Requests/sec | 3,926 | 134 | **29x** |

Test payload: `Contact john.doe@example.com or call (555) 123-4567. SSN: 123-45-6789.`

### Run Benchmarks

```bash
# REST API comparison vs Presidio (requires Docker + oha)
./scripts/benchmark-comparison.sh

# Criterion micro-benchmarks (Redact internals)
cargo bench --package redact-core
```

See [docs/benchmarks/](docs/benchmarks/) for methodology and detailed results.

## Project Structure

```
redact/
├── crates/
│   ├── redact-core/     # Core detection & anonymization engine
│   ├── redact-ner/      # ONNX NER integration
│   ├── redact-api/      # REST API service (Axum)
│   ├── redact-cli/      # Command-line tool
│   └── redact-wasm/     # WebAssembly bindings
├── patterns/            # PII detection patterns (GDPR, HIPAA, CCPA)
├── scripts/             # Utility scripts (model export)
├── examples/            # Usage examples
└── docs/                # Documentation
```

## Testing

```bash
# Run all tests
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture

# Run benchmarks
cargo bench --package redact-core

# Run NER E2E tests (requires ONNX model)
cargo test --package redact-ner --test ner_e2e -- --ignored

# Run specific test suites
cargo test --package redact-core --test pattern_coverage
cargo test --package redact-core --test error_scenarios
cargo test --package redact-core --test concurrent_operations
```

See [TEST_COVERAGE.md](TEST_COVERAGE.md) for detailed coverage report.

## Documentation

- [API Documentation](https://docs.rs/redact-core) — Rust API docs
- [Test Coverage](TEST_COVERAGE.md) — Testing details
- [Contributing Guide](CONTRIBUTING.md) — How to contribute
- [Examples](examples/) — Code examples

## Roadmap

### v0.1.0 (Current)

- [x] 36+ pattern-based entity types
- [x] Full ONNX NER integration
- [x] 4 anonymization strategies
- [x] REST API service
- [x] CLI tool
- [x] Comprehensive test suite (194 tests, ~75% coverage)

### v0.2.0 (In Progress)

- [ ] WebAssembly (WASM) browser support
- [ ] Publish crates to crates.io
- [ ] Docker images on GitHub Container Registry
- [ ] Enhanced documentation

### v0.3.0 (Planned)

- [ ] Mobile FFI bindings (Swift/Kotlin)
- [ ] Streaming API for large texts
- [ ] GPU acceleration for NER
- [ ] Multi-language support expansion

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/redact.git
cd redact

# Create a feature branch
git checkout -b feature/my-new-feature

# Make changes and test
cargo test --workspace
cargo clippy --all-targets --all-features
cargo fmt --all

# Commit and push
git commit -m "feat: add amazing feature"
git push origin feature/my-new-feature
```

## License

Apache 2.0 — See [LICENSE](LICENSE) for details.

Copyright 2024 Censgate

## Acknowledgments

- Inspired by [Microsoft Presidio](https://microsoft.github.io/presidio/)
- Built with [ONNX Runtime](https://onnxruntime.ai/)
- Powered by [Rust](https://www.rust-lang.org/)
- ML models from [HuggingFace](https://huggingface.co/)

## Support

- [GitHub Issues](https://github.com/censgate/redact/issues) — Bug reports and feature requests
- [GitHub Discussions](https://github.com/censgate/redact/discussions) — Questions and general discussion
- Email: support@censgate.com

---

<div align="center">

**[Star us on GitHub](https://github.com/censgate/redact)** if you find this project useful!

</div>

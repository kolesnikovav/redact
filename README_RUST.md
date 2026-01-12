# Redact - Rust PII Detection Engine

A high-performance, cross-platform PII (Personally Identifiable Information) detection and anonymization engine built in Rust. Designed as a replacement for Microsoft Presidio with support for web, mobile, and server deployments.

## 🚀 Features

- **Pattern-Based Detection**: Fast regex-based recognizers for 30+ PII entity types
- **NER Support**: Named Entity Recognition using ONNX Runtime (ready for model integration)
- **Multiple Anonymization Strategies**: Replace, mask, hash, encrypt
- **Policy-Aware**: Configurable rules with confidence thresholds
- **Multi-Platform**: Server, WASM (browser/mobile), CLI
- **High Performance**: Zero-copy where possible, efficient overlap resolution
- **Type-Safe**: Comprehensive Rust type system for reliability

## 📦 Crates

### `redact-core`
Core PII detection and anonymization engine with:
- Pattern recognizers (regex-based)
- Recognizer registry and orchestration
- Anonymizers (replace, mask, hash, encrypt)
- Analyzer engine
- Policy support

### `redact-ner`
Named Entity Recognition using ONNX Runtime:
- ONNX model loading and inference
- Token-based NER with BIO tagging
- Entity span detection
- Ready for quantized int8 models

### `redact-api`
REST API service using Axum:
- `/analyze` endpoint for PII detection
- `/anonymize` endpoint for anonymization
- JSON request/response
- Policy-based filtering

### `redact-wasm`
WASM bindings for browser and mobile:
- wasm-bindgen integration
- JavaScript/TypeScript bindings
- Runs entirely client-side

### `redact-cli`
Command-line tool:
- Analyze text for PII
- Anonymize detected entities
- Multiple output formats

## 🎯 Supported Entity Types

### Pattern-Based (Regex)
- **Contact**: Email, Phone, IP Address, URL, Domain
- **Financial**: Credit Card, IBAN, Bank Account Numbers
- **US**: SSN, Driver License, Passport
- **UK**: NHS Number, NINO, Postcode, Sort Code
- **Crypto**: Bitcoin, Ethereum addresses
- **Technical**: GUID, MAC Address, MD5/SHA1/SHA256 hashes
- **Temporal**: Date/Time

### NER-Based (ML Model)
- **Person** names
- **Organization** names
- **Location** names
- **Date/Time** expressions

## 🛠️ Installation

### Prerequisites
```bash
# Rust 1.75+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# For WASM target
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

### Build
```bash
# Build all crates
cargo build --release

# Build specific crate
cargo build --package redact-core --release

# Build WASM
cd crates/redact-wasm
wasm-pack build --target web
```

## 📖 Usage

### Rust Library (redact-core)

```rust
use redact_core::{AnalyzerEngine, AnonymizerConfig, AnonymizationStrategy};

fn main() -> anyhow::Result<()> {
    // Create analyzer engine
    let engine = AnalyzerEngine::new();

    // Analyze text for PII
    let text = "Contact John Doe at john@example.com or 555-1234";
    let result = engine.analyze(text, None)?;

    println!("Found {} entities:", result.detected_entities.len());
    for entity in &result.detected_entities {
        println!("  - {:?} at {}..{} (confidence: {:.2})",
            entity.entity_type, entity.start, entity.end, entity.score);
    }

    // Anonymize with replacement
    let config = AnonymizerConfig {
        strategy: AnonymizationStrategy::Replace,
        ..Default::default()
    };

    let anonymized = engine.anonymize(text, None, &config)?;
    println!("Anonymized: {}", anonymized.text);

    Ok(())
}
```

### CLI Tool

```bash
# Analyze text
redact analyze "John Doe lives in New York. Contact: john@example.com"

# Anonymize text
redact anonymize "SSN: 123-45-6789, Email: test@example.com"

# Show version
redact version
```

### Policy-Based Detection

```rust
use redact_core::{AnalyzerEngine, EntityType};

let engine = AnalyzerEngine::new();

// Analyze with specific entity types
let entities_to_detect = vec![
    EntityType::Person,
    EntityType::EmailAddress,
    EntityType::UsSsn,
];

let result = engine.analyze_with_entities(
    text,
    &entities_to_detect,
    None
)?;
```

## 🔧 Anonymization Strategies

### Replace
Simple placeholder replacement:
```rust
AnonymizerConfig {
    strategy: AnonymizationStrategy::Replace,
    ..Default::default()
}
// Result: "Email: [EMAIL_ADDRESS]"
```

### Mask
Partial masking with format preservation:
```rust
AnonymizerConfig {
    strategy: AnonymizationStrategy::Mask,
    mask_char: '*',
    mask_start_chars: 2,
    mask_end_chars: 4,
    preserve_format: true,
    ..Default::default()
}
// Result: "Email: jo**********l.com"
```

### Hash
Irreversible hashing:
```rust
AnonymizerConfig {
    strategy: AnonymizationStrategy::Hash,
    hash_salt: Some("my_salt".to_string()),
    ..Default::default()
}
// Result: "Email: [EMAIL_ADDRESS_a1b2c3d4]"
```

### Encrypt
Reversible encryption:
```rust
AnonymizerConfig {
    strategy: AnonymizationStrategy::Encrypt,
    encryption_key: Some("secret_key".to_string()),
    ..Default::default()
}
// Result: "Email: <TOKEN_uuid>" + tokens for restoration
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test --package redact-core

# Run with output
cargo test -- --nocapture
```

## 📊 Benchmarks

```bash
# Run benchmarks (when implemented)
cargo bench --package redact-core
```

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│         Analyzer Engine                 │
├─────────────────────────────────────────┤
│                                         │
│  ┌──────────────┐   ┌───────────────┐  │
│  │ Recognizer   │   │  Anonymizer   │  │
│  │  Registry    │   │   Registry    │  │
│  └──────────────┘   └───────────────┘  │
│         │                   │           │
│  ┌──────▼──────┐   ┌────────▼──────┐   │
│  │  Pattern    │   │   Replace     │   │
│  │ Recognizer  │   │  Anonymizer   │   │
│  └─────────────┘   └───────────────┘   │
│  ┌─────────────┐   ┌───────────────┐   │
│  │    NER      │   │     Mask      │   │
│  │ Recognizer  │   │  Anonymizer   │   │
│  └─────────────┘   └───────────────┘   │
│                    ┌───────────────┐   │
│                    │     Hash      │   │
│                    │  Anonymizer   │   │
│                    └───────────────┘   │
│                    ┌───────────────┐   │
│                    │   Encrypt     │   │
│                    │  Anonymizer   │   │
│                    └───────────────┘   │
└─────────────────────────────────────────┘
```

## 🔄 Comparison with Presidio

| Feature | Presidio (Python) | Redact (Rust) |
|---------|------------------|---------------|
| Pattern Detection | ✅ | ✅ |
| NER Support | ✅ | ✅ (ready for models) |
| Performance | Good | Excellent |
| Memory Usage | High | Low |
| WASM Support | ❌ | ✅ |
| Mobile Native | ❌ | ✅ |
| Type Safety | Runtime | Compile-time |
| Async | Yes | Yes |

## 🚧 Roadmap

- [x] Core pattern recognizers
- [x] Anonymizer strategies
- [x] Analyzer engine
- [x] NER framework
- [ ] ONNX model integration
- [ ] WASM full implementation
- [ ] REST API service
- [ ] Python model export pipeline
- [ ] Comprehensive benchmarks
- [ ] Mobile FFI bindings (Swift/Kotlin)

## 🤝 Contributing

Contributions welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

Apache 2.0 - See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- Inspired by [Microsoft Presidio](https://microsoft.github.io/presidio/)
- Built with [ONNX Runtime](https://onnxruntime.ai/)
- Powered by [Rust](https://www.rust-lang.org/)

## 📚 Resources

- **Sources:**
  - [Microsoft Presidio GitHub](https://github.com/microsoft/presidio)
  - [Microsoft Presidio Documentation](https://microsoft.github.io/presidio/)
  - [ONNX Runtime Rust](https://docs.rs/ort/)

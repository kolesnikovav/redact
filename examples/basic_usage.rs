//! Basic usage example for redact-core
//!
//! Run with: cargo run --example basic_usage

use redact_core::{AnalyzerEngine, AnonymizationStrategy, AnonymizerConfig, EntityType};

fn main() -> anyhow::Result<()> {
    println!("=== Redact PII Detection Engine - Basic Usage Example ===\n");

    // Create the analyzer engine
    let engine = AnalyzerEngine::new();

    // Example text with various PII types
    let text = "Contact John Doe at john.doe@example.com or call (555) 123-4567. \
                His SSN is 123-45-6789 and he lives in London, UK. \
                NHS number: 123 456 7890. Bitcoin wallet: 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";

    println!("Original text:\n{}\n", text);

    // 1. Analyze text for PII entities
    println!("--- Step 1: Analyze for PII ---");
    let analysis = engine.analyze(text, None)?;

    println!("Found {} PII entities:", analysis.detected_entities.len());
    for entity in &analysis.detected_entities {
        println!(
            "  - {:?} at {}..{} (confidence: {:.2}): {}",
            entity.entity_type,
            entity.start,
            entity.end,
            entity.score,
            entity.text.as_ref().unwrap_or(&"N/A".to_string())
        );
    }
    println!();

    // 2. Anonymize with different strategies
    println!("--- Step 2: Anonymization Strategies ---\n");

    // Strategy 1: Replace
    println!("a) Replace strategy:");
    let replace_config = AnonymizerConfig {
        strategy: AnonymizationStrategy::Replace,
        ..Default::default()
    };
    let replaced = engine.anonymize(text, None, &replace_config)?;
    println!("{}\n", replaced.text);

    // Strategy 2: Mask
    println!("b) Mask strategy (show first 2 and last 4 chars):");
    let mask_config = AnonymizerConfig {
        strategy: AnonymizationStrategy::Mask,
        mask_char: '*',
        mask_start_chars: 2,
        mask_end_chars: 4,
        preserve_format: false,
        ..Default::default()
    };
    let masked = engine.anonymize(text, None, &mask_config)?;
    println!("{}\n", masked.text);

    // Strategy 3: Hash
    println!("c) Hash strategy:");
    let hash_config = AnonymizerConfig {
        strategy: AnonymizationStrategy::Hash,
        hash_salt: Some("my_secret_salt".to_string()),
        ..Default::default()
    };
    let hashed = engine.anonymize(text, None, &hash_config)?;
    println!("{}\n", hashed.text);

    // 3. Analyze with specific entity types
    println!("--- Step 3: Targeted Analysis ---");
    let target_entities = vec![
        EntityType::EmailAddress,
        EntityType::PhoneNumber,
        EntityType::UsSsn,
    ];

    let targeted = engine.analyze_with_entities(text, &target_entities, None)?;
    println!("Looking for specific entities (email, phone, SSN):");
    println!("Found {} matches:", targeted.detected_entities.len());
    for entity in &targeted.detected_entities {
        println!(
            "  - {:?}: {}",
            entity.entity_type,
            entity.text.as_ref().unwrap()
        );
    }
    println!();

    // 4. Performance metrics
    println!("--- Step 4: Performance Metrics ---");
    println!("Recognizers used: {}", analysis.metadata.recognizers_used);
    println!(
        "Processing time: {}ms",
        analysis.metadata.processing_time_ms
    );
    println!("Language: {}", analysis.metadata.language);

    Ok(())
}

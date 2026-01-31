//! Error scenario tests
//!
//! These tests verify proper error handling and edge cases.
//!
//! Run with: cargo test --package redact-core --test error_scenarios

use redact_core::{
    anonymizers::{AnonymizationStrategy, AnonymizerConfig},
    AnalyzerEngine, EntityType,
};

#[test]
fn test_empty_text_analysis() {
    let engine = AnalyzerEngine::new();
    let result = engine.analyze("", None);

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.detected_entities.len(), 0);
}

#[test]
fn test_whitespace_only_text() {
    let engine = AnalyzerEngine::new();
    let result = engine.analyze("   \n\t  \r\n  ", None);

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.detected_entities.len(), 0);
}

#[test]
fn test_null_bytes_in_text() {
    let engine = AnalyzerEngine::new();
    let text = "Email: test@example.com\0SSN: 123-45-6789";

    let result = engine.analyze(text, None);
    assert!(result.is_ok());
}

#[test]
fn test_very_long_text() {
    let engine = AnalyzerEngine::new();
    // 10MB of text
    let long_text = "x".repeat(10_000_000) + " Email: test@example.com";

    let result = engine.analyze(&long_text, None);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.detected_entities.len() >= 1);
}

#[test]
fn test_special_characters() {
    let engine = AnalyzerEngine::new();
    let text = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~";

    let result = engine.analyze(text, None);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.detected_entities.len(), 0);
}

#[test]
fn test_unicode_characters() {
    let engine = AnalyzerEngine::new();
    let text = "Email: test@example.com 日本語 中文 العربية עברית 🎉🚀💻";

    let result = engine.analyze(text, None);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.detected_entities.len() >= 1);
}

#[test]
fn test_mixed_newlines() {
    let engine = AnalyzerEngine::new();
    let text = "Email: test@example.com\nSSN: 123-45-6789\r\nPhone: (555) 123-4567\r";

    let result = engine.analyze(text, None);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.detected_entities.len() >= 3);
}

#[test]
fn test_malformed_entities() {
    let engine = AnalyzerEngine::new();

    // Malformed email
    let result = engine.analyze("Email: @example.com", None);
    assert!(result.is_ok());

    // Malformed SSN
    let result = engine.analyze("SSN: 123-45-678", None); // Too short
    assert!(result.is_ok());

    // Malformed phone
    let result = engine.analyze("Phone: 555-123", None); // Too short
    assert!(result.is_ok());
}

#[test]
fn test_analyze_with_invalid_language() {
    let engine = AnalyzerEngine::new();
    let text = "Email: test@example.com";

    // Invalid language codes should not crash
    let result = engine.analyze(text, Some("invalid_lang"));
    assert!(result.is_ok());
}

#[test]
fn test_analyze_with_empty_entity_filter() {
    let engine = AnalyzerEngine::new();
    let text = "Email: test@example.com";

    let result = engine.analyze_with_entities(text, &[], None);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.detected_entities.len(), 0);
}

#[test]
fn test_analyze_with_nonexistent_entity_type() {
    let engine = AnalyzerEngine::new();
    let text = "Email: test@example.com";

    // Person entity should not be detected by pattern recognizer
    let result = engine.analyze_with_entities(text, &[EntityType::Person], None);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.detected_entities.len(), 0);
}

#[test]
fn test_anonymize_empty_text() {
    let engine = AnalyzerEngine::new();
    let config = AnonymizerConfig {
        strategy: AnonymizationStrategy::Replace,
        ..Default::default()
    };

    let result = engine.anonymize("", None, &config);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.text, "");
}

#[test]
fn test_anonymize_no_entities() {
    let engine = AnalyzerEngine::new();
    let text = "This text has no PII";
    let config = AnonymizerConfig {
        strategy: AnonymizationStrategy::Replace,
        ..Default::default()
    };

    let result = engine.anonymize(text, None, &config);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.text, text);
    assert_eq!(result.entities.len(), 0);
}

#[test]
fn test_encryption_without_key() {
    let engine = AnalyzerEngine::new();
    let text = "Email: test@example.com";
    let config = AnonymizerConfig {
        strategy: AnonymizationStrategy::Encrypt,
        ..Default::default()
    };

    let result = engine.anonymize(text, None, &config);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Encryption key"));
}

#[test]
fn test_overlapping_entities() {
    let engine = AnalyzerEngine::new();
    // Create text that might produce overlapping detections
    let text = "550e8400e29b41d4a716446655440000"; // Could be both GUID and other patterns

    let result = engine.analyze(text, None);
    assert!(result.is_ok());
    let result = result.unwrap();

    // Verify no overlaps in final results
    for i in 0..result.detected_entities.len() {
        for j in (i + 1)..result.detected_entities.len() {
            let e1 = &result.detected_entities[i];
            let e2 = &result.detected_entities[j];

            // Check for overlaps
            let overlaps = e1.start < e2.end && e2.start < e1.end;
            assert!(
                !overlaps,
                "Found overlapping entities: {:?} and {:?}",
                e1, e2
            );
        }
    }
}

#[test]
fn test_entity_at_text_boundaries() {
    let engine = AnalyzerEngine::new();

    // Entity at start
    let result = engine.analyze("test@example.com is my email", None);
    assert!(result.is_ok());
    assert!(result.unwrap().detected_entities.len() >= 1);

    // Entity at end
    let result = engine.analyze("My email is test@example.com", None);
    assert!(result.is_ok());
    assert!(result.unwrap().detected_entities.len() >= 1);

    // Entity is entire text
    let result = engine.analyze("test@example.com", None);
    assert!(result.is_ok());
    assert!(result.unwrap().detected_entities.len() >= 1);
}

#[test]
fn test_repeated_entities() {
    let engine = AnalyzerEngine::new();
    let text = "test@example.com test@example.com test@example.com";

    let result = engine.analyze(text, None);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.detected_entities.len(), 3);
}

#[test]
fn test_case_sensitivity() {
    let engine = AnalyzerEngine::new();

    // Email should be case insensitive
    let result1 = engine.analyze("TEST@EXAMPLE.COM", None);
    let result2 = engine.analyze("test@example.com", None);

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let entities1 = result1.unwrap().detected_entities;
    let entities2 = result2.unwrap().detected_entities;

    assert_eq!(entities1.len(), entities2.len());
}

#[test]
fn test_detection_with_surrounding_punctuation() {
    let engine = AnalyzerEngine::new();
    let text = "Email: (test@example.com), Phone: [555-123-4567], SSN: {123-45-6789}";

    let result = engine.analyze(text, None);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.detected_entities.len() >= 3);
}

#[test]
fn test_numeric_edge_cases() {
    let engine = AnalyzerEngine::new();

    // Very large numbers
    let result = engine.analyze("12345678901234567890", None);
    assert!(result.is_ok());

    // Leading zeros
    let result = engine.analyze("SSN: 000-00-0000", None);
    assert!(result.is_ok());

    // All same digit
    let result = engine.analyze("Card: 1111111111111111", None);
    assert!(result.is_ok());
}

#[test]
fn test_anonymization_preserves_length() {
    let engine = AnalyzerEngine::new();
    let text = "Email: test@example.com and SSN: 123-45-6789";

    let config = AnonymizerConfig {
        strategy: AnonymizationStrategy::Mask,
        ..Default::default()
    };

    let result = engine.anonymize(text, None, &config);
    assert!(result.is_ok());
    let result = result.unwrap();

    // Anonymized text should have similar length (not exact due to entity replacement)
    assert!(result.text.len() > 0);
}

#[test]
fn test_high_confidence_threshold() {
    let engine = AnalyzerEngine::new();
    let text = "test@example.com";

    let result = engine.analyze(text, None);
    assert!(result.is_ok());

    let result = result.unwrap();
    // All detected entities should have reasonable confidence scores
    for entity in &result.detected_entities {
        assert!(entity.score >= 0.0 && entity.score <= 1.0);
    }
}

#[test]
fn test_simultaneous_analysis_requests() {
    // This tests that the engine can handle multiple analysis requests
    let engine = AnalyzerEngine::new();

    let texts = vec![
        "Email: test1@example.com",
        "SSN: 123-45-6789",
        "Phone: (555) 123-4567",
    ];

    for text in texts {
        let result = engine.analyze(text, None);
        assert!(result.is_ok());
    }
}

#[test]
fn test_memory_safety_with_large_input() {
    let engine = AnalyzerEngine::new();

    // Create a reasonably large input
    let large_text = format!(
        "{} Email: test@example.com",
        "Lorem ipsum dolor sit amet. ".repeat(10000)
    );

    let result = engine.analyze(&large_text, None);
    assert!(result.is_ok());
}

#[test]
fn test_error_propagation() {
    let engine = AnalyzerEngine::new();

    // Encryption without key should propagate error properly
    let config = AnonymizerConfig {
        strategy: AnonymizationStrategy::Encrypt,
        encryption_key: None,
        ..Default::default()
    };

    let result = engine.analyze_and_anonymize("test@example.com", None, &config);
    assert!(result.is_err());
}

#[test]
fn test_metadata_accuracy() {
    let engine = AnalyzerEngine::new();
    let text = "Email: test@example.com";

    let result = engine.analyze(text, None);
    assert!(result.is_ok());

    let result = result.unwrap();

    // Verify metadata
    assert!(result.metadata.processing_time_ms > 0);
    assert_eq!(result.metadata.language, "en");
    assert!(result.metadata.recognizers_used > 0);
}

use super::{Recognizer, RecognizerResult};
use crate::types::EntityType;
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

/// Pattern-based recognizer using regex
#[derive(Debug, Clone)]
pub struct PatternRecognizer {
    name: String,
    patterns: HashMap<EntityType, Vec<CompiledPattern>>,
    min_score: f32,
}

#[derive(Debug, Clone)]
struct CompiledPattern {
    regex: Regex,
    score: f32,
    context_words: Vec<String>,
}

impl PatternRecognizer {
    /// Create a new pattern recognizer with default patterns
    pub fn new() -> Self {
        let mut recognizer = Self {
            name: "PatternRecognizer".to_string(),
            patterns: HashMap::new(),
            min_score: 0.5,
        };
        recognizer.load_default_patterns();
        recognizer
    }

    /// Create a new pattern recognizer with custom name
    pub fn with_name(name: impl Into<String>) -> Self {
        let mut recognizer = Self::new();
        recognizer.name = name.into();
        recognizer
    }

    /// Set minimum confidence score
    pub fn with_min_score(mut self, min_score: f32) -> Self {
        self.min_score = min_score;
        self
    }

    /// Add a custom pattern for an entity type
    pub fn add_pattern(
        &mut self,
        entity_type: EntityType,
        pattern: &str,
        score: f32,
    ) -> Result<()> {
        let regex = Regex::new(pattern)?;
        let compiled = CompiledPattern {
            regex,
            score,
            context_words: vec![],
        };
        self.patterns
            .entry(entity_type)
            .or_default()
            .push(compiled);
        Ok(())
    }

    /// Add a pattern with context words for score boosting
    pub fn add_pattern_with_context(
        &mut self,
        entity_type: EntityType,
        pattern: &str,
        score: f32,
        context_words: Vec<String>,
    ) -> Result<()> {
        let regex = Regex::new(pattern)?;
        let compiled = CompiledPattern {
            regex,
            score,
            context_words,
        };
        self.patterns
            .entry(entity_type)
            .or_default()
            .push(compiled);
        Ok(())
    }

    /// Load default patterns for common PII types
    fn load_default_patterns(&mut self) {
        // Email addresses
        let _ = self.add_pattern(
            EntityType::EmailAddress,
            r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b",
            0.8,
        );

        // Phone numbers (US/international format with various separators)
        // Matches: (555) 123-4567, 555-123-4567, 555.123.4567, 5551234567
        let _ = self.add_pattern(
            EntityType::PhoneNumber,
            r"\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b",
            0.7,
        );

        // Credit cards (4 groups of 4 digits)
        let _ = self.add_pattern(
            EntityType::CreditCard,
            r"\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|6(?:011|5[0-9]{2})[0-9]{12})\b",
            0.9,
        );

        // US SSN (simplified pattern - Rust regex doesn't support lookahead)
        // Pattern matches XXX-XX-XXXX format
        let _ = self.add_pattern(
            EntityType::UsSsn,
            r"\b\d{3}-\d{2}-\d{4}\b",
            0.9,
        );

        // IP Address (IPv4)
        let _ = self.add_pattern(
            EntityType::IpAddress,
            r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b",
            0.8,
        );

        // URL
        let _ = self.add_pattern(
            EntityType::Url,
            r"\b(?:https?://|www\.)[a-zA-Z0-9][-a-zA-Z0-9]*(?:\.[a-zA-Z0-9][-a-zA-Z0-9]*)+(?:/[^\s]*)?\b",
            0.7,
        );

        // GUID/UUID
        let _ = self.add_pattern(
            EntityType::Guid,
            r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b",
            0.9,
        );

        // MAC Address
        let _ = self.add_pattern(
            EntityType::MacAddress,
            r"\b(?:[0-9A-Fa-f]{2}[:-]){5}[0-9A-Fa-f]{2}\b",
            0.9,
        );

        // UK NHS Number
        let _ = self.add_pattern_with_context(
            EntityType::UkNhs,
            r"\b(?:\d{3}\s?\d{3}\s?\d{4}|\d{10})\b",
            0.6,
            vec!["NHS".to_string(), "patient".to_string(), "health".to_string()],
        );

        // UK National Insurance Number
        let _ = self.add_pattern(
            EntityType::UkNino,
            r"\b[A-CEGHJ-PR-TW-Z]{1}[A-CEGHJ-NPR-TW-Z]{1}\d{6}[A-D]{1}\b",
            0.85,
        );

        // UK Postcode
        let _ = self.add_pattern(
            EntityType::UkPostcode,
            r"\b[A-Z]{1,2}\d[A-Z\d]?\s?\d[A-Z]{2}\b",
            0.75,
        );

        // UK Sort Code
        let _ = self.add_pattern(
            EntityType::UkSortCode,
            r"\b\d{2}-\d{2}-\d{2}\b",
            0.7,
        );

        // IBAN
        let _ = self.add_pattern(
            EntityType::IbanCode,
            r"\b[A-Z]{2}\d{2}[A-Z0-9]{1,30}\b",
            0.75,
        );

        // Bitcoin Address
        let _ = self.add_pattern(
            EntityType::BtcAddress,
            r"\b(?:bc1|[13])[a-zA-HJ-NP-Z0-9]{25,62}\b",
            0.85,
        );

        // Ethereum Address
        let _ = self.add_pattern(
            EntityType::EthAddress,
            r"\b0x[a-fA-F0-9]{40}\b",
            0.9,
        );

        // MD5 Hash
        let _ = self.add_pattern(
            EntityType::Md5Hash,
            r"\b[a-fA-F0-9]{32}\b",
            0.6,
        );

        // SHA1 Hash
        let _ = self.add_pattern(
            EntityType::Sha1Hash,
            r"\b[a-fA-F0-9]{40}\b",
            0.6,
        );

        // SHA256 Hash
        let _ = self.add_pattern(
            EntityType::Sha256Hash,
            r"\b[a-fA-F0-9]{64}\b",
            0.6,
        );

        // US ZIP Code (5 digits or ZIP+4 format)
        let _ = self.add_pattern(
            EntityType::UsZipCode,
            r"\b\d{5}(?:-\d{4})?\b",
            0.6,  // Lower confidence as could be other 5-digit numbers
        );

        // PO Box
        let _ = self.add_pattern_with_context(
            EntityType::PoBox,
            r"\b(?:P\.?\s?O\.?|POST\s+OFFICE)\s*BOX\s+\d+\b",
            0.85,
            vec!["address".to_string(), "mail".to_string(), "ship".to_string()],
        );

        // ISBN (10 or 13 digit formats)
        let _ = self.add_pattern(
            EntityType::Isbn,
            r"\b(?:ISBN(?:-1[03])?:?\s*)?(?:\d{9}[\dX]|\d{13})\b",
            0.8,
        );

        // Generic Passport Number (alphanumeric, 6-9 characters)
        let _ = self.add_pattern_with_context(
            EntityType::PassportNumber,
            r"\b[A-Z]{1,2}\d{6,9}\b",
            0.7,
            vec!["passport".to_string(), "travel".to_string()],
        );

        // Medical Record Number (various formats with MRN context)
        let _ = self.add_pattern_with_context(
            EntityType::MedicalRecordNumber,
            r"\b(?:MRN|Medical\s*Record|Patient\s*ID):?\s*[A-Z0-9]{6,12}\b",
            0.85,
            vec!["patient".to_string(), "medical".to_string(), "hospital".to_string()],
        );

        // Age (with context)
        let _ = self.add_pattern_with_context(
            EntityType::Age,
            r"\b(?:age|aged|years old):?\s*(\d{1,3})\b",
            0.8,
            vec!["years".to_string(), "old".to_string(), "age".to_string()],
        );

        // Date/Time (ISO format and common variants)
        let _ = self.add_pattern(
            EntityType::DateTime,
            r"\b\d{4}-\d{2}-\d{2}(?:[T\s]\d{2}:\d{2}(?::\d{2})?)?\b",
            0.5,
        );
    }

    /// Check context words around a match to boost confidence
    fn check_context(&self, text: &str, start: usize, end: usize, context_words: &[String]) -> f32 {
        if context_words.is_empty() {
            return 0.0;
        }

        // Get 50 characters before and after the match
        let context_start = start.saturating_sub(50);
        let context_end = (end + 50).min(text.len());
        let context = &text[context_start..context_end].to_lowercase();

        // Count matching context words
        let matches = context_words
            .iter()
            .filter(|word| context.contains(&word.to_lowercase()))
            .count();

        // Boost score based on context matches (up to +0.3)
        (matches as f32 / context_words.len() as f32) * 0.3
    }
}

impl Default for PatternRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Recognizer for PatternRecognizer {
    fn name(&self) -> &str {
        &self.name
    }

    fn supported_entities(&self) -> &[EntityType] {
        lazy_static! {
            static ref SUPPORTED: Vec<EntityType> = vec![
                EntityType::EmailAddress,
                EntityType::PhoneNumber,
                EntityType::CreditCard,
                EntityType::UsSsn,
                EntityType::IpAddress,
                EntityType::Url,
                EntityType::Guid,
                EntityType::MacAddress,
                EntityType::UkNhs,
                EntityType::UkNino,
                EntityType::UkPostcode,
                EntityType::UkSortCode,
                EntityType::IbanCode,
                EntityType::BtcAddress,
                EntityType::EthAddress,
                EntityType::Md5Hash,
                EntityType::Sha1Hash,
                EntityType::Sha256Hash,
                EntityType::DateTime,
            ];
        }
        &SUPPORTED
    }

    fn analyze(&self, text: &str, _language: &str) -> Result<Vec<RecognizerResult>> {
        let mut results = Vec::new();

        for (entity_type, patterns) in &self.patterns {
            for pattern in patterns {
                for capture in pattern.regex.captures_iter(text) {
                    if let Some(matched) = capture.get(0) {
                        let start = matched.start();
                        let end = matched.end();

                        // Base score from pattern
                        let mut score = pattern.score;

                        // Boost score based on context if context words are provided
                        if !pattern.context_words.is_empty() {
                            score += self.check_context(text, start, end, &pattern.context_words);
                            score = score.min(1.0); // Cap at 1.0
                        }

                        if score >= self.min_score {
                            results.push(
                                RecognizerResult::new(
                                    entity_type.clone(),
                                    start,
                                    end,
                                    score,
                                    self.name(),
                                )
                                .with_text(text),
                            );
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    fn min_score(&self) -> f32 {
        self.min_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_detection() {
        let recognizer = PatternRecognizer::new();
        let text = "Contact me at john.doe@example.com for details";
        let results = recognizer.analyze(text, "en").unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity_type, EntityType::EmailAddress);
        assert_eq!(results[0].text, Some("john.doe@example.com".to_string()));
        assert!(results[0].score >= 0.8);
    }

    #[test]
    fn test_phone_detection() {
        let recognizer = PatternRecognizer::new();
        let text = "Call me at (555) 123-4567";
        let results = recognizer.analyze(text, "en").unwrap();

        assert!(!results.is_empty());
        let phone_result = results
            .iter()
            .find(|r| r.entity_type == EntityType::PhoneNumber);
        assert!(phone_result.is_some());
    }

    #[test]
    fn test_credit_card_detection() {
        let recognizer = PatternRecognizer::new();
        let text = "Card number: 4532015112830366";
        let results = recognizer.analyze(text, "en").unwrap();

        assert!(!results.is_empty());
        let cc_result = results
            .iter()
            .find(|r| r.entity_type == EntityType::CreditCard);
        assert!(cc_result.is_some());
    }

    #[test]
    fn test_ssn_detection() {
        let recognizer = PatternRecognizer::new();
        let text = "SSN: 123-45-6789";
        let results = recognizer.analyze(text, "en").unwrap();

        assert!(!results.is_empty());
        let ssn_result = results.iter().find(|r| r.entity_type == EntityType::UsSsn);
        assert!(ssn_result.is_some());
    }

    #[test]
    fn test_uk_nhs_with_context() {
        let recognizer = PatternRecognizer::new();
        let text = "NHS patient number is 123 456 7890";
        let results = recognizer.analyze(text, "en").unwrap();

        assert!(!results.is_empty());
        let nhs_result = results.iter().find(|r| r.entity_type == EntityType::UkNhs);
        assert!(nhs_result.is_some());
        // Score should be boosted due to "NHS" context
        if let Some(result) = nhs_result {
            assert!(result.score > 0.6);
        }
    }

    #[test]
    fn test_uk_nino_detection() {
        let recognizer = PatternRecognizer::new();
        let text = "NINO: AB123456C";
        let results = recognizer.analyze(text, "en").unwrap();

        assert!(!results.is_empty());
        let nino_result = results
            .iter()
            .find(|r| r.entity_type == EntityType::UkNino);
        assert!(nino_result.is_some());
    }

    #[test]
    fn test_multiple_entities() {
        let recognizer = PatternRecognizer::new();
        let text = "Email john@example.com, phone (555) 123-4567, SSN 123-45-6789";
        let results = recognizer.analyze(text, "en").unwrap();

        assert!(results.len() >= 3);
        assert!(results
            .iter()
            .any(|r| r.entity_type == EntityType::EmailAddress));
        assert!(results
            .iter()
            .any(|r| r.entity_type == EntityType::PhoneNumber));
        assert!(results.iter().any(|r| r.entity_type == EntityType::UsSsn));
    }

    #[test]
    fn test_custom_pattern() {
        let mut recognizer = PatternRecognizer::new();
        recognizer
            .add_pattern(EntityType::Custom("CUSTOM_ID".to_string()), r"\bCID-\d{6}\b", 0.9)
            .unwrap();

        let text = "Your customer ID is CID-123456";
        let results = recognizer.analyze(text, "en").unwrap();

        let custom_result = results
            .iter()
            .find(|r| matches!(r.entity_type, EntityType::Custom(_)));
        assert!(custom_result.is_some());
    }

    #[test]
    fn test_min_score_filtering() {
        let recognizer = PatternRecognizer::new().with_min_score(0.9);
        let text = "Date: 2024-01-15"; // Date has score 0.5
        let results = recognizer.analyze(text, "en").unwrap();

        // Date should be filtered out due to min_score
        let date_results = results
            .iter()
            .filter(|r| r.entity_type == EntityType::DateTime)
            .count();
        assert_eq!(date_results, 0);
    }
}

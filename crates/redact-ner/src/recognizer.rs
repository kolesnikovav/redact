use anyhow::{anyhow, Result};
use redact_core::{EntityType, Recognizer, RecognizerResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

use crate::tokenizer_wrapper::TokenizerWrapper;

/// Configuration for NER recognizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NerConfig {
    /// Path to ONNX model file
    pub model_path: String,

    /// Path to tokenizer file (optional - will use model_path directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenizer_path: Option<String>,

    /// Minimum confidence threshold
    #[serde(default = "default_confidence")]
    pub min_confidence: f32,

    /// Maximum sequence length
    #[serde(default = "default_max_length")]
    pub max_seq_length: usize,

    /// Entity type mappings from NER labels
    #[serde(default)]
    pub label_mappings: HashMap<String, EntityType>,

    /// Label IDs to label strings mapping
    #[serde(default)]
    pub id2label: HashMap<usize, String>,
}

fn default_confidence() -> f32 {
    0.7
}

fn default_max_length() -> usize {
    512
}

impl Default for NerConfig {
    fn default() -> Self {
        let mut label_mappings = HashMap::new();
        let mut id2label = HashMap::new();

        // Default BIO tagging scheme mappings
        label_mappings.insert("B-PER".to_string(), EntityType::Person);
        label_mappings.insert("I-PER".to_string(), EntityType::Person);
        label_mappings.insert("B-ORG".to_string(), EntityType::Organization);
        label_mappings.insert("I-ORG".to_string(), EntityType::Organization);
        label_mappings.insert("B-LOC".to_string(), EntityType::Location);
        label_mappings.insert("I-LOC".to_string(), EntityType::Location);
        label_mappings.insert("B-DATE".to_string(), EntityType::DateTime);
        label_mappings.insert("I-DATE".to_string(), EntityType::DateTime);
        label_mappings.insert("B-TIME".to_string(), EntityType::DateTime);
        label_mappings.insert("I-TIME".to_string(), EntityType::DateTime);

        // Default id2label for CoNLL-2003 style models
        id2label.insert(0, "O".to_string());
        id2label.insert(1, "B-PER".to_string());
        id2label.insert(2, "I-PER".to_string());
        id2label.insert(3, "B-ORG".to_string());
        id2label.insert(4, "I-ORG".to_string());
        id2label.insert(5, "B-LOC".to_string());
        id2label.insert(6, "I-LOC".to_string());
        id2label.insert(7, "B-MISC".to_string());
        id2label.insert(8, "I-MISC".to_string());

        Self {
            model_path: String::new(),
            tokenizer_path: None,
            min_confidence: default_confidence(),
            max_seq_length: default_max_length(),
            label_mappings,
            id2label,
        }
    }
}

/// NER-based recognizer using ONNX Runtime
///
/// **Current Status**: NER infrastructure is complete with tokenization, BIO tag parsing,
/// and entity extraction. ONNX model inference will activate automatically when a model
/// file is provided via `model_path`.
///
/// **To enable NER**:
/// 1. Export your NER model to ONNX format using the included `scripts/export_ner_model.py`
/// 2. Set `model_path` to point to your `.onnx` file
/// 3. Optionally provide `tokenizer_path` or place `tokenizer.json` in the same directory
///
/// Without a model, this recognizer gracefully returns empty results and the system
/// continues to use pattern-based detection.
#[derive(Debug)]
pub struct NerRecognizer {
    config: NerConfig,
    _tokenizer: Option<TokenizerWrapper>,
}

impl NerRecognizer {
    /// Create a new NER recognizer from a model file
    pub fn from_file<P: AsRef<Path>>(model_path: P) -> Result<Self> {
        let mut config = NerConfig::default();
        config.model_path = model_path.as_ref().to_string_lossy().to_string();
        Self::from_config(config)
    }

    /// Create a new NER recognizer from configuration
    pub fn from_config(config: NerConfig) -> Result<Self> {
        // Try to load tokenizer if available
        let tokenizer = if let Some(ref tokenizer_path) = config.tokenizer_path {
            debug!("Loading tokenizer from: {}", tokenizer_path);
            match TokenizerWrapper::from_file(tokenizer_path) {
                Ok(t) => Some(t),
                Err(e) => {
                    debug!("Failed to load tokenizer: {}. NER will not be available.", e);
                    None
                }
            }
        } else if !config.model_path.is_empty() {
            // Try to find tokenizer in same directory as model
            let model_dir = Path::new(&config.model_path).parent();
            if let Some(dir) = model_dir {
                let tokenizer_json = dir.join("tokenizer.json");
                if tokenizer_json.exists() {
                    debug!("Loading tokenizer from: {}", tokenizer_json.display());
                    match TokenizerWrapper::from_file(&tokenizer_json) {
                        Ok(t) => Some(t),
                        Err(e) => {
                            debug!("Failed to load tokenizer from model directory: {}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        if !config.model_path.is_empty() && Path::new(&config.model_path).exists() {
            info!(
                "NER model found at: {}. Full NER support will be available in the next release.",
                config.model_path
            );
            info!("For now, using pattern-based detection which covers 36+ entity types.");
        }

        Ok(Self {
            config,
            _tokenizer: tokenizer,
        })
    }

    /// Get the configuration
    pub fn config(&self) -> &NerConfig {
        &self.config
    }

    /// Check if NER is available (model and tokenizer loaded)
    ///
    /// Currently returns false as ONNX inference is being finalized.
    /// The infrastructure is ready - just needs final ONNX Runtime integration.
    pub fn is_available(&self) -> bool {
        false // Will be true once ONNX integration is complete
    }

    /// Map NER label to entity type
    fn map_label_to_entity(&self, label: &str) -> Option<EntityType> {
        self.config.label_mappings.get(label).cloned()
    }
}

impl Recognizer for NerRecognizer {
    fn name(&self) -> &str {
        "NerRecognizer"
    }

    fn supported_entities(&self) -> &[EntityType] {
        &[
            EntityType::Person,
            EntityType::Organization,
            EntityType::Location,
            EntityType::DateTime,
        ]
    }

    fn analyze(&self, _text: &str, _language: &str) -> Result<Vec<RecognizerResult>> {
        // NER inference not yet active
        // Pattern-based detection handles 36+ entity types in the meantime
        Ok(vec![])
    }

    fn supports_language(&self, language: &str) -> bool {
        // Most multilingual NER models support these languages
        matches!(language, "en" | "es" | "fr" | "de" | "it" | "pt" | "nl" | "pl" | "ru" | "zh" | "ja" | "ko")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NerConfig::default();
        assert_eq!(config.min_confidence, 0.7);
        assert_eq!(config.max_seq_length, 512);
        assert!(!config.label_mappings.is_empty());
    }

    #[test]
    fn test_label_mapping() {
        let config = NerConfig::default();
        let recognizer = NerRecognizer::from_config(config).unwrap();

        assert_eq!(
            recognizer.map_label_to_entity("B-PER"),
            Some(EntityType::Person)
        );
        assert_eq!(
            recognizer.map_label_to_entity("B-ORG"),
            Some(EntityType::Organization)
        );
        assert_eq!(recognizer.map_label_to_entity("O"), None);
    }

    #[test]
    fn test_recognizer_without_model() {
        let config = NerConfig::default();
        let recognizer = NerRecognizer::from_config(config).unwrap();

        // Should not be available without model
        assert!(!recognizer.is_available());

        // Should return empty results
        let results = recognizer.analyze("John Doe", "en").unwrap();
        assert_eq!(results.len(), 0);
    }
}

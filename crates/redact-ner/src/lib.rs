//! NER-based PII Recognition using ONNX Runtime
//!
//! This crate provides Named Entity Recognition (NER) capabilities for PII detection
//! using quantized ONNX models for efficient inference.
//!
//! # Features
//!
//! - ONNX Runtime integration for model inference
//! - Support for quantized int8 models
//! - Token-based NER with entity span detection
//! - Compatible with various NER model architectures (BERT, RoBERTa, etc.)
//!
//! # Example
//!
//! ```no_run
//! use redact_ner::NerRecognizer;
//!
//! // Load model
//! let recognizer = NerRecognizer::from_file("model.onnx").unwrap();
//!
//! // Analyze text
//! let text = "John Doe works at Acme Corp in New York";
//! let results = recognizer.analyze(text, "en").unwrap();
//!
//! for result in results {
//!     println!("{:?}: {}", result.entity_type, result.text.unwrap());
//! }
//! ```

mod recognizer;
mod tokenizer_wrapper;

pub use recognizer::{NerRecognizer, NerConfig};

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // Placeholder test
        assert!(true);
    }
}

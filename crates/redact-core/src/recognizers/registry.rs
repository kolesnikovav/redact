// Copyright (c) 2026 Censgate LLC.
// Licensed under the Business Source License 1.1 (BUSL-1.1).
// See the LICENSE file in the project root for license details,
// including the Additional Use Grant, Change Date, and Change License.

use super::{Recognizer, RecognizerResult};
use crate::types::EntityType;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Registry for managing multiple recognizers
#[derive(Debug, Clone)]
pub struct RecognizerRegistry {
    recognizers: Vec<Arc<dyn Recognizer>>,
    entity_map: HashMap<EntityType, Vec<usize>>,
}

impl RecognizerRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            recognizers: Vec::new(),
            entity_map: HashMap::new(),
        }
    }

    /// Add a recognizer to the registry
    pub fn add_recognizer(&mut self, recognizer: Arc<dyn Recognizer>) {
        let index = self.recognizers.len();

        // Map entity types to recognizer index
        for entity_type in recognizer.supported_entities() {
            self.entity_map
                .entry(entity_type.clone())
                .or_default()
                .push(index);
        }

        self.recognizers.push(recognizer);
    }

    /// Get all recognizers
    pub fn recognizers(&self) -> &[Arc<dyn Recognizer>] {
        &self.recognizers
    }

    /// Get recognizers that support a specific entity type
    pub fn recognizers_for_entity(&self, entity_type: &EntityType) -> Vec<Arc<dyn Recognizer>> {
        if let Some(indices) = self.entity_map.get(entity_type) {
            indices
                .iter()
                .map(|&idx| self.recognizers[idx].clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Analyze text using all recognizers
    pub fn analyze(&self, text: &str, language: &str) -> Result<Vec<RecognizerResult>> {
        let mut all_results = Vec::new();

        for recognizer in &self.recognizers {
            // Skip recognizers that don't support the language
            if !recognizer.supports_language(language) {
                continue;
            }

            let results = recognizer.analyze(text, language).with_context(|| {
                format!("Failed to analyze with recognizer: {}", recognizer.name())
            })?;

            all_results.extend(results);
        }

        // Sort and resolve overlaps
        all_results.sort();
        let resolved = self.resolve_overlaps(all_results);

        Ok(resolved)
    }

    /// Analyze text using only specific entity types
    pub fn analyze_with_entities(
        &self,
        text: &str,
        language: &str,
        entity_types: &[EntityType],
    ) -> Result<Vec<RecognizerResult>> {
        let mut all_results = Vec::new();

        // Get unique recognizers that support the requested entities
        let mut used_recognizers = std::collections::HashSet::new();

        for entity_type in entity_types {
            if let Some(indices) = self.entity_map.get(entity_type) {
                used_recognizers.extend(indices.iter().copied());
            }
        }

        for idx in used_recognizers {
            let recognizer = &self.recognizers[idx];

            if !recognizer.supports_language(language) {
                continue;
            }

            let results = recognizer.analyze(text, language).with_context(|| {
                format!("Failed to analyze with recognizer: {}", recognizer.name())
            })?;

            // Filter to only requested entity types
            let filtered: Vec<_> = results
                .into_iter()
                .filter(|r| entity_types.contains(&r.entity_type))
                .collect();

            all_results.extend(filtered);
        }

        all_results.sort();
        let resolved = self.resolve_overlaps(all_results);

        Ok(resolved)
    }

    /// Resolve overlapping detections by keeping highest confidence
    fn resolve_overlaps(&self, results: Vec<RecognizerResult>) -> Vec<RecognizerResult> {
        if results.is_empty() {
            return results;
        }

        let mut resolved = Vec::new();
        let mut skip_until = 0;

        for i in 0..results.len() {
            if i < skip_until {
                continue;
            }

            let current = &results[i];
            let mut keep = current.clone();
            skip_until = i + 1;

            // Check for overlaps with subsequent results
            for (offset, next) in results.iter().enumerate().skip(i + 1) {
                // If next result doesn't overlap, we're done
                if !current.overlaps_with(next) {
                    break;
                }

                // If current contains next, check scores
                if current.contains(next) {
                    // Keep the one with higher score
                    if next.score > keep.score {
                        keep = next.clone();
                    }
                    skip_until = offset + 1;
                } else if next.contains(current) {
                    // Next contains current, prefer the larger one with higher score
                    if next.score >= keep.score {
                        keep = next.clone();
                    }
                    skip_until = offset + 1;
                } else {
                    // Partial overlap - keep the one with higher score
                    if next.score > keep.score {
                        keep = next.clone();
                        skip_until = offset + 1;
                    }
                }
            }

            resolved.push(keep);
        }

        resolved
    }

    /// Get statistics about the registry
    pub fn stats(&self) -> RegistryStats {
        let mut entity_coverage = HashMap::new();
        for (entity_type, indices) in &self.entity_map {
            entity_coverage.insert(entity_type.clone(), indices.len());
        }

        RegistryStats {
            recognizer_count: self.recognizers.len(),
            entity_coverage,
        }
    }
}

impl Default for RecognizerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about a recognizer registry
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub recognizer_count: usize,
    pub entity_coverage: HashMap<EntityType, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recognizers::pattern::PatternRecognizer;

    #[test]
    fn test_registry_add_recognizer() {
        let mut registry = RecognizerRegistry::new();
        let recognizer = Arc::new(PatternRecognizer::new());

        registry.add_recognizer(recognizer);

        assert_eq!(registry.recognizers().len(), 1);
    }

    #[test]
    fn test_registry_analyze() {
        let mut registry = RecognizerRegistry::new();
        let recognizer = Arc::new(PatternRecognizer::new());
        registry.add_recognizer(recognizer);

        let text = "Email: john@example.com, Phone: (555) 123-4567";
        let results = registry.analyze(text, "en").unwrap();

        assert!(results.len() >= 2);
    }

    #[test]
    fn test_registry_analyze_with_entities() {
        let mut registry = RecognizerRegistry::new();
        let recognizer = Arc::new(PatternRecognizer::new());
        registry.add_recognizer(recognizer);

        let text = "Email: john@example.com, Phone: (555) 123-4567";
        let results = registry
            .analyze_with_entities(text, "en", &[EntityType::EmailAddress])
            .unwrap();

        // Should only get email results
        assert!(results
            .iter()
            .all(|r| r.entity_type == EntityType::EmailAddress));
    }

    #[test]
    fn test_overlap_resolution() {
        let mut registry = RecognizerRegistry::new();

        // Create overlapping results
        let mut results = vec![
            RecognizerResult::new(EntityType::Person, 0, 10, 0.8, "test1"),
            RecognizerResult::new(EntityType::Person, 5, 15, 0.9, "test2"),
        ];

        results.sort();
        let resolved = registry.resolve_overlaps(results);

        // Should keep only the higher confidence result
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].score, 0.9);
    }

    #[test]
    fn test_recognizers_for_entity() {
        let mut registry = RecognizerRegistry::new();
        let recognizer = Arc::new(PatternRecognizer::new());
        registry.add_recognizer(recognizer);

        let recognizers = registry.recognizers_for_entity(&EntityType::EmailAddress);
        assert_eq!(recognizers.len(), 1);

        let recognizers = registry.recognizers_for_entity(&EntityType::Person);
        assert_eq!(recognizers.len(), 0); // Pattern recognizer doesn't support Person
    }

    #[test]
    fn test_registry_stats() {
        let mut registry = RecognizerRegistry::new();
        let recognizer = Arc::new(PatternRecognizer::new());
        registry.add_recognizer(recognizer);

        let stats = registry.stats();
        assert_eq!(stats.recognizer_count, 1);
        assert!(!stats.entity_coverage.is_empty());
    }
}

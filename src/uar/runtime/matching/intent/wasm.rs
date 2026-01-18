//! WASM-based intent classifier stub.
//!
//! This module provides an adapter for WASM components that implement
//! the intent classification interface. The actual WASM component loading
//! is not implemented yet; this is a placeholder for future use.
//!
//! ## WIT Interface (proposed)
//!
//! ```wit
//! package intent:classifier;
//!
//! interface classifier {
//!     /// Classifies the input query and returns JSON with scores.
//!     /// Returns: [{ "label": "skill_id", "score": 0.87 }, ...]
//!     classify: func(input: string) -> string;
//! }
//!
//! world intent-classifier {
//!     export classifier;
//! }
//! ```
//!
//! ## Future Implementation
//!
//! When implemented, this will:
//! 1. Load a WASM component from the configured path
//! 2. Call the `classify` function via the WIT interface
//! 3. Parse the JSON response into `IntentScore` structs
//! 4. Enforce memory/time limits for sandboxing

use super::{ClassificationResult, IntentClassifier};
use crate::uar::runtime::skills::SkillRegistry;
use async_trait::async_trait;

/// WASM-based intent classifier.
///
/// This is currently a stub that returns an error. When implemented,
/// it will load and call a WASM component for classification.
#[derive(Debug)]
pub struct WasmClassifier {
    /// Path to the WASM component file
    component_path: Option<String>,
}

impl WasmClassifier {
    /// Creates a new WASM classifier.
    ///
    /// # Arguments
    /// * `component_path` - Optional path to the WASM component file
    pub fn new(component_path: Option<String>) -> Self {
        Self { component_path }
    }
}

#[async_trait]
impl IntentClassifier for WasmClassifier {
    async fn classify(
        &self,
        _query: &str,
        _context: &[String],
        _registry: &SkillRegistry,
    ) -> anyhow::Result<ClassificationResult> {
        // Stub implementation - returns error until WASM support is added
        if self.component_path.is_none() {
            return Err(anyhow::anyhow!(
                "WASM classifier not configured: no component_path specified"
            ));
        }

        // TODO: Implement WASM component loading and invocation
        // This would involve:
        // 1. Loading the WASM component using wasmtime or similar
        // 2. Binding the WIT interface
        // 3. Calling the classify function
        // 4. Parsing the JSON response
        //
        // Example pseudocode:
        // ```
        // let engine = Engine::default();
        // let component = Component::from_file(&engine, &path)?;
        // let mut store = Store::new(&engine, ());
        // let instance = Linker::new(&engine).instantiate(&mut store, &component)?;
        // let classify = instance.get_typed_func::<&str, String>(&mut store, "classify")?;
        // let result = classify.call(&mut store, query)?;
        // let scores: Vec<IntentScore> = serde_json::from_str(&result)?;
        // ```

        Err(anyhow::anyhow!(
            "WASM classifier not implemented yet. Component path: {:?}",
            self.component_path
        ))
    }
}

/// Proposed WIT interface for intent classifiers.
///
/// This is the interface that WASM components should implement.
/// Save this as `intent-classifier.wit` when building components.
pub const WIT_INTERFACE: &str = r#"
package intent:classifier@0.1.0;

interface classifier {
    /// Classification result for a single skill/intent.
    record intent-score {
        /// The skill ID or intent label
        label: string,
        /// Confidence score (0.0 to 1.0)
        score: float32,
    }

    /// Result of intent classification.
    record classification-result {
        /// Ranked list of intent scores (highest first)
        scores: list<intent-score>,
        /// Whether the query appears to be out-of-scope
        out-of-scope: bool,
    }

    /// Classifies the input query.
    ///
    /// Arguments:
    /// - input: The user's query text
    /// - context: Optional conversation context (JSON array of strings)
    ///
    /// Returns:
    /// - Classification result with ranked scores
    classify: func(input: string, context: string) -> result<classification-result, string>;
}

world intent-classifier {
    export classifier;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wasm_classifier_not_configured() {
        let classifier = WasmClassifier::new(None);
        let registry = SkillRegistry::default();

        let result = classifier.classify("test query", &[], &registry).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no component_path specified"));
    }

    #[tokio::test]
    async fn test_wasm_classifier_not_implemented() {
        let classifier = WasmClassifier::new(Some("/path/to/classifier.wasm".to_string()));
        let registry = SkillRegistry::default();

        let result = classifier.classify("test query", &[], &registry).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not implemented yet"));
    }
}

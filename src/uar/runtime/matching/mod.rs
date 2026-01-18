pub mod intent;
pub mod tag;
pub mod vector;

#[cfg(feature = "model-build")]
pub mod burn_model;

pub use intent::{
    ClassificationResult, ClassifierBackend, ClassifierConfig, HybridClassifier, IntentClassifier,
    IntentScore, create_classifier,
};
pub use tag::TagMatcher;
pub use vector::VectorMatcher;

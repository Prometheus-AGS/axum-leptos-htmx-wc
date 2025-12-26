//! External NLP Service Client
//!
//! REST client for connecting to external NLP services (SpaCy, Stanza, etc.)
//! following the defined OpenAPI specification.

use super::{ExtractionConfig, RelationshipExtractor};
use crate::uar::domain::{
    graph::{Entity, EntityType, ExtractionResult, Relationship},
    knowledge::KnowledgeChunk,
};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

// =============================================================================
// REST API Request/Response Types
// =============================================================================

/// Request body for the /extract endpoint.
#[derive(Debug, Serialize)]
pub struct ExtractRequest {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ExtractOptions>,
}

/// Options for extraction.
#[derive(Debug, Serialize)]
pub struct ExtractOptions {
    pub extract_entities: bool,
    pub extract_relations: bool,
    pub coreference: bool,
}

impl Default for ExtractOptions {
    fn default() -> Self {
        Self {
            extract_entities: true,
            extract_relations: true,
            coreference: false,
        }
    }
}

/// Response from the /extract endpoint.
#[derive(Debug, Deserialize)]
pub struct ExtractResponse {
    pub entities: Vec<EntityDto>,
    pub relations: Vec<RelationDto>,
}

/// Entity as returned by the external service.
#[derive(Debug, Deserialize)]
pub struct EntityDto {
    pub text: String,
    pub label: String,
    pub start: usize,
    pub end: usize,
    #[serde(default)]
    pub confidence: Option<f32>,
}

/// Relation as returned by the external service.
#[derive(Debug, Deserialize)]
pub struct RelationDto {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    #[serde(default)]
    pub confidence: Option<f32>,
}

// =============================================================================
// External NLP Extractor
// =============================================================================

/// Client for external NLP extraction services.
#[derive(Debug, Clone)]
pub struct ExternalNlpExtractor {
    client: Client,
    base_url: String,
    config: ExtractionConfig,
}

impl ExternalNlpExtractor {
    /// Create a new external NLP extractor.
    ///
    /// # Arguments
    /// * `base_url` - Base URL of the NLP service (e.g., "http://localhost:8080")
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            config: ExtractionConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(base_url: impl Into<String>, config: ExtractionConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            config,
        }
    }

    /// Convert NLP entity label to EntityType.
    fn parse_entity_type(label: &str) -> EntityType {
        match label.to_uppercase().as_str() {
            "PERSON" | "PER" => EntityType::Person,
            "ORG" | "ORGANIZATION" => EntityType::Organization,
            "GPE" | "LOC" | "LOCATION" => EntityType::Location,
            "EVENT" => EntityType::Event,
            "PRODUCT" | "WORK_OF_ART" => EntityType::Product,
            "DATE" | "TIME" => EntityType::Temporal,
            "MONEY" | "PERCENT" | "QUANTITY" | "CARDINAL" | "ORDINAL" => EntityType::Quantity,
            _ => EntityType::Concept,
        }
    }
}

#[async_trait]
impl RelationshipExtractor for ExternalNlpExtractor {
    async fn extract(&self, chunk: &KnowledgeChunk) -> Result<ExtractionResult> {
        self.extract_from_text(&chunk.content).await
    }

    async fn extract_from_text(&self, text: &str) -> Result<ExtractionResult> {
        let url = format!("{}/extract", self.base_url);

        let request = ExtractRequest {
            text: text.to_string(),
            options: Some(ExtractOptions::default()),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to connect to NLP service: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("NLP service returned error: {}", response.status()));
        }

        let extract_response: ExtractResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse NLP response: {}", e))?;

        // Convert to domain types
        let now = chrono::Utc::now().to_rfc3339();

        let entities: Vec<Entity> = extract_response
            .entities
            .into_iter()
            .filter(|e| e.confidence.unwrap_or(1.0) >= self.config.min_confidence)
            .take(self.config.max_entities)
            .map(|e| Entity {
                id: uuid::Uuid::new_v4().to_string(),
                canonical_name: e.text.to_lowercase(),
                entity_type: Self::parse_entity_type(&e.label),
                description: None,
                embedding: Vec::new(), // Would need to embed separately
                source_chunk_ids: Vec::new(),
                created_at: now.clone(),
            })
            .collect();

        // Build entity name -> id mapping for relationships
        let entity_map: std::collections::HashMap<String, String> = entities
            .iter()
            .map(|e| (e.canonical_name.clone(), e.id.clone()))
            .collect();

        let relationships: Vec<Relationship> = extract_response
            .relations
            .into_iter()
            .filter_map(|r| {
                let subject_id = entity_map.get(&r.subject.to_lowercase())?;
                let object_id = entity_map.get(&r.object.to_lowercase())?;
                Some(Relationship {
                    id: uuid::Uuid::new_v4().to_string(),
                    source_id: subject_id.clone(),
                    target_id: object_id.clone(),
                    relation_type: r.predicate.to_lowercase().replace(' ', "_"),
                    weight: r.confidence.unwrap_or(1.0),
                    description: Some(format!("{} {} {}", r.subject, r.predicate, r.object)),
                    source_chunk_id: String::new(),
                    created_at: now.clone(),
                })
            })
            .collect();

        Ok(ExtractionResult {
            entities,
            relationships,
        })
    }

    fn name(&self) -> &'static str {
        "external_nlp"
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_entity_type() {
        assert_eq!(
            ExternalNlpExtractor::parse_entity_type("PERSON"),
            EntityType::Person
        );
        assert_eq!(
            ExternalNlpExtractor::parse_entity_type("ORG"),
            EntityType::Organization
        );
        assert_eq!(
            ExternalNlpExtractor::parse_entity_type("GPE"),
            EntityType::Location
        );
        assert_eq!(
            ExternalNlpExtractor::parse_entity_type("UNKNOWN"),
            EntityType::Concept
        );
    }
}

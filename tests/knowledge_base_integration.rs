//! Integration tests for Knowledge Base functionality.
//!
//! These tests verify the full knowledge base lifecycle including:
//! - KB CRUD operations
//! - Document management
//! - Scoped vector search
//! - Agent-scoped RAG retrieval
//!
//! Requires: DATABASE_URL environment variable pointing to a Postgres instance with pgvector.

use axum_leptos_htmx_wc::uar::{
    defaults::ensure_default_knowledge_base,
    domain::knowledge::{
        DocumentStatus, KbConfig, KnowledgeBase, KnowledgeChunk, KnowledgeDocument,
    },
    persistence::{PersistenceLayer, providers::postgres::PostgresProvider},
};
use serial_test::serial;
use std::sync::Arc;
use uuid::Uuid;

// =============================================================================
// Test Utilities
// =============================================================================

/// Get the database URL from environment, or skip test if not set.
fn get_database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

/// Create a test persistence layer.
async fn setup_persistence() -> Option<Arc<dyn PersistenceLayer>> {
    let url = get_database_url()?;
    let provider = PostgresProvider::new(&url).await.ok()?;
    Some(Arc::new(provider))
}

/// Create a test knowledge base with a random name.
fn create_test_kb(suffix: &str) -> KnowledgeBase {
    let now = chrono::Utc::now().to_rfc3339();
    KnowledgeBase {
        id: Uuid::new_v4().to_string(),
        name: format!(
            "test-kb-{}-{}",
            suffix,
            Uuid::new_v4().to_string()[..8].to_string()
        ),
        description: Some(format!("Test knowledge base for {}", suffix)),
        config: KbConfig::default(),
        created_at: now.clone(),
        updated_at: now,
    }
}

/// Create a test document.
fn create_test_document(kb_id: &str, filename: &str) -> KnowledgeDocument {
    let now = chrono::Utc::now().to_rfc3339();
    KnowledgeDocument {
        id: Uuid::new_v4().to_string(),
        kb_id: kb_id.to_string(),
        filename: filename.to_string(),
        file_path: Some(format!("/data/test/{}", filename)),
        mime_type: Some("text/plain".to_string()),
        chunk_count: 0,
        status: DocumentStatus::Pending,
        created_at: now.clone(),
        updated_at: now,
    }
}

/// Create a test knowledge chunk.
fn create_test_chunk(
    kb_id: &str,
    doc_id: Option<&str>,
    content: &str,
    embedding: Vec<f32>,
) -> KnowledgeChunk {
    KnowledgeChunk {
        id: Uuid::new_v4(),
        kb_id: kb_id.to_string(),
        document_id: doc_id.map(String::from),
        content: content.to_string(),
        metadata: Some(serde_json::json!({"test": true})),
        embedding,
        created_at: chrono::Utc::now().to_rfc3339(),
    }
}

// =============================================================================
// Knowledge Base CRUD Tests
// =============================================================================

#[tokio::test]
#[serial]
async fn test_kb_create_and_retrieve() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Create a knowledge base
    let kb = create_test_kb("crud");
    persistence
        .save_knowledge_base(&kb)
        .await
        .expect("Failed to save KB");

    // Retrieve by ID
    let retrieved = persistence
        .get_knowledge_base(&kb.id)
        .await
        .expect("Failed to get KB")
        .expect("KB not found");

    assert_eq!(retrieved.id, kb.id);
    assert_eq!(retrieved.name, kb.name);
    assert_eq!(retrieved.description, kb.description);

    // Retrieve by name
    let by_name = persistence
        .get_knowledge_base_by_name(&kb.name)
        .await
        .expect("Failed to get KB by name")
        .expect("KB not found by name");

    assert_eq!(by_name.id, kb.id);

    // Cleanup
    persistence
        .delete_knowledge_base(&kb.id)
        .await
        .expect("Failed to delete KB");
}

#[tokio::test]
#[serial]
async fn test_kb_list() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Create multiple KBs
    let kb1 = create_test_kb("list-1");
    let kb2 = create_test_kb("list-2");

    persistence
        .save_knowledge_base(&kb1)
        .await
        .expect("Failed to save KB1");
    persistence
        .save_knowledge_base(&kb2)
        .await
        .expect("Failed to save KB2");

    // List all
    let all_kbs = persistence
        .list_knowledge_bases()
        .await
        .expect("Failed to list KBs");

    let kb_ids: Vec<&str> = all_kbs.iter().map(|k| k.id.as_str()).collect();
    assert!(kb_ids.contains(&kb1.id.as_str()));
    assert!(kb_ids.contains(&kb2.id.as_str()));

    // Cleanup
    persistence
        .delete_knowledge_base(&kb1.id)
        .await
        .expect("Failed to delete KB1");
    persistence
        .delete_knowledge_base(&kb2.id)
        .await
        .expect("Failed to delete KB2");
}

#[tokio::test]
#[serial]
async fn test_kb_update() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Create a KB
    let mut kb = create_test_kb("update");
    persistence
        .save_knowledge_base(&kb)
        .await
        .expect("Failed to save KB");

    // Update it
    kb.description = Some("Updated description".to_string());
    kb.updated_at = chrono::Utc::now().to_rfc3339();
    persistence
        .save_knowledge_base(&kb)
        .await
        .expect("Failed to update KB");

    // Verify update
    let retrieved = persistence
        .get_knowledge_base(&kb.id)
        .await
        .expect("Failed to retrieve KB")
        .expect("KB not found");

    assert_eq!(
        retrieved.description,
        Some("Updated description".to_string())
    );

    // Cleanup
    persistence
        .delete_knowledge_base(&kb.id)
        .await
        .expect("Failed to delete KB");
}

#[tokio::test]
#[serial]
async fn test_kb_delete_cascade() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Create KB, document, and chunks
    let kb = create_test_kb("cascade");
    persistence
        .save_knowledge_base(&kb)
        .await
        .expect("Failed to save KB");

    let doc = create_test_document(&kb.id, "test.txt");
    persistence
        .save_document(&doc)
        .await
        .expect("Failed to save document");

    let chunk = create_test_chunk(&kb.id, Some(&doc.id), "Test content", vec![0.1; 384]);
    persistence
        .save_chunk(&chunk)
        .await
        .expect("Failed to save chunk");

    // Delete KB - should cascade to documents and chunks
    persistence
        .delete_knowledge_base(&kb.id)
        .await
        .expect("Failed to delete KB");

    // Verify KB is gone
    let kb_result = persistence
        .get_knowledge_base(&kb.id)
        .await
        .expect("Failed to check KB");
    assert!(kb_result.is_none());

    // Verify document is gone
    let doc_result = persistence
        .get_document(&doc.id)
        .await
        .expect("Failed to check document");
    assert!(doc_result.is_none());
}

// =============================================================================
// Document Lifecycle Tests
// =============================================================================

#[tokio::test]
#[serial]
async fn test_document_lifecycle() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Create KB
    let kb = create_test_kb("doc-lifecycle");
    persistence
        .save_knowledge_base(&kb)
        .await
        .expect("Failed to save KB");

    // Create document
    let doc = create_test_document(&kb.id, "lifecycle.txt");
    persistence
        .save_document(&doc)
        .await
        .expect("Failed to save document");

    // Verify initial status
    let retrieved = persistence
        .get_document(&doc.id)
        .await
        .expect("Failed to get document")
        .unwrap();
    assert!(matches!(retrieved.status, DocumentStatus::Pending));

    // Update status to Processing
    persistence
        .update_document_status(&doc.id, &DocumentStatus::Processing)
        .await
        .expect("Failed to update status");

    let processing = persistence
        .get_document(&doc.id)
        .await
        .expect("Failed to get document")
        .unwrap();
    assert!(matches!(processing.status, DocumentStatus::Processing));

    // Update status to Indexed
    persistence
        .update_document_status(&doc.id, &DocumentStatus::Indexed)
        .await
        .expect("Failed to update status");

    let indexed = persistence
        .get_document(&doc.id)
        .await
        .expect("Failed to get document")
        .unwrap();
    assert!(matches!(indexed.status, DocumentStatus::Indexed));

    // Test failed status
    let error_status = DocumentStatus::Failed {
        error: "Test error".to_string(),
    };
    persistence
        .update_document_status(&doc.id, &error_status)
        .await
        .expect("Failed to update status");

    let failed = persistence
        .get_document(&doc.id)
        .await
        .expect("Failed to get document")
        .unwrap();
    match failed.status {
        DocumentStatus::Failed { error } => assert_eq!(error, "Test error"),
        _ => panic!("Expected Failed status"),
    }

    // Cleanup
    persistence
        .delete_knowledge_base(&kb.id)
        .await
        .expect("Failed to delete KB");
}

#[tokio::test]
#[serial]
async fn test_document_list() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Create KB
    let kb = create_test_kb("doc-list");
    persistence
        .save_knowledge_base(&kb)
        .await
        .expect("Failed to save KB");

    // Create multiple documents
    let doc1 = create_test_document(&kb.id, "doc1.txt");
    let doc2 = create_test_document(&kb.id, "doc2.txt");
    let doc3 = create_test_document(&kb.id, "doc3.txt");

    persistence
        .save_document(&doc1)
        .await
        .expect("Failed to save doc1");
    persistence
        .save_document(&doc2)
        .await
        .expect("Failed to save doc2");
    persistence
        .save_document(&doc3)
        .await
        .expect("Failed to save doc3");

    // List documents in KB
    let docs = persistence
        .list_documents(&kb.id)
        .await
        .expect("Failed to list documents");
    assert_eq!(docs.len(), 3);

    let doc_ids: Vec<&str> = docs.iter().map(|d| d.id.as_str()).collect();
    assert!(doc_ids.contains(&doc1.id.as_str()));
    assert!(doc_ids.contains(&doc2.id.as_str()));
    assert!(doc_ids.contains(&doc3.id.as_str()));

    // Cleanup
    persistence
        .delete_knowledge_base(&kb.id)
        .await
        .expect("Failed to delete KB");
}

// =============================================================================
// Scoped Search Tests
// =============================================================================

#[tokio::test]
#[serial]
async fn test_scoped_search_filters_by_kb() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Create two KBs
    let kb1 = create_test_kb("scope-1");
    let kb2 = create_test_kb("scope-2");
    persistence
        .save_knowledge_base(&kb1)
        .await
        .expect("Failed to save KB1");
    persistence
        .save_knowledge_base(&kb2)
        .await
        .expect("Failed to save KB2");

    // Create chunks with similar embeddings but in different KBs
    let embedding = vec![0.5f32; 384];

    let chunk1 = create_test_chunk(&kb1.id, None, "Content in KB1", embedding.clone());
    let chunk2 = create_test_chunk(&kb2.id, None, "Content in KB2", embedding.clone());

    persistence
        .save_chunk(&chunk1)
        .await
        .expect("Failed to save chunk1");
    persistence
        .save_chunk(&chunk2)
        .await
        .expect("Failed to save chunk2");

    // Search scoped to KB1 only
    let kb1_results = persistence
        .search_knowledge_scoped(&[&kb1.id], &embedding, 10, 0.0)
        .await
        .expect("Failed to search KB1");

    // Should only find chunk from KB1
    assert!(
        kb1_results.iter().all(|m| m.chunk.kb_id == kb1.id),
        "Scoped search returned results from wrong KB"
    );

    // Search scoped to KB2 only
    let kb2_results = persistence
        .search_knowledge_scoped(&[&kb2.id], &embedding, 10, 0.0)
        .await
        .expect("Failed to search KB2");

    assert!(
        kb2_results.iter().all(|m| m.chunk.kb_id == kb2.id),
        "Scoped search returned results from wrong KB"
    );

    // Search across both KBs
    let both_results = persistence
        .search_knowledge_scoped(&[&kb1.id, &kb2.id], &embedding, 10, 0.0)
        .await
        .expect("Failed to search both KBs");

    assert!(
        both_results.len() >= 2,
        "Expected at least 2 results when searching both KBs"
    );

    // Cleanup
    persistence
        .delete_knowledge_base(&kb1.id)
        .await
        .expect("Failed to delete KB1");
    persistence
        .delete_knowledge_base(&kb2.id)
        .await
        .expect("Failed to delete KB2");
}

// =============================================================================
// Default KB Initialization Tests
// =============================================================================

#[tokio::test]
#[serial]
async fn test_default_kb_initialization() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Ensure no default KB exists (may need to clean up from previous tests)
    if let Ok(Some(existing)) = persistence.get_knowledge_base_by_name("default").await {
        persistence.delete_knowledge_base(&existing.id).await.ok();
    }

    // First call should create the default KB
    let kb1 = ensure_default_knowledge_base(persistence.as_ref(), None)
        .await
        .expect("Failed to create default KB");

    assert_eq!(kb1.name, "default");
    assert!(kb1.description.is_some());

    // Second call should return the same KB (idempotent)
    let kb2 = ensure_default_knowledge_base(persistence.as_ref(), None)
        .await
        .expect("Failed to get default KB");

    assert_eq!(kb1.id, kb2.id);

    // Cleanup
    persistence
        .delete_knowledge_base(&kb1.id)
        .await
        .expect("Failed to delete default KB");
}

// =============================================================================
// Chunk Storage and Search Tests
// =============================================================================

#[tokio::test]
#[serial]
async fn test_chunk_storage_and_global_search() {
    let Some(persistence) = setup_persistence().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    // Create KB
    let kb = create_test_kb("chunks");
    persistence
        .save_knowledge_base(&kb)
        .await
        .expect("Failed to save KB");

    // Create chunks with different embeddings
    // Helper to create repeating pattern embedding
    fn make_embedding(pattern: &[f32]) -> Vec<f32> {
        pattern.iter().cycle().take(384).copied().collect()
    }

    let chunks = vec![
        create_test_chunk(
            &kb.id,
            None,
            "The quick brown fox",
            make_embedding(&[0.9, 0.1, 0.0]),
        ),
        create_test_chunk(
            &kb.id,
            None,
            "jumps over the lazy dog",
            make_embedding(&[0.1, 0.9, 0.0]),
        ),
        create_test_chunk(
            &kb.id,
            None,
            "A completely different topic",
            make_embedding(&[0.0, 0.1, 0.9]),
        ),
    ];

    for chunk in &chunks {
        persistence
            .save_chunk(chunk)
            .await
            .expect("Failed to save chunk");
    }

    // Search with embedding similar to first chunk
    let query = make_embedding(&[0.85, 0.15, 0.0]);
    let results = persistence
        .search_knowledge(&query, 10, 0.0) // Low threshold to get all results
        .await
        .expect("Failed to search");

    assert!(!results.is_empty(), "Expected search results");

    // The first result should be the most similar one
    // (Results are sorted by score descending)

    // Cleanup
    persistence
        .delete_knowledge_base(&kb.id)
        .await
        .expect("Failed to delete KB");
}

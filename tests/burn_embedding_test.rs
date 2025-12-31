use axum_leptos_htmx_wc::uar::runtime::matching::vector::VectorMatcher;

#[tokio::test]
async fn test_burn_embedding_initialization() {
    let matcher = VectorMatcher::new(0.7);
    // Initialize (should trigger model loading)
    let res = matcher.initialize().await;
    assert!(
        res.is_ok(),
        "Failed to initialize BurnVectorMatcher: {:?}",
        res.err()
    );
}

#[tokio::test]
async fn test_burn_embedding_shape() {
    let matcher = VectorMatcher::new(0.7);
    matcher.initialize().await.expect("Initialization failed");

    let texts = vec!["Hello world".to_string(), "Burn is great".to_string()];
    let embeddings = matcher.embed_batch(texts).await.expect("Embedding failed");

    assert_eq!(
        embeddings.len(),
        2,
        "Should return embeddings for all inputs"
    );
    assert_eq!(
        embeddings[0].len(),
        384,
        "Embedding dimension should be 384 for BG-Small-En-V1.5"
    );
}

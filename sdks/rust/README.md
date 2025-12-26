# axum-leptos-htmx-wc Rust SDK

Rust SDK for axum-leptos-htmx-wc - HTTP client and embeddable runtime.

## Features

- **http-client** (default): HTTP client for remote API calls
- **embedded**: Embed the full runtime in your Rust application
- **full**: Both features enabled

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
axum-leptos-htmx-wc-sdk = "0.1"
```

For embedded runtime:

```toml
[dependencies]
axum-leptos-htmx-wc-sdk = { version = "0.1", features = ["embedded"] }
```

## HTTP Client Usage

```rust
use axum_leptos_htmx_wc_sdk::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("http://localhost:3000")?;
    
    // Chat API
    let response = client.chat().send("Hello!").await?;
    println!("Session: {}", response.session_id);
    
    // Knowledge Base API
    let kbs = client.knowledge().list().await?;
    for kb in kbs {
        println!("KB: {} ({})", kb.name, kb.id);
    }
    
    // Search
    let results = client.knowledge().search("kb-id", "query").await?;
    for result in results.results {
        println!("Score: {:.2} - {}", result.score, result.content);
    }
    
    Ok(())
}
```

## Embedded Runtime Usage

```rust
use axum_leptos_htmx_wc_sdk::Runtime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Runtime::builder()
        .config_path("config.yaml")
        .build()
        .await?;
    
    // Start the full HTTP server
    runtime.start().await?;
    
    Ok(())
}
```

## License

MIT OR Apache-2.0

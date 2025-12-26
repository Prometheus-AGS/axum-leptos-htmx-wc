pub mod adapters;
pub mod ingest;
pub mod memory;
pub mod openai;
pub mod routes;
pub mod sse;
pub mod upload;

use axum::Router;

use crate::uar::runtime::manager::RunManager;
use std::sync::Arc;

pub fn router() -> Router<Arc<RunManager>> {
    // In M3 we will build the router in routes.rs and just return it here
    routes::build_router()
}

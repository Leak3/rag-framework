use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod routes;
pub mod store;
pub mod config;
pub mod providers;
pub mod ingest;
pub mod error;

pub struct AppState {
    pub store: RwLock<store::VectorStore>,
    pub config: config::Config,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::health::health,
        routes::upload::upload,
        routes::query::query,
        routes::delete::delete,
    ),
    components(
        schemas(
            routes::health::HealthResponse,
            routes::upload::UploadRequest,
            routes::upload::UploadResponse,
            routes::query::QueryRequest,
            routes::query::QueryResponse,
            routes::delete::DeleteResponse,
            error::Error,
        )
    )
)]
struct ApiDoc;

pub fn build_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(routes::health::health))
        .route(
            "/upload",
            post(routes::upload::upload).layer(axum::extract::DefaultBodyLimit::disable()),
        )
        .route("/query", post(routes::query::query))
        .route("/delete/{source}", delete(routes::delete::delete))
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
        .layer(CorsLayer::permissive())
}

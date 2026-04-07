use axum::{routing::{get, post, delete}, Router};
use tokio::net::TcpListener;
use std::sync::Arc;
use tokio::sync::RwLock;

mod routes;
mod store;
mod config;
mod providers;
mod ingest;
mod error;

pub struct AppState {
    pub store: RwLock<store::VectorStore>,
    pub config: config::Config,
}

#[tokio::main]
async fn main() {
    let config = config::load_config();
    let store = RwLock::new(store::VectorStore::load(&config.store_path));

    let state = Arc::new(AppState { store, config });

    let app = Router::new()
                .route("/health", get(routes::health::health))
                .route("/upload", post(routes::upload::upload)
                    .layer(axum::extract::DefaultBodyLimit::disable()))
                .route("/query", post(routes::query::query))
                .route("/delete/{source}", delete(routes::delete::delete))
                .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on port 3000");
    axum::serve(listener, app).await.unwrap();
}

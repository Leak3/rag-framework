use tokio::net::TcpListener;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config = rag_framework::config::load_config();
    let store = tokio::sync::RwLock::new(rag_framework::store::VectorStore::load(
        &config.storage.store_path,
    ));

    let state = Arc::new(rag_framework::AppState { store, config });
    let app = rag_framework::build_app(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on port 3000");
    axum::serve(listener, app).await.unwrap();
}

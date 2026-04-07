use axum::body::Body;
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;

fn test_state() -> Arc<rag_framework::AppState> {
    Arc::new(rag_framework::AppState {
        store: tokio::sync::RwLock::new(rag_framework::store::VectorStore::new()),
        config: rag_framework::config::Config {
            models: rag_framework::config::ModelsConfig {
                llm_model: "ai/ministral3".to_string(),
                embedding_model: "ai/mxbai-embed-large".to_string(),
                api_url: "http://localhost:12434/engines/{engine}/v1".to_string(),
                engine: Some("llama.cpp".to_string()),
                chunking_model: Some("ai/ministral3".to_string()),
            },
            chunking: rag_framework::config::ChunkingConfig {
                chunk_size: 500,
                chunk_overlap: 2,
                smart: false,
                smart_max_chars: Some(4000),
            },
            retrieval: rag_framework::config::RetrievalConfig {
                top_k: 10,
                final_k: 8,
            },
            storage: rag_framework::config::StorageConfig {
                store_path: "store.json".to_string(),
            },
        },
    })
}

#[tokio::test]
async fn health_returns_ok_json() {
    let app = rag_framework::build_app(test_state());
    let res = app
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), axum::http::StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("\"status\":true"));
}

#[tokio::test]
async fn openapi_is_served() {
    let app = rag_framework::build_app(test_state());
    let res = app
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api-docs/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), axum::http::StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("\"openapi\""));
    assert!(body_str.contains("\"/health\""));
    assert!(body_str.contains("\"/query\""));
}


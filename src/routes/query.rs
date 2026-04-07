use axum::extract::State;
use std::sync::Arc;
use std::collections::HashMap;

use crate::AppState;

#[derive(serde::Deserialize)]
pub struct QueryRequest {
    question: String,
}

#[derive(serde::Serialize)]
pub struct QueryResponse {
    answer: String,
    sources: Vec<String>,
}

pub async fn query(
    State(state): State<Arc<AppState>>,
    axum::Json(payload): axum::Json<QueryRequest>,
) -> Result<axum::Json<QueryResponse>, crate::error::Error> {
    println!("[query] question: {}", payload.question);

    let query_embedding = crate::providers::embeddings::embed(&payload.question).await?;
    let chunks = hybrid_search(&state, &payload.question, &query_embedding).await;
    let prompt = build_prompt(&chunks.join("\n\n"), &payload.question);
    let answer = crate::providers::llm::ask(prompt).await?;

    println!("[query] done");
    Ok(axum::Json(QueryResponse { answer, sources: chunks }))
}

async fn hybrid_search(state: &AppState, question: &str, query_embedding: &[f32]) -> Vec<String> {
    let top_k = state.config.top_k;
    let final_k = state.config.final_k;

    let vector_rank = state.store.read().await.vector_search(query_embedding, top_k);
    let bm25_rank = state.store.read().await.bm25_search(question, top_k);

    let mut rrf_scores: HashMap<String, f32> = HashMap::new();
    for (rank, chunk) in vector_rank.iter().enumerate() {
        *rrf_scores.entry(chunk.clone()).or_insert(0.0) += 1.0 / (60.0 + rank as f32 + 1.0);
    }
    for (rank, chunk) in bm25_rank.iter().enumerate() {
        *rrf_scores.entry(chunk.clone()).or_insert(0.0) += 1.0 / (60.0 + rank as f32 + 1.0);
    }

    let mut ranked: Vec<(String, f32)> = rrf_scores.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    ranked.truncate(final_k);
    ranked.into_iter().map(|(chunk, _)| chunk).collect()
}

fn build_prompt(context: &str, question: &str) -> String {
    format!(
        "You are a helpful assistant. Use the context below to answer the question naturally and conversationally. If the context doesn't contain the answer, say so.\n\nContext:\n{}\n\nQuestion: {}",
        context, question
    )
}
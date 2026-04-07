use axum::extract::State;
use std::sync::Arc;

use crate::AppState;

#[derive(serde::Serialize)]
pub struct UploadResponse {
    chunk_count: usize,
    chunks: Vec<String>,
}

pub async fn upload(
    State(state): State<Arc<AppState>>,
    mut multipart: axum::extract::Multipart,
) -> Result<axum::Json<UploadResponse>, crate::error::Error> {
    let mut chunks = Vec::<String>::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        println!("[upload] processing file: {}", file_name);

        let text = if file_name.ends_with(".pdf") {
            let data = field.bytes().await.unwrap();
            println!("[upload] extracting text from PDF ({} bytes)", data.len());
            crate::ingest::pdf::extract_text(data.to_vec()).await.unwrap_or_default()
        } else {
            field.text().await.unwrap()
        };

        let field_chunks = crate::ingest::chunker::chunk_text(&text, state.config.chunk_size, state.config.chunk_overlap);
        println!("[upload] {} chunks generated", field_chunks.len());

        for (i, chunk) in field_chunks.into_iter().enumerate() {
            let embedding = crate::providers::embeddings::embed(&chunk).await?;
            chunks.push(chunk.clone());
            state.store.write().await.add(chunk, embedding, &file_name);
            println!("[upload] embedded chunk {}", i + 1);
        }
    }

    println!("[upload] saving store");
    state.store.read().await.save(&state.config.store_path);

    Ok(axum::Json(UploadResponse {
        chunk_count: chunks.len(),
        chunks,
    }))
}

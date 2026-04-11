use axum::extract::State;
use std::sync::Arc;

use crate::AppState;

#[derive(utoipa::ToSchema)]
pub struct UploadRequest {
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
}

#[derive(serde::Serialize)]
#[derive(utoipa::ToSchema)]
pub struct UploadResponse {
    chunk_count: usize,
    chunks: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/upload",
    request_body(
        content = UploadRequest,
        description = "Multipart form upload (field name: file)",
        content_type = "multipart/form-data"
    ),
    responses(
        (status = 200, description = "Uploaded and embedded", body = UploadResponse),
        (status = 500, description = "Internal error", body = crate::error::Error)
    )
)]
pub async fn upload(
    State(state): State<Arc<AppState>>,
    mut multipart: axum::extract::Multipart,
) -> Result<axum::Json<UploadResponse>, crate::error::Error> {
    let mut chunks = Vec::<String>::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        println!("[upload] processing file: {}", file_name);

        let data = field.bytes().await.unwrap().to_vec();
        let upload_req = UploadRequest { file: data };
        if upload_req.file.is_empty() {
            return Err(crate::error::Error::new(400, "empty upload"));
        }

        let text = if file_name.ends_with(".pdf") {
            println!("[upload] extracting text from PDF ({} bytes)", upload_req.file.len());
            crate::ingest::pdf::extract_text(upload_req.file).await.unwrap_or_default()
        } else {
            String::from_utf8_lossy(&upload_req.file).to_string()
        };

        let field_chunks = crate::ingest::chunker::chunk_document(&state.config, &text).await;
        println!("[upload] {} chunks generated", field_chunks.len());

        for (i, chunk) in field_chunks.into_iter().enumerate() {
            let embedding = crate::providers::embeddings::embed(&state.config, &chunk).await?;
            chunks.push(chunk.clone());
            state.store.write().await.add(chunk, embedding, &file_name);
            println!("[upload] embedded chunk {}", i + 1);
        }
    }

    println!("[upload] saving store");
    state.store.read().await.save(&state.config.storage.store_path);

    Ok(axum::Json(UploadResponse {
        chunk_count: chunks.len(),
        chunks,
    }))
}

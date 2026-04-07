use axum::extract::State;
use std::sync::Arc;

use crate::AppState;

#[derive(serde::Serialize)]
pub struct DeleteResponse {
    success: bool,
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(source): axum::extract::Path<String>,
) -> axum::Json<DeleteResponse> {
    println!("[delete] removing source: {}", source);
    state.store.write().await.delete_by_source(&source);
    state.store.read().await.save(&state.config.store_path);
    println!("[delete] done");

    axum::Json(DeleteResponse { success: true })
}

use axum::extract::State;
use std::sync::Arc;

use crate::AppState;

#[derive(serde::Serialize)]
#[derive(utoipa::ToSchema)]
pub struct DeleteResponse {
    success: bool,
}

#[utoipa::path(
    delete,
    path = "/delete/{source}",
    params(
        ("source" = String, Path, description = "Document source identifier (e.g. filename)")
    ),
    responses(
        (status = 200, description = "Deleted documents for source", body = DeleteResponse)
    )
)]
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

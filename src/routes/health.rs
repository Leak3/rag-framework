#[derive(serde::Serialize)]
pub struct HealthResponse {
    message: &'static str,
    status: bool,
}

pub async fn health() -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse {
        message: "Healthy",
        status: true,
    })
}

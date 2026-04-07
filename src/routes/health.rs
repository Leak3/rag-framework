#[derive(serde::Serialize)]
#[derive(utoipa::ToSchema)]
pub struct HealthResponse {
    message: &'static str,
    status: bool,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service health", body = HealthResponse)
    )
)]
pub async fn health() -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse {
        message: "Healthy",
        status: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_handler_returns_expected_payload() {
        let resp = health().await;
        assert_eq!(resp.0.message, "Healthy");
        assert!(resp.0.status);
    }
}

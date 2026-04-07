use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;

#[derive(serde::Serialize)]
#[derive(utoipa::ToSchema)]
pub struct Error{
    code: u16,
    message: String,
    time: u64,
}

impl Error {
    pub fn new(code: u16, message: &str) -> Self {
        Error {
            code,
            message: message.to_string(),
            time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::new(500, &e.to_string())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, axum::Json(self)).into_response()
    }
}

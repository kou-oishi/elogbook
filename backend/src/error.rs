use actix_web::{http::StatusCode, HttpResponse, ResponseError};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    NotFound(String),
    #[error("database error: {0}")]
    Database(#[from] mongodb::error::Error),
    #[error("multipart error: {0}")]
    Multipart(#[from] actix_multipart::MultipartError),
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Database(_) | Self::Multipart(_) | Self::Io(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

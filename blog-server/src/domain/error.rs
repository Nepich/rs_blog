use thiserror::Error;
use actix_web::{error::ResponseError, HttpResponse};
use tonic::Status;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Post not found")]
    PostNotFound,

    #[error("Forbidden")]
    Forbidden,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl ResponseError for DomainError {
    fn error_response(&self) -> HttpResponse {
        match self {
            DomainError::UserAlreadyExists => {
                HttpResponse::Conflict().body("User already exists")
            }
            DomainError::InvalidCredentials => {
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
            DomainError::PostNotFound => {
                HttpResponse::NotFound().finish()
            }
            DomainError::Forbidden => {
                HttpResponse::Forbidden().finish()
            }
            _ => HttpResponse::InternalServerError().body(self.to_string()),
        }
    }
}

impl From<DomainError> for Status {
    fn from(error: DomainError) -> Self {
        match error {
            DomainError::UserAlreadyExists => Status::already_exists(error.to_string()),
            DomainError::InvalidCredentials => Status::unauthenticated(error.to_string()),
            DomainError::PostNotFound => Status::not_found(error.to_string()),
            DomainError::Forbidden => Status::permission_denied(error.to_string()),
            _ => Status::internal(error.to_string()),
        }
    }
}

use thiserror::Error;

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

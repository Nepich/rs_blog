use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlogClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("gRPC error: {0}")]
    Grpc(tonic::Status),

    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

impl From<tonic::Status> for BlogClientError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::NotFound => BlogClientError::NotFound,
            tonic::Code::Unauthenticated | tonic::Code::PermissionDenied => BlogClientError::Unauthorized,
            _ => BlogClientError::Grpc(status),
        }
    }
}

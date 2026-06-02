pub mod error;
pub mod grpc_client;
pub mod http_client;

pub mod proto {
    tonic::include_proto!("grpc");
}

pub use error::BlogClientError;

use crate::grpc_client::GrpcBlogClient;
use crate::http_client::HttpBlogClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: Option<String>,
    pub user: Option<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: u64,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum Transport {
    Http(String),
    Grpc(String),
}

pub struct BlogClient {
    transport: Transport,
    http_client: Option<HttpBlogClient>,
    grpc_client: Option<GrpcBlogClient>,
    token: Option<String>,
}

impl BlogClient {
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        let mut client = BlogClient {
            transport: transport.clone(),
            http_client: None,
            grpc_client: None,
            token: None,
        };

        match transport {
            Transport::Http(base_url) => {
                client.http_client = Some(HttpBlogClient::new(base_url));
            }
            Transport::Grpc(endpoint) => {
                client.grpc_client = Some(GrpcBlogClient::connect(endpoint).await?);
            }
        }

        Ok(client)
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        let auth = match &mut self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .expect("HTTP client missing")
                    .register(username, email, password)
                    .await?
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .expect("gRPC client missing")
                    .register(username, email, password)
                    .await?
            }
        };

        if let Some(token) = auth.token.as_ref() {
            self.token = Some(token.clone());
        }

        Ok(auth)
    }

    pub async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        let auth = match &mut self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .expect("HTTP client missing")
                    .login(username, password)
                    .await?
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .expect("gRPC client missing")
                    .login(username, password)
                    .await?
            }
        };

        if let Some(token) = auth.token.as_ref() {
            self.token = Some(token.clone());
        }

        Ok(auth)
    }

    pub async fn create_post(
        &mut self,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let token = self.token.as_ref().ok_or(BlogClientError::Unauthorized)?;
        match &mut self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .expect("HTTP client missing")
                    .create_post(token, title, content)
                    .await
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .expect("gRPC client missing")
                    .create_post(token, title, content)
                    .await
            }
        }
    }

    pub async fn get_post(&mut self, id: u64) -> Result<Post, BlogClientError> {
        match &mut self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .expect("HTTP client missing")
                    .get_post(id)
                    .await
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .expect("gRPC client missing")
                    .get_post(id)
                    .await
            }
        }
    }

    pub async fn update_post(
        &mut self,
        id: u64,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let token = self.token.as_ref().ok_or(BlogClientError::Unauthorized)?;
        match &mut self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .expect("HTTP client missing")
                    .update_post(token, id, title, content)
                    .await
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .expect("gRPC client missing")
                    .update_post(token, id, title, content)
                    .await
            }
        }
    }

    pub async fn delete_post(&mut self, id: u64) -> Result<(), BlogClientError> {
        let token = self.token.as_ref().ok_or(BlogClientError::Unauthorized)?;
        match &mut self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .expect("HTTP client missing")
                    .delete_post(token, id)
                    .await
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .expect("gRPC client missing")
                    .delete_post(token, id)
                    .await
            }
        }
    }

    pub async fn list_posts(
        &mut self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Post>, BlogClientError> {
        match &mut self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .expect("HTTP client missing")
                    .list_posts(limit, offset)
                    .await
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_mut()
                    .expect("gRPC client missing")
                    .list_posts(limit, offset)
                    .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blog_client_http_new_sets_no_token() {
        let client = BlogClient::new(Transport::Http("http://localhost:8080".to_string()))
            .await
            .expect("failed to create HTTP blog client");
        assert!(client.get_token().is_none());
    }

    #[test]
    fn test_http_blog_client_url_builder() {
        let client = http_client::HttpBlogClient::new("http://localhost:8080/".to_string());
        assert_eq!(client.url("/posts"), "http://localhost:8080/posts");
        assert_eq!(client.url("posts"), "http://localhost:8080/posts");
        assert_eq!(
            client.url("http://example.com/posts"),
            "http://localhost:8080/http://example.com/posts"
        );
    }

    #[test]
    fn test_blog_client_error_from_grpc_status() {
        let err = BlogClientError::from(tonic::Status::not_found("not found"));
        assert!(matches!(err, BlogClientError::NotFound));

        let err = BlogClientError::from(tonic::Status::unauthenticated("unauthorized"));
        assert!(matches!(err, BlogClientError::Unauthorized));

        let err = BlogClientError::from(tonic::Status::internal("internal error"));
        match err {
            BlogClientError::Grpc(status) => assert_eq!(status.code(), tonic::Code::Internal),
            _ => panic!("expected BlogClientError::Grpc"),
        }
    }
}

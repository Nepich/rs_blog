use crate::{AuthResponse, BlogClientError, Post, User};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct HttpBlogClient {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct PostRequest {
    title: String,
    content: String,
}

#[derive(Serialize)]
struct PostUpdateRequest {
    title: String,
    content: String,
}

#[derive(Deserialize)]
struct AuthRegisterResponse {
    id: u64,
    email: String,
    token: Option<String>,
}

#[derive(Deserialize)]
struct AuthLoginResponse {
    token: String,
}

#[derive(Deserialize)]
struct HttpPostResponse {
    id: u64,
    title: String,
    content: String,
}

#[derive(Deserialize)]
struct HttpPostsResponse {
    posts: Vec<HttpPostResponse>,
}

impl HttpBlogClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url.trim_end_matches('/'), path.trim_start_matches('/'))
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<T, BlogClientError> {
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return match status {
                StatusCode::UNAUTHORIZED => Err(BlogClientError::Unauthorized),
                StatusCode::NOT_FOUND => Err(BlogClientError::NotFound),
                StatusCode::BAD_REQUEST | StatusCode::CONFLICT => {
                    Err(BlogClientError::InvalidRequest(text))
                }
                _ => Err(BlogClientError::InvalidRequest(text)),
            };
        }

        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<()>() {
            let _ = response.text().await.unwrap_or_default();
            return Ok(serde_json::from_str("null").unwrap());
        }

        Ok(response.json().await?)
    }

    pub async fn register(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        let request = RegisterRequest {
            username,
            email,
            password,
        };

        let response = self
            .client
            .post(self.url("/auth/register"))
            .json(&request)
            .send()
            .await?;

        let body: AuthRegisterResponse = Self::handle_response(response).await?;
        Ok(AuthResponse {
            token: body.token,
            user: Some(User {
                id: body.id,
                email: body.email,
            }),
        })
    }

    pub async fn login(&self, username: String, password: String) -> Result<AuthResponse, BlogClientError> {
        let request = LoginRequest { username, password };
        let response = self
            .client
            .post(self.url("/auth/login"))
            .json(&request)
            .send()
            .await?;

        let body: AuthLoginResponse = Self::handle_response(response).await?;
        Ok(AuthResponse {
            token: Some(body.token),
            user: None,
        })
    }

    pub async fn create_post(
        &self,
        token: &str,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let request = PostRequest { title, content };
        let response = self
            .client
            .post(self.url("/posts"))
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?;

        let body: HttpPostResponse = Self::handle_response(response).await?;
        Ok(Post {
            id: body.id,
            title: body.title,
            content: body.content,
        })
    }

    pub async fn get_post(&self, id: u64) -> Result<Post, BlogClientError> {
        let response = self.client.get(self.url(&format!("/posts/{id}"))).send().await?;

        let body: HttpPostResponse = Self::handle_response(response).await?;
        Ok(Post {
            id: body.id,
            title: body.title,
            content: body.content,
        })
    }

    pub async fn update_post(
        &self,
        token: &str,
        id: u64,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let request = PostUpdateRequest { title, content };
        let response = self
            .client
            .put(self.url(&format!("/posts/{id}")))
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?;

        let body: HttpPostResponse = Self::handle_response(response).await?;
        Ok(Post {
            id: body.id,
            title: body.title,
            content: body.content,
        })
    }

    pub async fn delete_post(&self, token: &str, id: u64) -> Result<(), BlogClientError> {
        let response = self
            .client
            .delete(self.url(&format!("/posts/{id}")))
            .bearer_auth(token)
            .send()
            .await?;

        Self::handle_response::<()>(response).await?;
        Ok(())
    }

    pub async fn list_posts(&self, limit: u32, offset: u32) -> Result<Vec<Post>, BlogClientError> {
        let response = self
            .client
            .get(self.url(&format!("/posts?limit={limit}&offset={offset}")))
            .send()
            .await?;

        let body: HttpPostsResponse = Self::handle_response(response).await?;
        Ok(body
            .posts
            .into_iter()
            .map(|item| Post {
                id: item.id,
                title: item.title,
                content: item.content,
            })
            .collect())
    }
}

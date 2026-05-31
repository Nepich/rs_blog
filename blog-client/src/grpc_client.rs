use crate::{AuthResponse, BlogClientError, Post, User};
use tonic::{metadata::MetadataValue, Request};

pub struct GrpcBlogClient {
    inner: crate::proto::blog_service_client::BlogServiceClient<tonic::transport::Channel>,
}

impl GrpcBlogClient {
    pub async fn connect(endpoint: String) -> Result<Self, BlogClientError> {
        let channel = tonic::transport::Channel::from_shared(endpoint)
            .map_err(|err| BlogClientError::InvalidRequest(err.to_string()))?;
        let client = crate::proto::blog_service_client::BlogServiceClient::new(channel.connect().await?);
        Ok(Self { inner: client })
    }

    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<AuthResponse, BlogClientError> {
        let request = crate::proto::RegisterRequest {
            username,
            email,
            password,
        };

        let response = self
            .inner
            .register(Request::new(request))
            .await
            .map_err(BlogClientError::from)?;

        let payload = response.into_inner();
        Ok(AuthResponse {
            token: None,
            user: Some(User {
                id: payload.id,
                email: payload.email,
            }),
        })
    }

    pub async fn login(&mut self, email: String, password: String) -> Result<AuthResponse, BlogClientError> {
        let request = crate::proto::LoginRequest { email, password };
        let response = self
            .inner
            .login(Request::new(request))
            .await
            .map_err(BlogClientError::from)?;

        let payload = response.into_inner();
        Ok(AuthResponse {
            token: Some(payload.token),
            user: None,
        })
    }

    pub async fn create_post(
        &mut self,
        token: &str,
        title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let mut request = Request::new(crate::proto::PostCreateRequest { title, content });
        request.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from(format!("Bearer {token}"))
                .map_err(|err| BlogClientError::InvalidRequest(err.to_string()))?,
        );

        let response = self
            .inner
            .create_post(request)
            .await
            .map_err(BlogClientError::from)?;

        let payload = response.into_inner();
        Ok(Post {
            id: payload.id,
            title: payload.title,
            content: payload.content,
        })
    }

    pub async fn get_post(&mut self, id: u64) -> Result<Post, BlogClientError> {
        let response = self
            .inner
            .get_post(Request::new(crate::proto::PostGetRequest { id }))
            .await
            .map_err(BlogClientError::from)?;

        let payload = response.into_inner();
        Ok(Post {
            id: payload.id,
            title: payload.title,
            content: payload.content,
        })
    }

    pub async fn update_post(
        &mut self,
        token: &str,
        id: u64,
        _title: String,
        content: String,
    ) -> Result<Post, BlogClientError> {
        let mut request = Request::new(crate::proto::PostUpdateRequest { id, content });
        request.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from(format!("Bearer {token}"))
                .map_err(|err| BlogClientError::InvalidRequest(err.to_string()))?,
        );

        let response = self
            .inner
            .update_post(request)
            .await
            .map_err(BlogClientError::from)?;

        let payload = response.into_inner();
        Ok(Post {
            id: payload.id,
            title: payload.title,
            content: payload.content,
        })
    }

    pub async fn delete_post(&mut self, token: &str, id: u64) -> Result<(), BlogClientError> {
        let mut request = Request::new(crate::proto::PostDeleteRequest { id });
        request.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from(format!("Bearer {token}"))
                .map_err(|err| BlogClientError::InvalidRequest(err.to_string()))?,
        );

        self.inner
            .delete_post(request)
            .await
            .map_err(BlogClientError::from)?;
        Ok(())
    }

    pub async fn list_posts(&mut self, limit: u32, offset: u32) -> Result<Vec<Post>, BlogClientError> {
        let page_size = limit.max(1);
        let page = if limit == 0 { 1 } else { offset / limit + 1 };

        let response = self
            .inner
            .list_posts(Request::new(crate::proto::PostsGetRequest { page, page_size }))
            .await
            .map_err(BlogClientError::from)?;

        let payload = response.into_inner();
        Ok(payload
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

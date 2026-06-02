use std::sync::Arc;

use tonic::{Request, Response, Status, metadata::MetadataMap};

use crate::grpc;
use crate::application::{auth_service::AuthService, blog_service::BlogService};
use crate::infrastructure::jwt::{Claims, JwtService};

#[derive(Clone)]
pub struct BlogGrpcService {
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
    jwt_service: Arc<JwtService>,
}

impl BlogGrpcService {
    pub fn new(
        auth_service: Arc<AuthService>,
        blog_service: Arc<BlogService>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            auth_service,
            blog_service,
            jwt_service,
        }
    }

    fn parse_bearer_token(&self, metadata: &MetadataMap) -> Result<String, Status> {
        let auth_header = metadata
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Authorization header missing"))?;

        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            Ok(token.to_string())
        } else {
            Err(Status::unauthenticated("Bearer token required"))
        }
    }

    fn authorize(&self, metadata: &MetadataMap) -> Result<Claims, Status> {
        let token = self.parse_bearer_token(metadata)?;
        self.jwt_service
            .verify_token(&token)
            .map_err(|_| Status::unauthenticated("Invalid token"))
    }
}

#[tonic::async_trait]
impl grpc::blog_service_server::BlogService for BlogGrpcService {
    async fn create_post(
        &self,
        request: Request<grpc::PostCreateRequest>,
    ) -> Result<Response<grpc::PostResponse>, Status> {
        let claims = self.authorize(request.metadata())?;
        let body = request.into_inner();
        let post = self
            .blog_service
            .create_post(
                claims.user_id,
                crate::domain::post::PostCreateRequest {
                    title: body.title,
                    content: body.content,
                },
            )
            .await
            .map_err(|err| Status::from(err))?;

        Ok(Response::new(grpc::PostResponse {
            id: post.id as u64,
            title: post.title,
            content: post.content,
        }))
    }

    async fn get_post(
        &self,
        request: Request<grpc::PostGetRequest>,
    ) -> Result<Response<grpc::PostResponse>, Status> {
        let body = request.into_inner();
        let post = self
            .blog_service
            .get_post(body.id as i64)
            .await
            .map_err(|err| Status::from(err))?;

        Ok(Response::new(grpc::PostResponse {
            id: post.id as u64,
            title: post.title,
            content: post.content,
        }))
    }

    async fn list_posts(
        &self,
        request: Request<grpc::PostsGetRequest>,
    ) -> Result<Response<grpc::PostsResponse>, Status> {
        let body = request.into_inner();
        let page_size = if body.page_size == 0 {
            10
        } else {
            body.page_size
        };
        let page = if body.page == 0 { 1 } else { body.page };
        let offset = (page.saturating_sub(1) as i64) * page_size as i64;

        let (posts, _) = self
            .blog_service
            .list_posts(page_size as i64, offset)
            .await
            .map_err(|err| Status::from(err))?;

        let response_posts = posts
            .into_iter()
            .map(|post| grpc::PostResponse {
                id: post.id as u64,
                title: post.title,
                content: post.content,
            })
            .collect();

        Ok(Response::new(grpc::PostsResponse {
            posts: response_posts,
        }))
    }

    async fn update_post(
        &self,
        request: Request<grpc::PostUpdateRequest>,
    ) -> Result<Response<grpc::PostResponse>, Status> {
        let claims = self.authorize(request.metadata())?;
        let body = request.into_inner();

        let current_post =
            self.blog_service
                .get_post(body.id as i64)
                .await
                .map_err(|err| Status::from(err))?;

        let post = self
            .blog_service
            .update_post(
                claims.user_id,
                body.id as i64,
                crate::domain::post::PostUpdateRequest {
                    title: current_post.title,
                    content: body.content,
                },
            )
            .await
            .map_err(|err| Status::from(err))?;

        Ok(Response::new(grpc::PostResponse {
            id: post.id as u64,
            title: post.title,
            content: post.content,
        }))
    }

    async fn delete_post(
        &self,
        request: Request<grpc::PostDeleteRequest>,
    ) -> Result<Response<grpc::DeleteResponse>, Status> {
        let claims = self.authorize(request.metadata())?;
        let body = request.into_inner();

        self.blog_service
            .delete_post(claims.user_id, body.id as i64)
            .await
            .map_err(|err| Status::from(err))?;

        Ok(Response::new(grpc::DeleteResponse { success: true }))
    }

    async fn login(
        &self,
        request: Request<grpc::LoginRequest>,
    ) -> Result<Response<grpc::LoginResponse>, Status> {
        let body = request.into_inner();
        let (_, token) = self
            .auth_service
            .login(body.username.clone(), body.password.clone())
            .await
            .map_err(|err| Status::from(err))?;

        Ok(Response::new(grpc::LoginResponse { token }))
    }

    async fn register(
        &self,
        request: Request<grpc::RegisterRequest>,
    ) -> Result<Response<grpc::RegisterResponse>, Status> {
        let body = request.into_inner();
        let (user, _token) = self
            .auth_service
            .register(crate::domain::user::RegisterRequest {
                username: body.username,
                email: body.email,
                password: body.password,
            })
            .await
            .map_err(|err| Status::from(err))?;

        Ok(Response::new(grpc::RegisterResponse {
            id: user.id as u64,
            email: user.email,
        }))
    }

    async fn logout(
        &self,
        request: Request<grpc::LogoutRequest>,
    ) -> Result<Response<grpc::LogoutResponse>, Status> {
        let body = request.into_inner();
        self.jwt_service
            .verify_token(&body.token)
            .map_err(|_| Status::unauthenticated("Invalid token"))?;
        Ok(Response::new(grpc::LogoutResponse { success: true }))
    }
}

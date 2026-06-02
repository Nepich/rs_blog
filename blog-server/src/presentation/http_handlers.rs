use std::sync::Arc;

use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use serde::Deserialize;

use crate::application::{auth_service::AuthService, blog_service::BlogService};
use crate::domain::post::{PostCreateRequest, PostUpdateRequest};
use crate::domain::user::{LoginRequest, RegisterRequest};
use crate::presentation::AuthenticatedUser;

#[derive(Deserialize)]
pub struct PostsQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(serde::Serialize)]
struct RegisterResponse {
    id: u64,
    email: String,
    token: Option<String>,
}

#[derive(serde::Serialize)]
struct LoginResponse {
    token: String,
}

#[derive(serde::Serialize)]
struct PostsListResponse {
    posts: Vec<crate::domain::post::Post>,
    total: i64,
    limit: u32,
    offset: u32,
}

fn get_authenticated_user(req: &HttpRequest) -> Option<AuthenticatedUser> {
    req.extensions().get::<AuthenticatedUser>().cloned()
}

pub async fn register(
    auth_service: web::Data<Arc<AuthService>>,
    payload: web::Json<RegisterRequest>,
) -> Result<impl actix_web::Responder, crate::domain::error::DomainError> {
    let (user, token) = auth_service.register(payload.into_inner()).await?;
    Ok(HttpResponse::Created().json(RegisterResponse {
        id: user.id as u64,
        email: user.email,
        token: Some(token),
    }))
}

pub async fn login(
    auth_service: web::Data<Arc<AuthService>>,
    payload: web::Json<LoginRequest>,
) -> Result<impl actix_web::Responder, crate::domain::error::DomainError> {
    let (_, token) = auth_service
        .login(payload.username.clone(), payload.password.clone())
        .await?;
    Ok(HttpResponse::Ok().json(LoginResponse { token }))
}

pub async fn create_post(
    req: actix_web::HttpRequest,
    blog_service: web::Data<Arc<BlogService>>,
    payload: web::Json<PostCreateRequest>,
) -> Result<impl actix_web::Responder, crate::domain::error::DomainError> {
    let user = get_authenticated_user(&req)
        .ok_or(crate::domain::error::DomainError::Forbidden)?;

    let post = blog_service
        .create_post(user.user_id, payload.into_inner())
        .await?;
    Ok(HttpResponse::Created().json(post))
}

pub async fn get_post(
    path: web::Path<(i64,)>,
    blog_service: web::Data<Arc<BlogService>>,
) -> Result<impl actix_web::Responder, crate::domain::error::DomainError> {
    let post_id = path.into_inner().0;
    let post = blog_service.get_post(post_id).await?;
    Ok(HttpResponse::Ok().json(post))
}

pub async fn update_post(
    req: actix_web::HttpRequest,
    path: web::Path<(i64,)>,
    blog_service: web::Data<Arc<BlogService>>,
    payload: web::Json<PostUpdateRequest>,
) -> Result<impl actix_web::Responder, crate::domain::error::DomainError> {
    let user = get_authenticated_user(&req)
        .ok_or(crate::domain::error::DomainError::Forbidden)?;

    let post_id = path.into_inner().0;
    let post = blog_service
        .update_post(user.user_id, post_id, payload.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(post))
}

pub async fn delete_post(
    req: actix_web::HttpRequest,
    path: web::Path<(i64,)>,
    blog_service: web::Data<Arc<BlogService>>,
) -> Result<impl actix_web::Responder, crate::domain::error::DomainError> {
    let user = get_authenticated_user(&req)
        .ok_or(crate::domain::error::DomainError::Forbidden)?;

    let post_id = path.into_inner().0;
    blog_service.delete_post(user.user_id, post_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn list_posts(
    query: web::Query<PostsQuery>,
    blog_service: web::Data<Arc<BlogService>>,
) -> Result<impl actix_web::Responder, crate::domain::error::DomainError> {
    let limit = query.limit.unwrap_or(10).clamp(1, 100) as i64;
    let offset = query.offset.unwrap_or(0) as i64;

    let (posts, total) = blog_service.list_posts(limit, offset).await?;
    Ok(HttpResponse::Ok().json(PostsListResponse {
        posts,
        total,
        limit: limit as u32,
        offset: offset as u32,
    }))
}

use std::sync::Arc;

use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web};
use serde::Deserialize;

use crate::application::{auth_service::AuthService, blog_service::BlogService};
use crate::domain::post::{PostCreateRequest, PostUpdateRequest};
use crate::domain::user::{LoginRequest, RegisterRequest, User};
use crate::presentation::AuthenticatedUser;

#[derive(Deserialize)]
pub struct PostsQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(serde::Serialize)]
struct AuthResponse {
    token: String,
    user: User,
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
) -> impl Responder {
    match auth_service.register(payload.into_inner()).await {
        Ok((user, token)) => HttpResponse::Created().json(AuthResponse { token, user }),
        Err(err) => match err {
            crate::domain::error::DomainError::UserAlreadyExists => {
                HttpResponse::Conflict().body("User already exists")
            }
            _ => HttpResponse::InternalServerError().body(err.to_string()),
        },
    }
}

pub async fn login(
    auth_service: web::Data<Arc<AuthService>>,
    payload: web::Json<LoginRequest>,
) -> impl Responder {
    match auth_service
        .login(payload.username.clone(), payload.password.clone())
        .await
    {
        Ok((user, token)) => HttpResponse::Ok().json(AuthResponse { token, user }),
        Err(_) => HttpResponse::Unauthorized().body("Invalid credentials"),
    }
}

pub async fn create_post(
    req: actix_web::HttpRequest,
    blog_service: web::Data<Arc<BlogService>>,
    payload: web::Json<PostCreateRequest>,
) -> impl Responder {
    let user = match get_authenticated_user(&req) {
        Some(user) => user,
        None => return HttpResponse::Unauthorized().finish(),
    };

    match blog_service
        .create_post(user.user_id, payload.into_inner())
        .await
    {
        Ok(post) => HttpResponse::Created().json(post),
        Err(err) => match err {
            crate::domain::error::DomainError::PostNotFound => HttpResponse::NotFound().finish(),
            crate::domain::error::DomainError::Forbidden => HttpResponse::Forbidden().finish(),
            _ => HttpResponse::InternalServerError().body(err.to_string()),
        },
    }
}

pub async fn get_post(
    path: web::Path<(i64,)>,
    blog_service: web::Data<Arc<BlogService>>,
) -> impl Responder {
    let post_id = path.into_inner().0;
    match blog_service.get_post(post_id).await {
        Ok(post) => HttpResponse::Ok().json(post),
        Err(crate::domain::error::DomainError::PostNotFound) => HttpResponse::NotFound().finish(),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

pub async fn update_post(
    req: actix_web::HttpRequest,
    path: web::Path<(i64,)>,
    blog_service: web::Data<Arc<BlogService>>,
    payload: web::Json<PostUpdateRequest>,
) -> impl Responder {
    let user = match get_authenticated_user(&req) {
        Some(user) => user,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let post_id = path.into_inner().0;
    match blog_service
        .update_post(user.user_id, post_id, payload.into_inner())
        .await
    {
        Ok(post) => HttpResponse::Ok().json(post),
        Err(err) => match err {
            crate::domain::error::DomainError::PostNotFound => HttpResponse::NotFound().finish(),
            crate::domain::error::DomainError::Forbidden => HttpResponse::Forbidden().finish(),
            _ => HttpResponse::InternalServerError().body(err.to_string()),
        },
    }
}

pub async fn delete_post(
    req: actix_web::HttpRequest,
    path: web::Path<(i64,)>,
    blog_service: web::Data<Arc<BlogService>>,
) -> impl Responder {
    let user = match get_authenticated_user(&req) {
        Some(user) => user,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let post_id = path.into_inner().0;
    match blog_service.delete_post(user.user_id, post_id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(crate::domain::error::DomainError::PostNotFound) => HttpResponse::NotFound().finish(),
        Err(crate::domain::error::DomainError::Forbidden) => HttpResponse::Forbidden().finish(),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

pub async fn list_posts(
    query: web::Query<PostsQuery>,
    blog_service: web::Data<Arc<BlogService>>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(10).clamp(1, 100) as i64;
    let offset = query.offset.unwrap_or(0) as i64;

    match blog_service.list_posts(limit, offset).await {
        Ok((posts, total)) => HttpResponse::Ok().json(PostsListResponse {
            posts,
            total,
            limit: limit as u32,
            offset: offset as u32,
        }),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

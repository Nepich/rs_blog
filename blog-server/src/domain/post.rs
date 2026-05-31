use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostCreateRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostUpdateRequest {
    pub title: String,
    pub content: String,
}

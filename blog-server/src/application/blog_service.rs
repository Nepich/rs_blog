use std::sync::Arc;

use crate::data::{
    post_repository::PostgresPostRepository, user_repository::PostgresUserRepository,
};
use crate::domain::error::DomainError;
use crate::domain::post::{Post, PostCreateRequest, PostUpdateRequest};

#[derive(Clone)]
pub struct BlogService {
    post_repo: Arc<PostgresPostRepository>,
    user_repo: Arc<PostgresUserRepository>,
}

impl BlogService {
    pub fn new(
        post_repo: Arc<PostgresPostRepository>,
        user_repo: Arc<PostgresUserRepository>,
    ) -> Self {
        Self {
            post_repo,
            user_repo,
        }
    }

    pub async fn create_post(
        &self,
        author_id: i64,
        create: PostCreateRequest,
    ) -> Result<Post, DomainError> {
        self.user_repo
            .find_by_id(author_id)
            .await?
            .ok_or(DomainError::Forbidden)?;

        let post = self
            .post_repo
            .create_post(author_id, &create.title, &create.content)
            .await?;

        Ok(post)
    }

    pub async fn get_post(&self, post_id: i64) -> Result<Post, DomainError> {
        self.post_repo
            .get_post(post_id)
            .await?
            .ok_or(DomainError::PostNotFound)
    }

    pub async fn list_posts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Post>, i64), DomainError> {
        let (posts, total) = self.post_repo.list_posts(limit, offset).await?;
        Ok((posts, total))
    }

    pub async fn update_post(
        &self,
        user_id: i64,
        post_id: i64,
        update: PostUpdateRequest,
    ) -> Result<Post, DomainError> {
        let post = self
            .post_repo
            .get_post(post_id)
            .await?
            .ok_or(DomainError::PostNotFound)?;

        if post.author_id != user_id {
            return Err(DomainError::Forbidden);
        }

        let updated_post = self
            .post_repo
            .update_post(post_id, &update.title, &update.content)
            .await?
            .ok_or(DomainError::PostNotFound)?;

        Ok(updated_post)
    }

    pub async fn delete_post(&self, user_id: i64, post_id: i64) -> Result<(), DomainError> {
        let post = self
            .post_repo
            .get_post(post_id)
            .await?
            .ok_or(DomainError::PostNotFound)?;

        if post.author_id != user_id {
            return Err(DomainError::Forbidden);
        }

        self.post_repo.delete_post(post_id).await?;
        Ok(())
    }
}

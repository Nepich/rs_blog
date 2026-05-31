use crate::domain::post::Post;
use sqlx::PgPool;

#[derive(Clone)]
pub struct PostgresPostRepository {
    pub pool: PgPool,
}

impl PostgresPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_post(
        &self,
        author_id: i64,
        title: &str,
        content: &str,
    ) -> Result<Post, sqlx::Error> {
        let post = sqlx::query_as::<_, Post>(
            "INSERT INTO posts (title, content, author_id) VALUES ($1, $2, $3) RETURNING id, title, content, author_id, created_at, updated_at",
        )
        .bind(title)
        .bind(content)
        .bind(author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(post)
    }

    pub async fn get_post(&self, post_id: i64) -> Result<Option<Post>, sqlx::Error> {
        let post = sqlx::query_as::<_, Post>(
            "SELECT id, title, content, author_id, created_at, updated_at FROM posts WHERE id = $1",
        )
        .bind(post_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(post)
    }

    pub async fn list_posts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Post>, i64), sqlx::Error> {
        let posts = sqlx::query_as::<_, Post>(
            "SELECT id, title, content, author_id, created_at, updated_at FROM posts ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts")
            .fetch_one(&self.pool)
            .await?;

        Ok((posts, total))
    }

    pub async fn update_post(
        &self,
        post_id: i64,
        title: &str,
        content: &str,
    ) -> Result<Option<Post>, sqlx::Error> {
        let post = sqlx::query_as::<_, Post>(
            "UPDATE posts SET title = $1, content = $2, updated_at = now() WHERE id = $3 RETURNING id, title, content, author_id, created_at, updated_at",
        )
        .bind(title)
        .bind(content)
        .bind(post_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(post)
    }

    pub async fn delete_post(&self, post_id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM posts WHERE id = $1")
            .bind(post_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

use sqlx::{Pool, Postgres, Row, postgres::PgArguments};
use std::sync::Arc;

pub(crate) struct PostRepository {
    pool: Arc<Pool<Postgres>>,
}

impl PostRepository {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }

    pub async fn create_post(&self, title: &str, content: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO posts (title, content) VALUES ($1, $2)")
            .bind(title)
            .bind(content)
            .execute(self.pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn get_post(&self, post_id: i32) -> Result<(i32, String, String), sqlx::Error> {
        let row = sqlx::query("SELECT id, title, content FROM posts WHERE id = $1")
            .bind(post_id)
            .fetch_one(self.pool.as_ref())
            .await?;

        let id: i32 = row.try_get("id")?;
        let title: String = row.try_get("title")?;
        let content: String = row.try_get("content")?;

        Ok((id, title, content))
    }

    pub async fn list_posts(&self) -> Result<Vec<(i32, String, String)>, sqlx::Error> {
        let rows = sqlx::query("SELECT id, title, content FROM posts ORDER BY id")
            .fetch_all(self.pool.as_ref())
            .await?;

        let posts = rows
            .into_iter()
            .map(|row| {
                let id: i32 = row.try_get("id").unwrap_or_default();
                let title: String = row.try_get("title").unwrap_or_default();
                let content: String = row.try_get("content").unwrap_or_default();
                (id, title, content)
            })
            .collect();

        Ok(posts)
    }

    pub async fn update_post(&self, post_id: i32, title: &str, content: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE posts SET title = $1, content = $2, updated_at = now() WHERE id = $3")
            .bind(title)
            .bind(content)
            .bind(post_id)
            .execute(self.pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn delete_post(&self, post_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM posts WHERE id = $1")
            .bind(post_id)
            .execute(self.pool.as_ref())
            .await?;

        Ok(())
    }
}
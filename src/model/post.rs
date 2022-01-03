use serde::Serialize;
use sqlx::PgPool;

#[derive(Serialize)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub body: String,
}

impl Post {
    pub async fn find_all(pool: &PgPool) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let records = sqlx::query!("select id, title, body from posts")
            .fetch_all(pool)
            .await?;

        let posts = records
            .into_iter()
            .map(|record| Post {
                id: record.id.to_string(),
                title: record.title,
                body: record.body,
            })
            .collect();

        Ok(posts)
    }

    pub async fn find(pool: &PgPool, id: &str) -> Result<Post, Box<dyn std::error::Error>> {
        let uuid = sqlx::types::Uuid::parse_str(&id)?;

        let record = sqlx::query!("select id, title, body from posts where id = $1", uuid)
            .fetch_one(pool)
            .await?;

        let post = Post {
            id: record.id.to_string(),
            title: record.title,
            body: record.body,
        };

        Ok(post)
    }
}

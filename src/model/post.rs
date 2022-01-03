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

    pub async fn create(pool: &PgPool, title: &str, body: &str) -> Result<Post, sqlx::Error> {
        let record = sqlx::query!(
            "insert into posts (title, body) values ($1, $2) returning id, title, body",
            title,
            body
        )
        .fetch_one(pool)
        .await?;

        let post = Post {
            id: record.id.to_string(),
            title: record.title,
            body: record.body,
        };

        Ok(post)
    }

    pub fn uuid(&self) -> Result<sqlx::types::Uuid, sqlx::types::uuid::Error> {
        sqlx::types::Uuid::parse_str(&self.id)
    }

    pub async fn destroy(&self, pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
        let id = self.uuid()?;
        sqlx::query!("delete from posts where id = $1", id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn update(
        &self,
        pool: &PgPool,
        title: &str,
        body: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let id = self.uuid()?;

        sqlx::query!(
            "update posts set (title, body) = ($1, $2) where id = $3",
            title,
            body,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

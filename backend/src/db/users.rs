//! User database operations

use chrono::{DateTime, Utc};
use shared::models::{User, UserStatus};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id: String,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub avatar_url: Option<String>,
    pub status: String,
    pub status_message: Option<String>,
    pub last_seen: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            email: row.email,
            username: row.username,
            display_name: row.display_name,
            password_hash: row.password_hash,
            avatar_url: row.avatar_url,
            status: serde_json::from_str(&format!("\"{}\"", row.status)).unwrap_or_default(),
            status_message: row.status_message,
            last_seen: row.last_seen.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
            created_at: DateTime::parse_from_rfc3339(&row.created_at).unwrap().with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.updated_at).unwrap().with_timezone(&Utc),
        }
    }
}

pub struct UserRepository;

impl UserRepository {
    pub async fn create(
        pool: &PgPool,
        email: &str,
        username: &str,
        display_name: &str,
        password_hash: &str,
    ) -> Result<User, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let status = "offline";

        sqlx::query(
            r#"
            INSERT INTO users (id, email, username, display_name, password_hash, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(email)
        .bind(username)
        .bind(display_name)
        .bind(password_hash)
        .bind(status)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, &Uuid::parse_str(&id).unwrap()).await
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<User, sqlx::Error> {
        let row: UserRow = sqlx::query_as(
            r#"SELECT * FROM users WHERE id = ?"#,
        )
        .bind(id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<User, sqlx::Error> {
        let row: UserRow = sqlx::query_as(
            r#"SELECT * FROM users WHERE email = ?"#,
        )
        .bind(email)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<User, sqlx::Error> {
        let row: UserRow = sqlx::query_as(
            r#"SELECT * FROM users WHERE username = ?"#,
        )
        .bind(username)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn update(
        pool: &PgPool,
        id: &Uuid,
        display_name: Option<&str>,
        avatar_url: Option<&str>,
        status: Option<UserStatus>,
        status_message: Option<&str>,
    ) -> Result<User, sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        let mut query = String::from("UPDATE users SET updated_at = ?");
        let mut params: Vec<String> = vec![now.clone()];

        if let Some(name) = display_name {
            query.push_str(", display_name = ?");
            params.push(name.to_string());
        }

        if let Some(url) = avatar_url {
            query.push_str(", avatar_url = ?");
            params.push(url.to_string());
        }

        if let Some(s) = status {
            query.push_str(", status = ?");
            params.push(serde_json::to_string(&s).unwrap().trim_matches('"').to_string());
        }

        if let Some(msg) = status_message {
            query.push_str(", status_message = ?");
            params.push(msg.to_string());
        }

        query.push_str(" WHERE id = ?");
        params.push(id.to_string());

        let mut q = sqlx::query(&query);
        for param in &params {
            q = q.bind(param);
        }
        q.execute(pool).await?;

        Self::find_by_id(pool, id).await
    }

    pub async fn update_password(
        pool: &PgPool,
        id: &Uuid,
        password_hash: &str,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?"#,
        )
        .bind(password_hash)
        .bind(&now)
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_last_seen(pool: &PgPool, id: &Uuid) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"UPDATE users SET last_seen = ?, updated_at = ? WHERE id = ?"#,
        )
        .bind(&now)
        .bind(&now)
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn search(
        pool: &PgPool,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<User>, sqlx::Error> {
        let search_pattern = format!("%{}%", query);

        let rows: Vec<UserRow> = sqlx::query_as(
            r#"
            SELECT * FROM users 
            WHERE username LIKE ? OR display_name LIKE ? OR email LIKE ?
            ORDER BY username
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn exists_by_email(pool: &PgPool, email: &str) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM users WHERE email = ?"#,
        )
        .bind(email)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }

    pub async fn exists_by_username(pool: &PgPool, username: &str) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM users WHERE username = ?"#,
        )
        .bind(username)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }
}

//! User database operations

use chrono::{DateTime, Utc};
use shared::models::{User, UserStatus};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub avatar_url: Option<String>,
    pub status: String,
    pub status_message: Option<String>,
    pub last_seen: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User {
            id: row.id,
            email: row.email,
            username: row.username,
            display_name: row.display_name,
            password_hash: row.password_hash,
            avatar_url: row.avatar_url,
            status: serde_json::from_str(&format!("\"{}\"", row.status)).unwrap_or_default(),
            status_message: row.status_message,
            last_seen: row.last_seen,
            created_at: row.created_at,
            updated_at: row.updated_at,
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
        let id = Uuid::new_v4();
        let now = Utc::now();
        let status = "offline";

        sqlx::query(
            r#"
            INSERT INTO users (id, email, username, display_name, password_hash, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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

        Self::find_by_id(pool, &id).await
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<User, sqlx::Error> {
        let row: UserRow = sqlx::query_as(
            r#"SELECT id, email, username, display_name, password_hash, avatar_url, status, status_message, last_seen, created_at, updated_at FROM users WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<User, sqlx::Error> {
        let row: UserRow = sqlx::query_as(
            r#"SELECT id, email, username, display_name, password_hash, avatar_url, status, status_message, last_seen, created_at, updated_at FROM users WHERE email = $1"#,
        )
        .bind(email)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<User, sqlx::Error> {
        let row: UserRow = sqlx::query_as(
            r#"SELECT id, email, username, display_name, password_hash, avatar_url, status, status_message, last_seen, created_at, updated_at FROM users WHERE username = $1"#,
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
        let now = Utc::now();
        
        // Build dynamic query for PostgreSQL
        let mut set_clauses = vec!["updated_at = $1".to_string()];
        let mut param_index = 2;
        
        if display_name.is_some() {
            set_clauses.push(format!("display_name = ${}", param_index));
            param_index += 1;
        }
        
        if avatar_url.is_some() {
            set_clauses.push(format!("avatar_url = ${}", param_index));
            param_index += 1;
        }
        
        if status.is_some() {
            set_clauses.push(format!("status = ${}", param_index));
            param_index += 1;
        }
        
        if status_message.is_some() {
            set_clauses.push(format!("status_message = ${}", param_index));
            param_index += 1;
        }
        
        let query = format!(
            "UPDATE users SET {} WHERE id = ${}",
            set_clauses.join(", "),
            param_index
        );

        let mut q = sqlx::query(&query).bind(&now);
        
        if let Some(name) = display_name {
            q = q.bind(name);
        }
        if let Some(url) = avatar_url {
            q = q.bind(url);
        }
        if let Some(s) = status {
            q = q.bind(serde_json::to_string(&s).unwrap().trim_matches('"').to_string());
        }
        if let Some(msg) = status_message {
            q = q.bind(msg);
        }
        
        q = q.bind(id);
        q.execute(pool).await?;

        Self::find_by_id(pool, id).await
    }

    pub async fn update_password(
        pool: &PgPool,
        id: &Uuid,
        password_hash: &str,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3"#,
        )
        .bind(password_hash)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_last_seen(pool: &PgPool, id: &Uuid) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"UPDATE users SET last_seen = $1, updated_at = $2 WHERE id = $3"#,
        )
        .bind(&now)
        .bind(&now)
        .bind(id)
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
            SELECT id, email, username, display_name, password_hash, avatar_url, status, status_message, last_seen, created_at, updated_at 
            FROM users 
            WHERE username ILIKE $1 OR display_name ILIKE $2 OR email ILIKE $3
            ORDER BY username
            LIMIT $4 OFFSET $5
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
            r#"SELECT COUNT(*) FROM users WHERE email = $1"#,
        )
        .bind(email)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }

    pub async fn exists_by_username(pool: &PgPool, username: &str) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM users WHERE username = $1"#,
        )
        .bind(username)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }
}

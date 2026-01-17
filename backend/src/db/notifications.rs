//! Notification database operations

use chrono::{DateTime, Utc};
use shared::models::Notification;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct NotificationRow {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub body: String,
    pub notification_type: String,
    pub reference_id: Option<String>,
    pub read: bool,
    pub created_at: String,
}

impl From<NotificationRow> for Notification {
    fn from(row: NotificationRow) -> Self {
        Notification {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            user_id: Uuid::parse_str(&row.user_id).unwrap_or_default(),
            title: row.title,
            body: row.body,
            notification_type: row.notification_type,
            reference_id: row.reference_id.and_then(|s| Uuid::parse_str(&s).ok()),
            read: row.read,
            created_at: DateTime::parse_from_rfc3339(&row.created_at).unwrap().with_timezone(&Utc),
        }
    }
}

pub struct NotificationRepository;

impl NotificationRepository {
    pub async fn create(
        pool: &SqlitePool,
        user_id: &Uuid,
        title: &str,
        body: &str,
        notification_type: &str,
        reference_id: Option<&Uuid>,
    ) -> Result<Notification, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO notifications (id, user_id, title, body, notification_type, reference_id, read, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(user_id.to_string())
        .bind(title)
        .bind(body)
        .bind(notification_type)
        .bind(reference_id.map(|r| r.to_string()))
        .bind(false)
        .bind(&now)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, &Uuid::parse_str(&id).unwrap()).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &Uuid) -> Result<Notification, sqlx::Error> {
        let row: NotificationRow = sqlx::query_as(
            r#"SELECT * FROM notifications WHERE id = ?"#,
        )
        .bind(id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_user(
        pool: &SqlitePool,
        user_id: &Uuid,
        unread_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Notification>, sqlx::Error> {
        let query = if unread_only {
            r#"SELECT * FROM notifications WHERE user_id = ? AND read = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"#
        } else {
            r#"SELECT * FROM notifications WHERE user_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"#
        };

        let rows: Vec<NotificationRow> = if unread_only {
            sqlx::query_as(query)
                .bind(user_id.to_string())
                .bind(false)
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await?
        } else {
            sqlx::query_as(query)
                .bind(user_id.to_string())
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await?
        };

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn mark_as_read(pool: &SqlitePool, id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE notifications SET read = ? WHERE id = ?"#,
        )
        .bind(true)
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn mark_all_as_read(pool: &SqlitePool, user_id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE notifications SET read = ? WHERE user_id = ? AND read = ?"#,
        )
        .bind(true)
        .bind(user_id.to_string())
        .bind(false)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_unread_count(pool: &SqlitePool, user_id: &Uuid) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM notifications WHERE user_id = ? AND read = ?"#,
        )
        .bind(user_id.to_string())
        .bind(false)
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }

    pub async fn delete(pool: &SqlitePool, id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(r#"DELETE FROM notifications WHERE id = ?"#)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn delete_old(pool: &SqlitePool, days: i64) -> Result<u64, sqlx::Error> {
        let cutoff = (Utc::now() - chrono::Duration::days(days)).to_rfc3339();

        let result = sqlx::query(
            r#"DELETE FROM notifications WHERE created_at < ? AND read = ?"#,
        )
        .bind(&cutoff)
        .bind(true)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

//! Notification database operations

use chrono::{DateTime, Utc};
use shared::models::{Notification, NotificationType};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct NotificationRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub body: String,
    pub notification_type: String,
    pub reference_id: Option<String>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

impl From<NotificationRow> for Notification {
    fn from(row: NotificationRow) -> Self {
        Notification {
            id: row.id,
            user_id: row.user_id,
            title: row.title,
            body: row.body,
            notification_type: row.notification_type.parse().unwrap_or(NotificationType::System),
            reference_id: row.reference_id,
            read: row.read,
            created_at: row.created_at,
        }
    }
}

pub struct NotificationRepository;

impl NotificationRepository {
    pub async fn create(
        pool: &PgPool,
        user_id: &Uuid,
        title: &str,
        body: &str,
        notification_type: &str,
        reference_id: Option<&Uuid>,
    ) -> Result<Notification, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO notifications (id, user_id, title, body, notification_type, reference_id, read, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&id)
        .bind(user_id)
        .bind(title)
        .bind(body)
        .bind(notification_type)
        .bind(reference_id.map(|r| r.to_string()))
        .bind(false)
        .bind(&now)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, &id).await
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Notification, sqlx::Error> {
        let row: NotificationRow = sqlx::query_as(
            r#"SELECT id, user_id, title, body, notification_type, reference_id, read, created_at FROM notifications WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_user(
        pool: &PgPool,
        user_id: &Uuid,
        unread_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Notification>, sqlx::Error> {
        let rows: Vec<NotificationRow> = if unread_only {
            sqlx::query_as(
                r#"SELECT id, user_id, title, body, notification_type, reference_id, read, created_at FROM notifications WHERE user_id = $1 AND read = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4"#
            )
            .bind(user_id)
            .bind(false)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as(
                r#"SELECT id, user_id, title, body, notification_type, reference_id, read, created_at FROM notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"#
            )
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        };

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn mark_as_read(pool: &PgPool, id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE notifications SET read = $1 WHERE id = $2"#,
        )
        .bind(true)
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn mark_all_as_read(pool: &PgPool, user_id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE notifications SET read = $1 WHERE user_id = $2 AND read = $3"#,
        )
        .bind(true)
        .bind(user_id)
        .bind(false)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_unread_count(pool: &PgPool, user_id: &Uuid) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND read = $2"#,
        )
        .bind(user_id)
        .bind(false)
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }

    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(r#"DELETE FROM notifications WHERE id = $1"#)
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn delete_old(pool: &PgPool, days: i64) -> Result<u64, sqlx::Error> {
        let cutoff = Utc::now() - chrono::Duration::days(days);

        let result = sqlx::query(
            r#"DELETE FROM notifications WHERE created_at < $1 AND read = $2"#,
        )
        .bind(&cutoff)
        .bind(true)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

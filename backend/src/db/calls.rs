//! Call database operations

use chrono::{DateTime, Utc};
use shared::models::{Call, CallParticipant, CallStatus, CallType};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct CallRow {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub initiator_id: Uuid,
    pub call_type: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

impl From<CallRow> for Call {
    fn from(row: CallRow) -> Self {
        Call {
            id: row.id,
            channel_id: row.channel_id,
            initiator_id: row.initiator_id,
            call_type: serde_json::from_str(&format!("\"{}\"", row.call_type)).unwrap_or(CallType::Audio),
            status: serde_json::from_str(&format!("\"{}\"", row.status)).unwrap_or(CallStatus::Ringing),
            started_at: row.started_at,
            ended_at: row.ended_at,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct CallParticipantRow {
    pub id: Uuid,
    pub call_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub is_muted: bool,
    pub is_video_enabled: bool,
}

impl From<CallParticipantRow> for CallParticipant {
    fn from(row: CallParticipantRow) -> Self {
        CallParticipant {
            id: row.id,
            call_id: row.call_id,
            user_id: row.user_id,
            joined_at: row.joined_at,
            left_at: row.left_at,
            is_muted: row.is_muted,
            is_video_enabled: row.is_video_enabled,
        }
    }
}

pub struct CallRepository;

impl CallRepository {
    pub async fn create(
        pool: &PgPool,
        channel_id: &Uuid,
        initiator_id: &Uuid,
        call_type: CallType,
    ) -> Result<Call, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let type_str = serde_json::to_string(&call_type).unwrap().trim_matches('"').to_string();
        let status_str = "ringing";

        sqlx::query(
            r#"
            INSERT INTO calls (id, channel_id, initiator_id, call_type, status, started_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&id)
        .bind(channel_id)
        .bind(initiator_id)
        .bind(&type_str)
        .bind(status_str)
        .bind(&now)
        .execute(pool)
        .await?;

        // Add initiator as participant
        Self::add_participant(pool, &id, initiator_id, call_type == CallType::Video).await?;

        Self::find_by_id(pool, &id).await
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Call, sqlx::Error> {
        let row: CallRow = sqlx::query_as(
            r#"SELECT id, channel_id, initiator_id, call_type, status, started_at, ended_at FROM calls WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_active_by_channel(pool: &PgPool, channel_id: &Uuid) -> Result<Option<Call>, sqlx::Error> {
        let row: Option<CallRow> = sqlx::query_as(
            r#"SELECT id, channel_id, initiator_id, call_type, status, started_at, ended_at FROM calls WHERE channel_id = $1 AND status IN ('ringing', 'in_progress')"#,
        )
        .bind(channel_id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn update_status(
        pool: &PgPool,
        id: &Uuid,
        status: CallStatus,
    ) -> Result<Call, sqlx::Error> {
        let status_str = serde_json::to_string(&status).unwrap().trim_matches('"').to_string();
        let ended_at = if status == CallStatus::Ended || status == CallStatus::Missed || status == CallStatus::Declined {
            Some(Utc::now())
        } else {
            None
        };

        if let Some(ended) = &ended_at {
            sqlx::query(
                r#"UPDATE calls SET status = $1, ended_at = $2 WHERE id = $3"#,
            )
            .bind(&status_str)
            .bind(ended)
            .bind(id)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                r#"UPDATE calls SET status = $1 WHERE id = $2"#,
            )
            .bind(&status_str)
            .bind(id)
            .execute(pool)
            .await?;
        }

        Self::find_by_id(pool, id).await
    }

    pub async fn add_participant(
        pool: &PgPool,
        call_id: &Uuid,
        user_id: &Uuid,
        is_video_enabled: bool,
    ) -> Result<CallParticipant, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO call_participants (id, call_id, user_id, joined_at, is_muted, is_video_enabled)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&id)
        .bind(call_id)
        .bind(user_id)
        .bind(&now)
        .bind(false)
        .bind(is_video_enabled)
        .execute(pool)
        .await?;

        Self::find_participant(pool, call_id, user_id).await
    }

    pub async fn find_participant(
        pool: &PgPool,
        call_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<CallParticipant, sqlx::Error> {
        let row: CallParticipantRow = sqlx::query_as(
            r#"SELECT id, call_id, user_id, joined_at, left_at, is_muted, is_video_enabled FROM call_participants WHERE call_id = $1 AND user_id = $2 AND left_at IS NULL"#,
        )
        .bind(call_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_participants(pool: &PgPool, call_id: &Uuid) -> Result<Vec<CallParticipant>, sqlx::Error> {
        let rows: Vec<CallParticipantRow> = sqlx::query_as(
            r#"SELECT id, call_id, user_id, joined_at, left_at, is_muted, is_video_enabled FROM call_participants WHERE call_id = $1 AND left_at IS NULL ORDER BY joined_at"#,
        )
        .bind(call_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_participant(
        pool: &PgPool,
        call_id: &Uuid,
        user_id: &Uuid,
        is_muted: Option<bool>,
        is_video_enabled: Option<bool>,
    ) -> Result<CallParticipant, sqlx::Error> {
        match (is_muted, is_video_enabled) {
            (Some(muted), Some(video)) => {
                sqlx::query(
                    r#"UPDATE call_participants SET is_muted = $1, is_video_enabled = $2 WHERE call_id = $3 AND user_id = $4 AND left_at IS NULL"#,
                )
                .bind(muted)
                .bind(video)
                .bind(call_id)
                .bind(user_id)
                .execute(pool)
                .await?;
            }
            (Some(muted), None) => {
                sqlx::query(
                    r#"UPDATE call_participants SET is_muted = $1 WHERE call_id = $2 AND user_id = $3 AND left_at IS NULL"#,
                )
                .bind(muted)
                .bind(call_id)
                .bind(user_id)
                .execute(pool)
                .await?;
            }
            (None, Some(video)) => {
                sqlx::query(
                    r#"UPDATE call_participants SET is_video_enabled = $1 WHERE call_id = $2 AND user_id = $3 AND left_at IS NULL"#,
                )
                .bind(video)
                .bind(call_id)
                .bind(user_id)
                .execute(pool)
                .await?;
            }
            (None, None) => {}
        }

        Self::find_participant(pool, call_id, user_id).await
    }

    pub async fn remove_participant(
        pool: &PgPool,
        call_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"UPDATE call_participants SET left_at = $1 WHERE call_id = $2 AND user_id = $3 AND left_at IS NULL"#,
        )
        .bind(&now)
        .bind(call_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn is_participant(
        pool: &PgPool,
        call_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM call_participants WHERE call_id = $1 AND user_id = $2 AND left_at IS NULL"#,
        )
        .bind(call_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }

    pub async fn get_participant_count(pool: &PgPool, call_id: &Uuid) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM call_participants WHERE call_id = $1 AND left_at IS NULL"#,
        )
        .bind(call_id)
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }
}

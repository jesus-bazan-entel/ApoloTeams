//! Call database operations

use chrono::{DateTime, Utc};
use shared::models::{Call, CallParticipant, CallStatus, CallType};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct CallRow {
    pub id: String,
    pub channel_id: String,
    pub initiator_id: String,
    pub call_type: String,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
}

impl From<CallRow> for Call {
    fn from(row: CallRow) -> Self {
        Call {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            channel_id: Uuid::parse_str(&row.channel_id).unwrap_or_default(),
            initiator_id: Uuid::parse_str(&row.initiator_id).unwrap_or_default(),
            call_type: serde_json::from_str(&format!("\"{}\"", row.call_type)).unwrap_or(CallType::Audio),
            status: serde_json::from_str(&format!("\"{}\"", row.status)).unwrap_or(CallStatus::Ringing),
            started_at: DateTime::parse_from_rfc3339(&row.started_at).unwrap().with_timezone(&Utc),
            ended_at: row.ended_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct CallParticipantRow {
    pub id: String,
    pub call_id: String,
    pub user_id: String,
    pub joined_at: String,
    pub left_at: Option<String>,
    pub is_muted: bool,
    pub is_video_enabled: bool,
}

impl From<CallParticipantRow> for CallParticipant {
    fn from(row: CallParticipantRow) -> Self {
        CallParticipant {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            call_id: Uuid::parse_str(&row.call_id).unwrap_or_default(),
            user_id: Uuid::parse_str(&row.user_id).unwrap_or_default(),
            joined_at: DateTime::parse_from_rfc3339(&row.joined_at).unwrap().with_timezone(&Utc),
            left_at: row.left_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
            is_muted: row.is_muted,
            is_video_enabled: row.is_video_enabled,
        }
    }
}

pub struct CallRepository;

impl CallRepository {
    pub async fn create(
        pool: &SqlitePool,
        channel_id: &Uuid,
        initiator_id: &Uuid,
        call_type: CallType,
    ) -> Result<Call, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let type_str = serde_json::to_string(&call_type).unwrap().trim_matches('"').to_string();
        let status_str = "ringing";

        sqlx::query(
            r#"
            INSERT INTO calls (id, channel_id, initiator_id, call_type, status, started_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(channel_id.to_string())
        .bind(initiator_id.to_string())
        .bind(&type_str)
        .bind(status_str)
        .bind(&now)
        .execute(pool)
        .await?;

        // Add initiator as participant
        Self::add_participant(pool, &Uuid::parse_str(&id).unwrap(), initiator_id, call_type == CallType::Video).await?;

        Self::find_by_id(pool, &Uuid::parse_str(&id).unwrap()).await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &Uuid) -> Result<Call, sqlx::Error> {
        let row: CallRow = sqlx::query_as(
            r#"SELECT * FROM calls WHERE id = ?"#,
        )
        .bind(id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_active_by_channel(pool: &SqlitePool, channel_id: &Uuid) -> Result<Option<Call>, sqlx::Error> {
        let row: Option<CallRow> = sqlx::query_as(
            r#"SELECT * FROM calls WHERE channel_id = ? AND status IN ('ringing', 'in_progress')"#,
        )
        .bind(channel_id.to_string())
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn update_status(
        pool: &SqlitePool,
        id: &Uuid,
        status: CallStatus,
    ) -> Result<Call, sqlx::Error> {
        let status_str = serde_json::to_string(&status).unwrap().trim_matches('"').to_string();
        let ended_at = if status == CallStatus::Ended || status == CallStatus::Missed || status == CallStatus::Declined {
            Some(Utc::now().to_rfc3339())
        } else {
            None
        };

        if let Some(ended) = &ended_at {
            sqlx::query(
                r#"UPDATE calls SET status = ?, ended_at = ? WHERE id = ?"#,
            )
            .bind(&status_str)
            .bind(ended)
            .bind(id.to_string())
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                r#"UPDATE calls SET status = ? WHERE id = ?"#,
            )
            .bind(&status_str)
            .bind(id.to_string())
            .execute(pool)
            .await?;
        }

        Self::find_by_id(pool, id).await
    }

    pub async fn add_participant(
        pool: &SqlitePool,
        call_id: &Uuid,
        user_id: &Uuid,
        is_video_enabled: bool,
    ) -> Result<CallParticipant, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO call_participants (id, call_id, user_id, joined_at, is_muted, is_video_enabled)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(call_id.to_string())
        .bind(user_id.to_string())
        .bind(&now)
        .bind(false)
        .bind(is_video_enabled)
        .execute(pool)
        .await?;

        Self::find_participant(pool, call_id, user_id).await
    }

    pub async fn find_participant(
        pool: &SqlitePool,
        call_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<CallParticipant, sqlx::Error> {
        let row: CallParticipantRow = sqlx::query_as(
            r#"SELECT * FROM call_participants WHERE call_id = ? AND user_id = ? AND left_at IS NULL"#,
        )
        .bind(call_id.to_string())
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_participants(pool: &SqlitePool, call_id: &Uuid) -> Result<Vec<CallParticipant>, sqlx::Error> {
        let rows: Vec<CallParticipantRow> = sqlx::query_as(
            r#"SELECT * FROM call_participants WHERE call_id = ? AND left_at IS NULL ORDER BY joined_at"#,
        )
        .bind(call_id.to_string())
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_participant(
        pool: &SqlitePool,
        call_id: &Uuid,
        user_id: &Uuid,
        is_muted: Option<bool>,
        is_video_enabled: Option<bool>,
    ) -> Result<CallParticipant, sqlx::Error> {
        let mut query = String::from("UPDATE call_participants SET id = id");
        let mut params: Vec<String> = vec![];

        if let Some(muted) = is_muted {
            query.push_str(", is_muted = ?");
            params.push(muted.to_string());
        }

        if let Some(video) = is_video_enabled {
            query.push_str(", is_video_enabled = ?");
            params.push(video.to_string());
        }

        query.push_str(" WHERE call_id = ? AND user_id = ? AND left_at IS NULL");
        params.push(call_id.to_string());
        params.push(user_id.to_string());

        let mut q = sqlx::query(&query);
        for param in &params {
            if param == "true" || param == "false" {
                q = q.bind(param == "true");
            } else {
                q = q.bind(param);
            }
        }
        q.execute(pool).await?;

        Self::find_participant(pool, call_id, user_id).await
    }

    pub async fn remove_participant(
        pool: &SqlitePool,
        call_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"UPDATE call_participants SET left_at = ? WHERE call_id = ? AND user_id = ? AND left_at IS NULL"#,
        )
        .bind(&now)
        .bind(call_id.to_string())
        .bind(user_id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn is_participant(
        pool: &SqlitePool,
        call_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM call_participants WHERE call_id = ? AND user_id = ? AND left_at IS NULL"#,
        )
        .bind(call_id.to_string())
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }

    pub async fn get_participant_count(pool: &SqlitePool, call_id: &Uuid) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM call_participants WHERE call_id = ? AND left_at IS NULL"#,
        )
        .bind(call_id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }
}

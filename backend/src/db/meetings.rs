//! Meeting database operations

use chrono::{DateTime, Utc};
use shared::models::{Meeting, MeetingParticipant, MeetingResponseStatus, MeetingStatus, RecurrenceType};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct MeetingRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub organizer_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub timezone: String,
    pub status: String,
    pub is_online: bool,
    pub meeting_link: Option<String>,
    pub location: Option<String>,
    pub recurrence: String,
    pub channel_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<MeetingRow> for Meeting {
    fn from(row: MeetingRow) -> Self {
        Meeting {
            id: row.id,
            title: row.title,
            description: row.description,
            organizer_id: row.organizer_id,
            start_time: row.start_time,
            end_time: row.end_time,
            timezone: row.timezone,
            status: serde_json::from_str(&format!("\"{}\"", row.status)).unwrap_or(MeetingStatus::Scheduled),
            is_online: row.is_online,
            meeting_link: row.meeting_link,
            location: row.location,
            recurrence: serde_json::from_str(&format!("\"{}\"", row.recurrence)).unwrap_or(RecurrenceType::None),
            channel_id: row.channel_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct MeetingParticipantRow {
    pub id: Uuid,
    pub meeting_id: Uuid,
    pub user_id: Uuid,
    pub response_status: String,
    pub is_organizer: bool,
    pub invited_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
}

impl From<MeetingParticipantRow> for MeetingParticipant {
    fn from(row: MeetingParticipantRow) -> Self {
        MeetingParticipant {
            id: row.id,
            meeting_id: row.meeting_id,
            user_id: row.user_id,
            response_status: serde_json::from_str(&format!("\"{}\"", row.response_status))
                .unwrap_or(MeetingResponseStatus::Pending),
            is_organizer: row.is_organizer,
            invited_at: row.invited_at,
            responded_at: row.responded_at,
        }
    }
}

pub struct MeetingRepository;

impl MeetingRepository {
    pub async fn create(
        pool: &PgPool,
        organizer_id: &Uuid,
        title: &str,
        description: Option<&str>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        timezone: Option<&str>,
        is_online: bool,
        location: Option<&str>,
        recurrence: RecurrenceType,
        channel_id: Option<Uuid>,
    ) -> Result<Meeting, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let tz = timezone.unwrap_or("UTC");
        let status_str = serde_json::to_string(&MeetingStatus::Scheduled)
            .unwrap()
            .trim_matches('"')
            .to_string();
        let recurrence_str = serde_json::to_string(&recurrence)
            .unwrap()
            .trim_matches('"')
            .to_string();

        // Generate meeting link for online meetings
        let meeting_link = if is_online {
            Some(format!("/meeting/{}", id))
        } else {
            None
        };

        sqlx::query(
            r#"
            INSERT INTO meetings (id, title, description, organizer_id, start_time, end_time, timezone, status, is_online, meeting_link, location, recurrence, channel_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(&id)
        .bind(title)
        .bind(description)
        .bind(organizer_id)
        .bind(&start_time)
        .bind(&end_time)
        .bind(tz)
        .bind(&status_str)
        .bind(is_online)
        .bind(&meeting_link)
        .bind(location)
        .bind(&recurrence_str)
        .bind(&channel_id)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        // Add organizer as participant with accepted status
        Self::add_participant(pool, &id, organizer_id, true).await?;

        Self::find_by_id(pool, &id).await
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Meeting, sqlx::Error> {
        let row: MeetingRow = sqlx::query_as(
            r#"SELECT id, title, description, organizer_id, start_time, end_time, timezone, status, is_online, meeting_link, location, recurrence, channel_id, created_at, updated_at FROM meetings WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_organizer(pool: &PgPool, organizer_id: &Uuid) -> Result<Vec<Meeting>, sqlx::Error> {
        let rows: Vec<MeetingRow> = sqlx::query_as(
            r#"SELECT id, title, description, organizer_id, start_time, end_time, timezone, status, is_online, meeting_link, location, recurrence, channel_id, created_at, updated_at FROM meetings WHERE organizer_id = $1 ORDER BY start_time"#,
        )
        .bind(organizer_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn find_by_user(pool: &PgPool, user_id: &Uuid) -> Result<Vec<Meeting>, sqlx::Error> {
        let rows: Vec<MeetingRow> = sqlx::query_as(
            r#"
            SELECT m.id, m.title, m.description, m.organizer_id, m.start_time, m.end_time, m.timezone, m.status, m.is_online, m.meeting_link, m.location, m.recurrence, m.channel_id, m.created_at, m.updated_at
            FROM meetings m
            INNER JOIN meeting_participants mp ON m.id = mp.meeting_id
            WHERE mp.user_id = $1 AND m.status != 'cancelled'
            ORDER BY m.start_time
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn find_by_date_range(
        pool: &PgPool,
        user_id: &Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<Meeting>, sqlx::Error> {
        let rows: Vec<MeetingRow> = sqlx::query_as(
            r#"
            SELECT m.id, m.title, m.description, m.organizer_id, m.start_time, m.end_time, m.timezone, m.status, m.is_online, m.meeting_link, m.location, m.recurrence, m.channel_id, m.created_at, m.updated_at
            FROM meetings m
            INNER JOIN meeting_participants mp ON m.id = mp.meeting_id
            WHERE mp.user_id = $1
              AND m.status != 'cancelled'
              AND m.start_time >= $2
              AND m.start_time <= $3
            ORDER BY m.start_time
            "#,
        )
        .bind(user_id)
        .bind(&start_date)
        .bind(&end_date)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update(
        pool: &PgPool,
        id: &Uuid,
        title: Option<&str>,
        description: Option<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        timezone: Option<&str>,
        is_online: Option<bool>,
        location: Option<&str>,
        recurrence: Option<RecurrenceType>,
        status: Option<MeetingStatus>,
    ) -> Result<Meeting, sqlx::Error> {
        let now = Utc::now();

        // Build dynamic update query
        let mut updates = vec!["updated_at = $1"];
        let mut param_count = 1;

        if title.is_some() { param_count += 1; updates.push("title = $"); }
        if description.is_some() { param_count += 1; updates.push("description = $"); }
        if start_time.is_some() { param_count += 1; updates.push("start_time = $"); }
        if end_time.is_some() { param_count += 1; updates.push("end_time = $"); }
        if timezone.is_some() { param_count += 1; updates.push("timezone = $"); }
        if is_online.is_some() { param_count += 1; updates.push("is_online = $"); }
        if location.is_some() { param_count += 1; updates.push("location = $"); }
        if recurrence.is_some() { param_count += 1; updates.push("recurrence = $"); }
        if status.is_some() { param_count += 1; updates.push("status = $"); }

        // Simple approach: update all provided fields
        let mut query = sqlx::query("UPDATE meetings SET updated_at = $1");
        let mut binding = query.bind(&now);

        if let Some(t) = title {
            binding = sqlx::query("UPDATE meetings SET updated_at = $1, title = $2 WHERE id = $3")
                .bind(&now)
                .bind(t)
                .bind(id);
            binding.execute(pool).await?;
        }
        if let Some(d) = description {
            sqlx::query("UPDATE meetings SET description = $1, updated_at = $2 WHERE id = $3")
                .bind(d)
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;
        }
        if let Some(st) = start_time {
            sqlx::query("UPDATE meetings SET start_time = $1, updated_at = $2 WHERE id = $3")
                .bind(&st)
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;
        }
        if let Some(et) = end_time {
            sqlx::query("UPDATE meetings SET end_time = $1, updated_at = $2 WHERE id = $3")
                .bind(&et)
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;
        }
        if let Some(tz) = timezone {
            sqlx::query("UPDATE meetings SET timezone = $1, updated_at = $2 WHERE id = $3")
                .bind(tz)
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;
        }
        if let Some(online) = is_online {
            sqlx::query("UPDATE meetings SET is_online = $1, updated_at = $2 WHERE id = $3")
                .bind(online)
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;
        }
        if let Some(loc) = location {
            sqlx::query("UPDATE meetings SET location = $1, updated_at = $2 WHERE id = $3")
                .bind(loc)
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;
        }
        if let Some(rec) = recurrence {
            let rec_str = serde_json::to_string(&rec).unwrap().trim_matches('"').to_string();
            sqlx::query("UPDATE meetings SET recurrence = $1, updated_at = $2 WHERE id = $3")
                .bind(&rec_str)
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;
        }
        if let Some(s) = status {
            let status_str = serde_json::to_string(&s).unwrap().trim_matches('"').to_string();
            sqlx::query("UPDATE meetings SET status = $1, updated_at = $2 WHERE id = $3")
                .bind(&status_str)
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;
        }

        Self::find_by_id(pool, id).await
    }

    pub async fn update_status(
        pool: &PgPool,
        id: &Uuid,
        status: MeetingStatus,
    ) -> Result<Meeting, sqlx::Error> {
        let now = Utc::now();
        let status_str = serde_json::to_string(&status).unwrap().trim_matches('"').to_string();

        sqlx::query("UPDATE meetings SET status = $1, updated_at = $2 WHERE id = $3")
            .bind(&status_str)
            .bind(&now)
            .bind(id)
            .execute(pool)
            .await?;

        Self::find_by_id(pool, id).await
    }

    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM meetings WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    // Participant methods

    pub async fn add_participant(
        pool: &PgPool,
        meeting_id: &Uuid,
        user_id: &Uuid,
        is_organizer: bool,
    ) -> Result<MeetingParticipant, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let status = if is_organizer {
            MeetingResponseStatus::Accepted
        } else {
            MeetingResponseStatus::Pending
        };
        let status_str = serde_json::to_string(&status).unwrap().trim_matches('"').to_string();
        let responded_at = if is_organizer { Some(now) } else { None };

        sqlx::query(
            r#"
            INSERT INTO meeting_participants (id, meeting_id, user_id, response_status, is_organizer, invited_at, responded_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (meeting_id, user_id) DO NOTHING
            "#,
        )
        .bind(&id)
        .bind(meeting_id)
        .bind(user_id)
        .bind(&status_str)
        .bind(is_organizer)
        .bind(&now)
        .bind(&responded_at)
        .execute(pool)
        .await?;

        Self::find_participant(pool, meeting_id, user_id).await
    }

    pub async fn find_participant(
        pool: &PgPool,
        meeting_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<MeetingParticipant, sqlx::Error> {
        let row: MeetingParticipantRow = sqlx::query_as(
            r#"SELECT id, meeting_id, user_id, response_status, is_organizer, invited_at, responded_at FROM meeting_participants WHERE meeting_id = $1 AND user_id = $2"#,
        )
        .bind(meeting_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_participants(pool: &PgPool, meeting_id: &Uuid) -> Result<Vec<MeetingParticipant>, sqlx::Error> {
        let rows: Vec<MeetingParticipantRow> = sqlx::query_as(
            r#"SELECT id, meeting_id, user_id, response_status, is_organizer, invited_at, responded_at FROM meeting_participants WHERE meeting_id = $1 ORDER BY is_organizer DESC, invited_at"#,
        )
        .bind(meeting_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_participant_response(
        pool: &PgPool,
        meeting_id: &Uuid,
        user_id: &Uuid,
        response: MeetingResponseStatus,
    ) -> Result<MeetingParticipant, sqlx::Error> {
        let now = Utc::now();
        let status_str = serde_json::to_string(&response).unwrap().trim_matches('"').to_string();

        sqlx::query(
            r#"UPDATE meeting_participants SET response_status = $1, responded_at = $2 WHERE meeting_id = $3 AND user_id = $4"#,
        )
        .bind(&status_str)
        .bind(&now)
        .bind(meeting_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Self::find_participant(pool, meeting_id, user_id).await
    }

    pub async fn remove_participant(
        pool: &PgPool,
        meeting_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM meeting_participants WHERE meeting_id = $1 AND user_id = $2")
            .bind(meeting_id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn is_participant(
        pool: &PgPool,
        meeting_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM meeting_participants WHERE meeting_id = $1 AND user_id = $2"#,
        )
        .bind(meeting_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }

    pub async fn is_organizer(
        pool: &PgPool,
        meeting_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM meeting_participants WHERE meeting_id = $1 AND user_id = $2 AND is_organizer = true"#,
        )
        .bind(meeting_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }
}

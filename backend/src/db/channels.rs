//! Channel database operations

use chrono::{DateTime, Utc};
use shared::models::{Channel, ChannelMember, ChannelType};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct ChannelRow {
    pub id: String,
    pub team_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub channel_type: String,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ChannelRow> for Channel {
    fn from(row: ChannelRow) -> Self {
        Channel {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            team_id: row.team_id.and_then(|s| Uuid::parse_str(&s).ok()),
            name: row.name,
            description: row.description,
            channel_type: serde_json::from_str(&format!("\"{}\"", row.channel_type)).unwrap_or_default(),
            created_by: Uuid::parse_str(&row.created_by).unwrap_or_default(),
            created_at: DateTime::parse_from_rfc3339(&row.created_at).unwrap().with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.updated_at).unwrap().with_timezone(&Utc),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct ChannelMemberRow {
    pub id: String,
    pub channel_id: String,
    pub user_id: String,
    pub joined_at: String,
    pub last_read_at: Option<String>,
}

impl From<ChannelMemberRow> for ChannelMember {
    fn from(row: ChannelMemberRow) -> Self {
        ChannelMember {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            channel_id: Uuid::parse_str(&row.channel_id).unwrap_or_default(),
            user_id: Uuid::parse_str(&row.user_id).unwrap_or_default(),
            joined_at: DateTime::parse_from_rfc3339(&row.joined_at).unwrap().with_timezone(&Utc),
            last_read_at: row.last_read_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
        }
    }
}

pub struct ChannelRepository;

impl ChannelRepository {
    pub async fn create(
        pool: &PgPool,
        team_id: Option<&Uuid>,
        name: &str,
        description: Option<&str>,
        channel_type: ChannelType,
        created_by: &Uuid,
    ) -> Result<Channel, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let type_str = serde_json::to_string(&channel_type).unwrap().trim_matches('"').to_string();

        sqlx::query(
            r#"
            INSERT INTO channels (id, team_id, name, description, channel_type, created_by, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(team_id.map(|t| t.to_string()))
        .bind(name)
        .bind(description)
        .bind(&type_str)
        .bind(created_by.to_string())
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        // Add creator as a member
        let member_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO channel_members (id, channel_id, user_id, joined_at)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&member_id)
        .bind(&id)
        .bind(created_by.to_string())
        .bind(&now)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, &Uuid::parse_str(&id).unwrap()).await
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Channel, sqlx::Error> {
        let row: ChannelRow = sqlx::query_as(
            r#"SELECT * FROM channels WHERE id = ?"#,
        )
        .bind(id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_team(pool: &PgPool, team_id: &Uuid) -> Result<Vec<Channel>, sqlx::Error> {
        let rows: Vec<ChannelRow> = sqlx::query_as(
            r#"SELECT * FROM channels WHERE team_id = ? ORDER BY name"#,
        )
        .bind(team_id.to_string())
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn find_by_user(pool: &PgPool, user_id: &Uuid) -> Result<Vec<Channel>, sqlx::Error> {
        let rows: Vec<ChannelRow> = sqlx::query_as(
            r#"
            SELECT c.* FROM channels c
            INNER JOIN channel_members cm ON c.id = cm.channel_id
            WHERE cm.user_id = ?
            ORDER BY c.name
            "#,
        )
        .bind(user_id.to_string())
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn find_dm_channel(
        pool: &PgPool,
        user1_id: &Uuid,
        user2_id: &Uuid,
    ) -> Result<Option<Channel>, sqlx::Error> {
        let row: Option<ChannelRow> = sqlx::query_as(
            r#"
            SELECT c.* FROM channels c
            INNER JOIN channel_members cm1 ON c.id = cm1.channel_id
            INNER JOIN channel_members cm2 ON c.id = cm2.channel_id
            WHERE c.channel_type = 'direct_message'
            AND cm1.user_id = ?
            AND cm2.user_id = ?
            "#,
        )
        .bind(user1_id.to_string())
        .bind(user2_id.to_string())
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn update(
        pool: &PgPool,
        id: &Uuid,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Channel, sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        let mut query = String::from("UPDATE channels SET updated_at = ?");
        let mut params: Vec<String> = vec![now.clone()];

        if let Some(n) = name {
            query.push_str(", name = ?");
            params.push(n.to_string());
        }

        if let Some(d) = description {
            query.push_str(", description = ?");
            params.push(d.to_string());
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

    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(r#"DELETE FROM channels WHERE id = ?"#)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_member_count(pool: &PgPool, channel_id: &Uuid) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM channel_members WHERE channel_id = ?"#,
        )
        .bind(channel_id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }

    pub async fn add_member(
        pool: &PgPool,
        channel_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<ChannelMember, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO channel_members (id, channel_id, user_id, joined_at)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(channel_id.to_string())
        .bind(user_id.to_string())
        .bind(&now)
        .execute(pool)
        .await?;

        Self::find_member(pool, channel_id, user_id).await
    }

    pub async fn find_member(
        pool: &PgPool,
        channel_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<ChannelMember, sqlx::Error> {
        let row: ChannelMemberRow = sqlx::query_as(
            r#"SELECT * FROM channel_members WHERE channel_id = ? AND user_id = ?"#,
        )
        .bind(channel_id.to_string())
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_members(pool: &PgPool, channel_id: &Uuid) -> Result<Vec<ChannelMember>, sqlx::Error> {
        let rows: Vec<ChannelMemberRow> = sqlx::query_as(
            r#"SELECT * FROM channel_members WHERE channel_id = ? ORDER BY joined_at"#,
        )
        .bind(channel_id.to_string())
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn remove_member(
        pool: &PgPool,
        channel_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"DELETE FROM channel_members WHERE channel_id = ? AND user_id = ?"#,
        )
        .bind(channel_id.to_string())
        .bind(user_id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn is_member(
        pool: &PgPool,
        channel_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM channel_members WHERE channel_id = ? AND user_id = ?"#,
        )
        .bind(channel_id.to_string())
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }

    pub async fn mark_as_read(
        pool: &PgPool,
        channel_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"UPDATE channel_members SET last_read_at = ? WHERE channel_id = ? AND user_id = ?"#,
        )
        .bind(&now)
        .bind(channel_id.to_string())
        .bind(user_id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_unread_count(
        pool: &PgPool,
        channel_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM messages m
            INNER JOIN channel_members cm ON m.channel_id = cm.channel_id
            WHERE m.channel_id = ?
            AND cm.user_id = ?
            AND (cm.last_read_at IS NULL OR m.created_at > cm.last_read_at)
            "#,
        )
        .bind(channel_id.to_string())
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }
}

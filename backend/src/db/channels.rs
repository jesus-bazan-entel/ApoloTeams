//! Channel database operations

use chrono::{DateTime, Utc};
use shared::models::{Channel, ChannelMember, ChannelType};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct ChannelRow {
    pub id: Uuid,
    pub team_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub channel_type: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ChannelRow> for Channel {
    fn from(row: ChannelRow) -> Self {
        Channel {
            id: row.id,
            team_id: row.team_id,
            name: row.name,
            description: row.description,
            channel_type: serde_json::from_str(&format!("\"{}\"", row.channel_type)).unwrap_or_default(),
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct ChannelMemberRow {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
    pub last_read_at: Option<DateTime<Utc>>,
}

impl From<ChannelMemberRow> for ChannelMember {
    fn from(row: ChannelMemberRow) -> Self {
        ChannelMember {
            id: row.id,
            channel_id: row.channel_id,
            user_id: row.user_id,
            joined_at: row.joined_at,
            last_read_at: row.last_read_at,
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
        let id = Uuid::new_v4();
        let now = Utc::now();
        let type_str = serde_json::to_string(&channel_type).unwrap().trim_matches('"').to_string();

        sqlx::query(
            r#"
            INSERT INTO channels (id, team_id, name, description, channel_type, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&id)
        .bind(team_id)
        .bind(name)
        .bind(description)
        .bind(&type_str)
        .bind(created_by)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        // Add creator as a member
        let member_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO channel_members (id, channel_id, user_id, joined_at)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(&member_id)
        .bind(&id)
        .bind(created_by)
        .bind(&now)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, &id).await
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Channel, sqlx::Error> {
        let row: ChannelRow = sqlx::query_as(
            r#"SELECT id, team_id, name, description, channel_type, created_by, created_at, updated_at FROM channels WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_team(pool: &PgPool, team_id: &Uuid) -> Result<Vec<Channel>, sqlx::Error> {
        let rows: Vec<ChannelRow> = sqlx::query_as(
            r#"SELECT id, team_id, name, description, channel_type, created_by, created_at, updated_at FROM channels WHERE team_id = $1 ORDER BY name"#,
        )
        .bind(team_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn find_by_user(pool: &PgPool, user_id: &Uuid) -> Result<Vec<Channel>, sqlx::Error> {
        let rows: Vec<ChannelRow> = sqlx::query_as(
            r#"
            SELECT c.id, c.team_id, c.name, c.description, c.channel_type, c.created_by, c.created_at, c.updated_at 
            FROM channels c
            INNER JOIN channel_members cm ON c.id = cm.channel_id
            WHERE cm.user_id = $1
            ORDER BY c.name
            "#,
        )
        .bind(user_id)
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
            SELECT c.id, c.team_id, c.name, c.description, c.channel_type, c.created_by, c.created_at, c.updated_at 
            FROM channels c
            INNER JOIN channel_members cm1 ON c.id = cm1.channel_id
            INNER JOIN channel_members cm2 ON c.id = cm2.channel_id
            WHERE c.channel_type = 'direct_message'
            AND cm1.user_id = $1
            AND cm2.user_id = $2
            "#,
        )
        .bind(user1_id)
        .bind(user2_id)
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
        let now = Utc::now();
        let mut set_clauses = vec!["updated_at = $1".to_string()];
        let mut param_index = 2;

        if name.is_some() {
            set_clauses.push(format!("name = ${}", param_index));
            param_index += 1;
        }

        if description.is_some() {
            set_clauses.push(format!("description = ${}", param_index));
            param_index += 1;
        }

        let query = format!(
            "UPDATE channels SET {} WHERE id = ${}",
            set_clauses.join(", "),
            param_index
        );

        let mut q = sqlx::query(&query).bind(&now);
        
        if let Some(n) = name {
            q = q.bind(n);
        }
        if let Some(d) = description {
            q = q.bind(d);
        }
        
        q = q.bind(id);
        q.execute(pool).await?;

        Self::find_by_id(pool, id).await
    }

    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(r#"DELETE FROM channels WHERE id = $1"#)
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_member_count(pool: &PgPool, channel_id: &Uuid) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM channel_members WHERE channel_id = $1"#,
        )
        .bind(channel_id)
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }

    pub async fn add_member(
        pool: &PgPool,
        channel_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<ChannelMember, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO channel_members (id, channel_id, user_id, joined_at)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(&id)
        .bind(channel_id)
        .bind(user_id)
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
            r#"SELECT id, channel_id, user_id, joined_at, last_read_at FROM channel_members WHERE channel_id = $1 AND user_id = $2"#,
        )
        .bind(channel_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_members(pool: &PgPool, channel_id: &Uuid) -> Result<Vec<ChannelMember>, sqlx::Error> {
        let rows: Vec<ChannelMemberRow> = sqlx::query_as(
            r#"SELECT id, channel_id, user_id, joined_at, last_read_at FROM channel_members WHERE channel_id = $1 ORDER BY joined_at"#,
        )
        .bind(channel_id)
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
            r#"DELETE FROM channel_members WHERE channel_id = $1 AND user_id = $2"#,
        )
        .bind(channel_id)
        .bind(user_id)
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
            r#"SELECT COUNT(*) FROM channel_members WHERE channel_id = $1 AND user_id = $2"#,
        )
        .bind(channel_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }

    pub async fn mark_as_read(
        pool: &PgPool,
        channel_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"UPDATE channel_members SET last_read_at = $1 WHERE channel_id = $2 AND user_id = $3"#,
        )
        .bind(&now)
        .bind(channel_id)
        .bind(user_id)
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
            WHERE m.channel_id = $1
            AND cm.user_id = $2
            AND (cm.last_read_at IS NULL OR m.created_at > cm.last_read_at)
            "#,
        )
        .bind(channel_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }
}

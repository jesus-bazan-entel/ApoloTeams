//! Team database operations

use chrono::{DateTime, Utc};
use shared::models::{Team, TeamMember, TeamRole};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct TeamRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub owner_id: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<TeamRow> for Team {
    fn from(row: TeamRow) -> Self {
        Team {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            name: row.name,
            description: row.description,
            avatar_url: row.avatar_url,
            owner_id: Uuid::parse_str(&row.owner_id).unwrap_or_default(),
            created_at: DateTime::parse_from_rfc3339(&row.created_at).unwrap().with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.updated_at).unwrap().with_timezone(&Utc),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct TeamMemberRow {
    pub id: String,
    pub team_id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: String,
}

impl From<TeamMemberRow> for TeamMember {
    fn from(row: TeamMemberRow) -> Self {
        TeamMember {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            team_id: Uuid::parse_str(&row.team_id).unwrap_or_default(),
            user_id: Uuid::parse_str(&row.user_id).unwrap_or_default(),
            role: serde_json::from_str(&format!("\"{}\"", row.role)).unwrap_or_default(),
            joined_at: DateTime::parse_from_rfc3339(&row.joined_at).unwrap().with_timezone(&Utc),
        }
    }
}

pub struct TeamRepository;

impl TeamRepository {
    pub async fn create(
        pool: &PgPool,
        name: &str,
        description: Option<&str>,
        avatar_url: Option<&str>,
        owner_id: &Uuid,
    ) -> Result<Team, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO teams (id, name, description, avatar_url, owner_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(name)
        .bind(description)
        .bind(avatar_url)
        .bind(owner_id.to_string())
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        // Add owner as a member with Owner role
        let member_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO team_members (id, team_id, user_id, role, joined_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&member_id)
        .bind(&id)
        .bind(owner_id.to_string())
        .bind("owner")
        .bind(&now)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, &Uuid::parse_str(&id).unwrap()).await
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Team, sqlx::Error> {
        let row: TeamRow = sqlx::query_as(
            r#"SELECT * FROM teams WHERE id = ?"#,
        )
        .bind(id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_user(pool: &PgPool, user_id: &Uuid) -> Result<Vec<Team>, sqlx::Error> {
        let rows: Vec<TeamRow> = sqlx::query_as(
            r#"
            SELECT t.* FROM teams t
            INNER JOIN team_members tm ON t.id = tm.team_id
            WHERE tm.user_id = ?
            ORDER BY t.name
            "#,
        )
        .bind(user_id.to_string())
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update(
        pool: &PgPool,
        id: &Uuid,
        name: Option<&str>,
        description: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<Team, sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        let mut query = String::from("UPDATE teams SET updated_at = ?");
        let mut params: Vec<String> = vec![now.clone()];

        if let Some(n) = name {
            query.push_str(", name = ?");
            params.push(n.to_string());
        }

        if let Some(d) = description {
            query.push_str(", description = ?");
            params.push(d.to_string());
        }

        if let Some(url) = avatar_url {
            query.push_str(", avatar_url = ?");
            params.push(url.to_string());
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
        sqlx::query(r#"DELETE FROM teams WHERE id = ?"#)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_member_count(pool: &PgPool, team_id: &Uuid) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM team_members WHERE team_id = ?"#,
        )
        .bind(team_id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }

    pub async fn add_member(
        pool: &PgPool,
        team_id: &Uuid,
        user_id: &Uuid,
        role: TeamRole,
    ) -> Result<TeamMember, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let role_str = serde_json::to_string(&role).unwrap().trim_matches('"').to_string();

        sqlx::query(
            r#"
            INSERT INTO team_members (id, team_id, user_id, role, joined_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(team_id.to_string())
        .bind(user_id.to_string())
        .bind(&role_str)
        .bind(&now)
        .execute(pool)
        .await?;

        Self::find_member(pool, team_id, user_id).await
    }

    pub async fn find_member(
        pool: &PgPool,
        team_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<TeamMember, sqlx::Error> {
        let row: TeamMemberRow = sqlx::query_as(
            r#"SELECT * FROM team_members WHERE team_id = ? AND user_id = ?"#,
        )
        .bind(team_id.to_string())
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_members(pool: &PgPool, team_id: &Uuid) -> Result<Vec<TeamMember>, sqlx::Error> {
        let rows: Vec<TeamMemberRow> = sqlx::query_as(
            r#"SELECT * FROM team_members WHERE team_id = ? ORDER BY joined_at"#,
        )
        .bind(team_id.to_string())
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_member_role(
        pool: &PgPool,
        team_id: &Uuid,
        user_id: &Uuid,
        role: TeamRole,
    ) -> Result<TeamMember, sqlx::Error> {
        let role_str = serde_json::to_string(&role).unwrap().trim_matches('"').to_string();

        sqlx::query(
            r#"UPDATE team_members SET role = ? WHERE team_id = ? AND user_id = ?"#,
        )
        .bind(&role_str)
        .bind(team_id.to_string())
        .bind(user_id.to_string())
        .execute(pool)
        .await?;

        Self::find_member(pool, team_id, user_id).await
    }

    pub async fn remove_member(
        pool: &PgPool,
        team_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"DELETE FROM team_members WHERE team_id = ? AND user_id = ?"#,
        )
        .bind(team_id.to_string())
        .bind(user_id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn is_member(
        pool: &PgPool,
        team_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM team_members WHERE team_id = ? AND user_id = ?"#,
        )
        .bind(team_id.to_string())
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(result.0 > 0)
    }

    pub async fn get_user_role(
        pool: &PgPool,
        team_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<Option<TeamRole>, sqlx::Error> {
        let result: Option<(String,)> = sqlx::query_as(
            r#"SELECT role FROM team_members WHERE team_id = ? AND user_id = ?"#,
        )
        .bind(team_id.to_string())
        .bind(user_id.to_string())
        .fetch_optional(pool)
        .await?;

        Ok(result.map(|(role,)| serde_json::from_str(&format!("\"{}\"", role)).unwrap_or_default()))
    }
}

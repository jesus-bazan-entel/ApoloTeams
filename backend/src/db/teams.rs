//! Team database operations

use chrono::{DateTime, Utc};
use shared::models::{Team, TeamMember, TeamRole};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct TeamRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TeamRow> for Team {
    fn from(row: TeamRow) -> Self {
        Team {
            id: row.id,
            name: row.name,
            description: row.description,
            avatar_url: row.avatar_url,
            owner_id: row.owner_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct TeamMemberRow {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

impl From<TeamMemberRow> for TeamMember {
    fn from(row: TeamMemberRow) -> Self {
        TeamMember {
            id: row.id,
            team_id: row.team_id,
            user_id: row.user_id,
            role: serde_json::from_str(&format!("\"{}\"", row.role)).unwrap_or_default(),
            joined_at: row.joined_at,
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
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO teams (id, name, description, avatar_url, owner_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(&id)
        .bind(name)
        .bind(description)
        .bind(avatar_url)
        .bind(owner_id)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        // Add owner as a member with Owner role
        let member_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO team_members (id, team_id, user_id, role, joined_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(&member_id)
        .bind(&id)
        .bind(owner_id)
        .bind("owner")
        .bind(&now)
        .execute(pool)
        .await?;

        Self::find_by_id(pool, &id).await
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Team, sqlx::Error> {
        let row: TeamRow = sqlx::query_as(
            r#"SELECT id, name, description, avatar_url, owner_id, created_at, updated_at FROM teams WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_by_user(pool: &PgPool, user_id: &Uuid) -> Result<Vec<Team>, sqlx::Error> {
        let rows: Vec<TeamRow> = sqlx::query_as(
            r#"
            SELECT t.id, t.name, t.description, t.avatar_url, t.owner_id, t.created_at, t.updated_at 
            FROM teams t
            INNER JOIN team_members tm ON t.id = tm.team_id
            WHERE tm.user_id = $1
            ORDER BY t.name
            "#,
        )
        .bind(user_id)
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

        if avatar_url.is_some() {
            set_clauses.push(format!("avatar_url = ${}", param_index));
            param_index += 1;
        }

        let query = format!(
            "UPDATE teams SET {} WHERE id = ${}",
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
        if let Some(url) = avatar_url {
            q = q.bind(url);
        }
        
        q = q.bind(id);
        q.execute(pool).await?;

        Self::find_by_id(pool, id).await
    }

    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(r#"DELETE FROM teams WHERE id = $1"#)
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_member_count(pool: &PgPool, team_id: &Uuid) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM team_members WHERE team_id = $1"#,
        )
        .bind(team_id)
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
        let id = Uuid::new_v4();
        let now = Utc::now();
        let role_str = serde_json::to_string(&role).unwrap().trim_matches('"').to_string();

        sqlx::query(
            r#"
            INSERT INTO team_members (id, team_id, user_id, role, joined_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(&id)
        .bind(team_id)
        .bind(user_id)
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
            r#"SELECT id, team_id, user_id, role, joined_at FROM team_members WHERE team_id = $1 AND user_id = $2"#,
        )
        .bind(team_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(row.into())
    }

    pub async fn find_members(pool: &PgPool, team_id: &Uuid) -> Result<Vec<TeamMember>, sqlx::Error> {
        let rows: Vec<TeamMemberRow> = sqlx::query_as(
            r#"SELECT id, team_id, user_id, role, joined_at FROM team_members WHERE team_id = $1 ORDER BY joined_at"#,
        )
        .bind(team_id)
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
            r#"UPDATE team_members SET role = $1 WHERE team_id = $2 AND user_id = $3"#,
        )
        .bind(&role_str)
        .bind(team_id)
        .bind(user_id)
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
            r#"DELETE FROM team_members WHERE team_id = $1 AND user_id = $2"#,
        )
        .bind(team_id)
        .bind(user_id)
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
            r#"SELECT COUNT(*) FROM team_members WHERE team_id = $1 AND user_id = $2"#,
        )
        .bind(team_id)
        .bind(user_id)
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
            r#"SELECT role FROM team_members WHERE team_id = $1 AND user_id = $2"#,
        )
        .bind(team_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(result.map(|(role,)| serde_json::from_str(&format!("\"{}\"", role)).unwrap_or_default()))
    }
}

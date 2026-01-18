//! Team service

use shared::dto::{
    CreateTeamRequest, TeamMemberResponse, TeamResponse, UpdateTeamRequest, UserResponse,
};
use shared::error::AppError;
use shared::models::TeamRole;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::{TeamRepository, UserRepository};

pub struct TeamService {
    pool: Arc<PgPool>,
}

impl TeamService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_team(
        &self,
        owner_id: &Uuid,
        request: CreateTeamRequest,
    ) -> Result<TeamResponse, AppError> {
        let team = TeamRepository::create(
            &self.pool,
            &request.name,
            request.description.as_deref(),
            request.avatar_url.as_deref(),
            owner_id,
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let member_count = TeamRepository::get_member_count(&self.pool, &team.id)
            .await
            .unwrap_or(0);

        Ok(TeamResponse {
            id: team.id,
            name: team.name,
            description: team.description,
            avatar_url: team.avatar_url,
            owner_id: team.owner_id,
            member_count,
            created_at: team.created_at,
        })
    }

    pub async fn get_team(&self, team_id: &Uuid) -> Result<TeamResponse, AppError> {
        let team = TeamRepository::find_by_id(&self.pool, team_id)
            .await
            .map_err(|_| AppError::NotFoundError("Team not found".to_string()))?;

        let member_count = TeamRepository::get_member_count(&self.pool, &team.id)
            .await
            .unwrap_or(0);

        Ok(TeamResponse {
            id: team.id,
            name: team.name,
            description: team.description,
            avatar_url: team.avatar_url,
            owner_id: team.owner_id,
            member_count,
            created_at: team.created_at,
        })
    }

    pub async fn list_user_teams(&self, user_id: &Uuid) -> Result<Vec<TeamResponse>, AppError> {
        let teams = TeamRepository::find_by_user(&self.pool, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut responses = Vec::new();
        for team in teams {
            let member_count = TeamRepository::get_member_count(&self.pool, &team.id)
                .await
                .unwrap_or(0);

            responses.push(TeamResponse {
                id: team.id,
                name: team.name,
                description: team.description,
                avatar_url: team.avatar_url,
                owner_id: team.owner_id,
                member_count,
                created_at: team.created_at,
            });
        }

        Ok(responses)
    }

    pub async fn update_team(
        &self,
        team_id: &Uuid,
        user_id: &Uuid,
        request: UpdateTeamRequest,
    ) -> Result<TeamResponse, AppError> {
        // Check if user has permission
        self.check_admin_permission(team_id, user_id).await?;

        let team = TeamRepository::update(
            &self.pool,
            team_id,
            request.name.as_deref(),
            request.description.as_deref(),
            request.avatar_url.as_deref(),
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let member_count = TeamRepository::get_member_count(&self.pool, &team.id)
            .await
            .unwrap_or(0);

        Ok(TeamResponse {
            id: team.id,
            name: team.name,
            description: team.description,
            avatar_url: team.avatar_url,
            owner_id: team.owner_id,
            member_count,
            created_at: team.created_at,
        })
    }

    pub async fn delete_team(&self, team_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        // Only owner can delete team
        let team = TeamRepository::find_by_id(&self.pool, team_id)
            .await
            .map_err(|_| AppError::NotFoundError("Team not found".to_string()))?;

        if team.owner_id != *user_id {
            return Err(AppError::AuthorizationError(
                "Only the team owner can delete the team".to_string(),
            ));
        }

        TeamRepository::delete(&self.pool, team_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_members(&self, team_id: &Uuid) -> Result<Vec<TeamMemberResponse>, AppError> {
        let members = TeamRepository::find_members(&self.pool, team_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut responses = Vec::new();
        for member in members {
            let user = UserRepository::find_by_id(&self.pool, &member.user_id)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            responses.push(TeamMemberResponse {
                id: member.id,
                user: UserResponse {
                    id: user.id,
                    email: user.email,
                    username: user.username,
                    display_name: user.display_name,
                    avatar_url: user.avatar_url,
                    status: user.status,
                    status_message: user.status_message,
                    last_seen: user.last_seen,
                    created_at: user.created_at,
                },
                role: member.role,
                joined_at: member.joined_at,
            });
        }

        Ok(responses)
    }

    pub async fn add_member(
        &self,
        team_id: &Uuid,
        requester_id: &Uuid,
        user_id: &Uuid,
        role: Option<TeamRole>,
    ) -> Result<TeamMemberResponse, AppError> {
        // Check if requester has permission
        self.check_admin_permission(team_id, requester_id).await?;

        // Check if user is already a member
        if TeamRepository::is_member(&self.pool, team_id, user_id)
            .await
            .unwrap_or(false)
        {
            return Err(AppError::ConflictError("User is already a team member".to_string()));
        }

        let member = TeamRepository::add_member(
            &self.pool,
            team_id,
            user_id,
            role.unwrap_or(TeamRole::Member),
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let user = UserRepository::find_by_id(&self.pool, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(TeamMemberResponse {
            id: member.id,
            user: UserResponse {
                id: user.id,
                email: user.email,
                username: user.username,
                display_name: user.display_name,
                avatar_url: user.avatar_url,
                status: user.status,
                status_message: user.status_message,
                last_seen: user.last_seen,
                created_at: user.created_at,
            },
            role: member.role,
            joined_at: member.joined_at,
        })
    }

    pub async fn update_member_role(
        &self,
        team_id: &Uuid,
        requester_id: &Uuid,
        user_id: &Uuid,
        role: TeamRole,
    ) -> Result<TeamMemberResponse, AppError> {
        // Check if requester has permission
        self.check_admin_permission(team_id, requester_id).await?;

        // Cannot change owner's role
        let team = TeamRepository::find_by_id(&self.pool, team_id)
            .await
            .map_err(|_| AppError::NotFoundError("Team not found".to_string()))?;

        if team.owner_id == *user_id && role != TeamRole::Owner {
            return Err(AppError::BadRequest("Cannot change owner's role".to_string()));
        }

        let member = TeamRepository::update_member_role(&self.pool, team_id, user_id, role)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let user = UserRepository::find_by_id(&self.pool, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(TeamMemberResponse {
            id: member.id,
            user: UserResponse {
                id: user.id,
                email: user.email,
                username: user.username,
                display_name: user.display_name,
                avatar_url: user.avatar_url,
                status: user.status,
                status_message: user.status_message,
                last_seen: user.last_seen,
                created_at: user.created_at,
            },
            role: member.role,
            joined_at: member.joined_at,
        })
    }

    pub async fn remove_member(
        &self,
        team_id: &Uuid,
        requester_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), AppError> {
        // Check if requester has permission (or is removing themselves)
        if requester_id != user_id {
            self.check_admin_permission(team_id, requester_id).await?;
        }

        // Cannot remove owner
        let team = TeamRepository::find_by_id(&self.pool, team_id)
            .await
            .map_err(|_| AppError::NotFoundError("Team not found".to_string()))?;

        if team.owner_id == *user_id {
            return Err(AppError::BadRequest(
                "Cannot remove team owner. Transfer ownership first.".to_string(),
            ));
        }

        TeamRepository::remove_member(&self.pool, team_id, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn is_member(&self, team_id: &Uuid, user_id: &Uuid) -> Result<bool, AppError> {
        TeamRepository::is_member(&self.pool, team_id, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    async fn check_admin_permission(&self, team_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        let role = TeamRepository::get_user_role(&self.pool, team_id, user_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        match role {
            Some(TeamRole::Owner) | Some(TeamRole::Admin) => Ok(()),
            Some(_) => Err(AppError::AuthorizationError(
                "You don't have permission to perform this action".to_string(),
            )),
            None => Err(AppError::AuthorizationError(
                "You are not a member of this team".to_string(),
            )),
        }
    }
}

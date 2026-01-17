//! Team handlers

use actix_web::{web, HttpRequest, HttpResponse};
use shared::dto::{AddTeamMemberRequest, CreateTeamRequest, UpdateTeamMemberRequest, UpdateTeamRequest};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::error::{ApiError, ApiResult};
use crate::middleware::get_user_id_from_request;
use crate::services::Services;

pub async fn list_teams(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let teams = services.teams.list_user_teams(&user_id).await?;
    Ok(HttpResponse::Ok().json(teams))
}

pub async fn create_team(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    body: web::Json<CreateTeamRequest>,
) -> ApiResult<HttpResponse> {
    body.validate().map_err(ApiError::from)?;

    let user_id = get_user_id_from_request(&req, &services)?;
    let team = services.teams.create_team(&user_id, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(team))
}

pub async fn get_team(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let team_id = path.into_inner();

    // Verify user is a member
    if !services.teams.is_member(&team_id, &user_id).await? {
        return Err(ApiError(shared::error::AppError::AuthorizationError(
            "You are not a member of this team".to_string(),
        )));
    }

    let team = services.teams.get_team(&team_id).await?;
    Ok(HttpResponse::Ok().json(team))
}

pub async fn update_team(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateTeamRequest>,
) -> ApiResult<HttpResponse> {
    body.validate().map_err(ApiError::from)?;

    let user_id = get_user_id_from_request(&req, &services)?;
    let team_id = path.into_inner();

    let team = services
        .teams
        .update_team(&team_id, &user_id, body.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(team))
}

pub async fn delete_team(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let team_id = path.into_inner();

    services.teams.delete_team(&team_id, &user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn list_team_members(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let team_id = path.into_inner();

    // Verify user is a member
    if !services.teams.is_member(&team_id, &user_id).await? {
        return Err(ApiError(shared::error::AppError::AuthorizationError(
            "You are not a member of this team".to_string(),
        )));
    }

    let members = services.teams.list_members(&team_id).await?;
    Ok(HttpResponse::Ok().json(members))
}

pub async fn add_team_member(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<Uuid>,
    body: web::Json<AddTeamMemberRequest>,
) -> ApiResult<HttpResponse> {
    let user_id = get_user_id_from_request(&req, &services)?;
    let team_id = path.into_inner();

    let member = services
        .teams
        .add_member(&team_id, &user_id, &body.user_id, body.role.clone())
        .await?;
    Ok(HttpResponse::Created().json(member))
}

#[derive(serde::Deserialize)]
pub struct TeamMemberPath {
    team_id: Uuid,
    user_id: Uuid,
}

pub async fn update_team_member(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<TeamMemberPath>,
    body: web::Json<UpdateTeamMemberRequest>,
) -> ApiResult<HttpResponse> {
    let requester_id = get_user_id_from_request(&req, &services)?;
    let params = path.into_inner();

    let member = services
        .teams
        .update_member_role(&params.team_id, &requester_id, &params.user_id, body.role.clone())
        .await?;
    Ok(HttpResponse::Ok().json(member))
}

pub async fn remove_team_member(
    req: HttpRequest,
    services: web::Data<Arc<Services>>,
    path: web::Path<TeamMemberPath>,
) -> ApiResult<HttpResponse> {
    let requester_id = get_user_id_from_request(&req, &services)?;
    let params = path.into_inner();

    services
        .teams
        .remove_member(&params.team_id, &requester_id, &params.user_id)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

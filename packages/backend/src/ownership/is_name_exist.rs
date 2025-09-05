use axum::{extract::{Query, State}, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use diesel::prelude::*;
use std::sync::Arc;
use crate::schema::users_info;
use crate::config::app_state::AppState;

#[derive(Deserialize, ToSchema)]
pub struct UserExistsQuery {
    #[schema(example = "john_doe")]
    username: String,
}

#[derive(Serialize, ToSchema)]
pub struct UserExistsResponse {
    exists: bool,
}

#[utoipa::path(
    get,
    path = "/api/user/exists",
    params(
        ("username" = String, Query, description = "User's username", example = "john_doe")
    ),
    responses(
        (status = 200, description = "Check if user exists", body = UserExistsResponse, example = json!({
            "exists": true
        })),
        (status = 400, description = "Username not provided"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users"
)]
pub async fn user_exists(
    Query(query): Query<UserExistsQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let conn = &mut state.db_pool.get().map_err(|e| {
        eprintln!("Failed to get DB connection: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Internal server error: {}", e)})),
        ).into_response()
    }).unwrap();

    // Check if user exists
    let exists: bool = users_info::table
        .filter(users_info::username.eq(&query.username))
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .map(|count| count > 0)
        .map_err(|e| {
            eprintln!("Error checking user existence for {}: {:?}", query.username, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Internal server error: {}", e)})),
            ).into_response()
        }).unwrap();

    (
        StatusCode::OK,
        Json(UserExistsResponse { exists }),
    ).into_response()
}
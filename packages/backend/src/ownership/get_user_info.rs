use crate::config::app_state::AppState;
use crate::contract_models::UserInfo;
use crate::schema::users_info;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct UserQuery {
    #[schema(example = "0x1234567890abcdef1234567890abcdef12345678")]
    user_address: Option<String>,
    #[schema(example = "john_doe")]
    username: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    user_address: String,
    username: String,
    is_registered: bool,
    created_at: String,
    tnx_hash: String,
}

#[utoipa::path(
    get,
    path = "/api/user/get",
    params(
        ("user_address" = Option<String>, Query, description = "User's blockchain address (e.g., 0x...), case-insensitive"),
        ("username" = Option<String>, Query, description = "User's username", example = "john_doe")
    ),
    responses(
        (status = 200, description = "User found", body = UserResponse, example = json!({
            "user_address": "0x1234567890abcdef1234567890abcdef12345678",
            "username": "john_doe",
            "is_registered": true,
            "created_at": "2025-08-25 19:22:00",
            "tnx_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        })),
        (status = 400, description = "Neither user_address nor username provided"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users"
)]
pub async fn get_user(
    Query(query): Query<UserQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Ensure at least one parameter is provided
    if query.user_address.is_none() && query.username.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Either user_address or username must be provided"})),
        )
            .into_response();
    }

    let conn = &mut state
        .db_pool
        .get()
        .map_err(|e| {
            eprintln!("Failed to get DB connection: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Internal server error: {}", e)})),
            )
                .into_response()
        })
        .unwrap();

    // Build the query
    let mut user_query = users_info::table.into_boxed();

    if let Some(address) = query.user_address {
        // Normalize address to lowercase for case-insensitive search
        user_query = user_query.filter(users_info::user_address.eq(address));
    } else if let Some(username) = query.username {
        user_query = user_query.filter(users_info::username.eq(username));
    }

    // Execute the query
    match user_query.first::<UserInfo>(conn) {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(DieselError::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "User not found"})),
        )
            .into_response(),
        Err(e) => {
            eprintln!("Error fetching user: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Internal server error: {}", e)})),
            )
                .into_response()
        }
    }
}

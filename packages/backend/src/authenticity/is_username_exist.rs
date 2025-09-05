use crate::config::app_state::AppState;
use crate::schema::manufacturers;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use diesel::associations::HasTable;
use diesel::{prelude::*, QueryDsl};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct IsExistsQuery {
    #[schema(example = "john_doe")]
    username: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct IsExistsResponse {
    exists: bool,
}

#[utoipa::path(
    get,
    path = "/api/manufacturer/exists",
    params(
        ("username" = Option<String>, Query, description = "Manufacturer's username", example = "john_doe")
    ),
    responses(
        (status = 200, description = "Check if manufacturer exists", body = IsExistsResponse, example = json!({
            "exists": true
        })),
        (status = 400, description = "Username not provided"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Manufacturers"
)]
pub async fn manufacturer_name_exists(
    Query(query): Query<IsExistsQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    eprintln!("username: {:?}", query.username);
    match check_manufacturer_exists_internal(&state, &query).await {
        Ok(exists) => (
            StatusCode::OK,
            Json(IsExistsResponse { exists }),
        ).into_response(),
        Err(e) => {
            eprintln!("Error checking manufacturer existence: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Internal server error: {}", e)})),
            ).into_response()
        }
    }
}

async fn check_manufacturer_exists_internal(
    state: &Arc<AppState>,
    query: &IsExistsQuery,
) -> Result<bool> {
    eprintln!("username: {:?}", query.username);
    let username = query.username.as_ref().ok_or_else(|| {
        eprintln!("Username not provided");
        eyre::eyre!("Username must be provided")
    })?;

    let mut conn = state.db_pool.get().map_err(|e| {
        eprintln!("Failed to get DB connection: {:?}", e);
        eyre::eyre!("Failed to get DB connection: {}", e)
    })?;

    let exists: bool = manufacturers::table
        .filter(manufacturers::manufacturer_name.eq(username))
        .select(diesel::dsl::count_star())
        .first::<i64>(&mut conn)
        .map(|count| count > 0)
        .map_err(|e| {
            eprintln!("Failed to check manufacturer with username {}: {:?}", username, e);
            eyre::eyre!("Failed to check manufacturer: {}", e)
        })?;

    Ok(exists)
}


//TODO: I WANT TO KEEP IT

// async fn check_manufacturer_exists_internal(
//     state: &Arc<AppState>,
//     query: &IsExistsQuery,
// ) -> Result<bool> {
//     let username = query.manufacturer_name.as_ref().ok_or_else(|| {
//         eprintln!("Username not provided");
//         eyre::eyre!("Username must be provided")
//     })?;
//
//     let mut conn = state.db_pool.get().map_err(|e| {
//         eprintln!("Failed to get DB connection: {:?}", e);
//         eyre::eyre!("Failed to get DB connection: {}", e)
//     })?;
//
//     let exists = manufacturers::table
//         .filter(manufacturers::manufacturer_name.eq(username))
//         .select(Manufacturer::as_select())
//         .first::<Manufacturer>(&mut conn)
//         .optional()
//         .map_err(|e| {
//             eprintln!("Failed to check manufacturer with username {}: {:?}", username, e);
//             eyre::eyre!("Failed to check manufacturer: {}", e)
//         })?
//         .is_some();
//
//     Ok(exists)
// }

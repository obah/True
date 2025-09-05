use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use diesel::prelude::*;
use eyre::Result;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;
use crate::config::app_state::AppState;


// Request structure for the endpoint
#[derive(Deserialize, ToSchema)]
pub struct OwnershipCheckQuery {
    #[schema(example = "item123")]
    pub ownership_code: String,
    #[schema(example = "0x1234567890abcdef1234567890abcdef12345678")]
    pub caller: String,
}

// Response structure for the endpoint
#[derive(serde::Serialize, ToSchema)]
pub struct OwnershipCheckResponse {
    is_temp_owner: bool,
}

/// Check if the caller is the temporary owner for the given item ID
#[utoipa::path(
    post,
    path = "/api/ownership/check_temp_owner",
    request_body = OwnershipCheckQuery,
    responses(
        (status = 200, description = "Successfully checked temporary owner status", body = OwnershipCheckResponse),
        (status = 400, description = "Invalid request parameters", body = serde_json::Value),
        (status = 404, description = "Item ID not found", body = serde_json::Value),
        (status = 500, description = "Internal server error", body = serde_json::Value)
    ),
    tag = "Ownership"
)]
pub async fn check_before_claim(
    Query(query): Query<OwnershipCheckQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match check_temp_owner_internal(&state, &query).await {
        Ok(is_temp_owner) => (
            StatusCode::OK,
            Json(OwnershipCheckResponse { is_temp_owner }),
        ).into_response(),
        Err(e) => {
            eprintln!("Error checking temp owner for item {}: {:?}", query.ownership_code, e);
            let (status, message) = match e.to_string().as_str() {
                "Ownership Code not found" => (StatusCode::NOT_FOUND, e.to_string()),
                "Caller is not the temp_owner" => (StatusCode::BAD_REQUEST, e.to_string()),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal server error: {}", e),
                ),
            };
            (status, Json(serde_json::json!({"error": message}))).into_response()
        }
    }
}

async fn check_temp_owner_internal(state: &Arc<AppState>, query: &OwnershipCheckQuery) -> Result<bool> {
    use crate::schema::ownership_codes::dsl::*;

    let connection = &mut state.db_pool.get()
        .map_err(|e| eyre::eyre!("Failed to get DB connection: {}", e))?;

    let result = ownership_codes
        .filter(ownership_code.eq(&query.ownership_code))
        .select(temp_owner)
        .first::<String>(connection)
        .optional()
        .map_err(|e| eyre::eyre!("Failed to query database: {}", e))?;

    let t_owner = result.ok_or_else(|| eyre::eyre!("Item ID not found"))?;

    // Compare caller with temp_owner (case-insensitive to handle check summed addresses)
    if query.caller.to_lowercase() != t_owner.to_lowercase() {
        return Err(eyre::eyre!("Caller is not the temp_owner"));
    }

    Ok(true)
}
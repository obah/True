use crate::config::app_state::AppState;
use crate::schema::ownership_codes;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use diesel::prelude::*;
use diesel::RunQueryDsl;
use eyre::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use utoipa::OpenApi;

// Define the query struct for the endpoint
#[derive(Deserialize, utoipa::ToSchema)]
pub struct OwnershipQuery {
    #[schema(example = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890")]
    pub ownership_code: String,
    #[schema(example = "0x1234567890abcdef1234567890abcdef12345678")]
    pub caller: String,
}

// Define the response struct for successful ownership retrieval
#[derive(Serialize, utoipa::ToSchema)]
pub struct OwnershipResponse {
    ownership_code: String
}

// Define the error response struct
#[derive(Serialize, utoipa::ToSchema)]
struct ErrorResponse {
    error: String,
}

#[utoipa::path(
    post,
    path = "/api/revoke_ownership_code",
    params(
        ("ownership_code" = String, Query, description = "Ownership code to verify", example = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"),
        ("caller" = String, Query, description = "Address of the caller", example = "0x1234567890abcdef1234567890abcdef12345678")
    ),
    responses(
        (status = 200, description = "Ownership verified and deleted successfully", body = OwnershipResponse, example = json!({
            "item_id": "item123",
            "item_owner": "0x1234567890abcdef1234567890abcdef12345678",
            "temp_owner": "0xabcdef1234567890abcdef1234567890abcdef12",
            "created_at": "2025-08-26T15:54:00+00:00"
        })),
        (status = 400, description = "Invalid input (e.g., caller is not the item owner)", body = ErrorResponse, example = json!({"error": "Caller is not the item owner"})),
        (status = 404, description = "Ownership code not found", body = ErrorResponse, example = json!({"error": "Ownership code not found"})),
        (status = 500, description = "Internal server error", body = ErrorResponse, example = json!({"error": "Internal server error: Database error"}))
    ),
    tag = "Ownership"
)]
pub async fn revoke_ownership_code(
    Query(query): Query<OwnershipQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match verify_and_delete_ownership_internal(&state, &query).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => {
            eprintln!(
                "Error verifying ownership code {}: {:?}",
                query.ownership_code, e
            );
            let (status, message) = match e.to_string().as_str() {
                "Caller is not the item owner" => (StatusCode::BAD_REQUEST, e.to_string()),
                "Ownership code not found" => (StatusCode::NOT_FOUND, e.to_string()),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal server error: {}", e),
                ),
            };
            (status, Json(json!({"error": message}))).into_response()
        }
    }
}


async fn verify_and_delete_ownership_internal(
    state: &Arc<AppState>,
    query: &OwnershipQuery,
) -> Result<OwnershipResponse> {
    let conn = &mut state.db_pool.get().map_err(|e| {
        eprintln!("Failed to get DB connection: {:?}", e);
        eyre::eyre!("Failed to get DB connection: {}", e)
    })?;
    eprintln!("Caller: {:?}", query.caller);

    // Delete the record from ownership_codes where ownership_code and item_owner match
    let deleted_rows = diesel::delete(
        ownership_codes::table
            .filter(ownership_codes::ownership_code.eq(&query.ownership_code))
            .filter(ownership_codes::item_owner.eq(&query.caller))
    )
        .execute(conn)
        .map_err(|e| {
            eprintln!(
                "Failed to delete ownership code {}: {:?}",
                query.ownership_code, e
            );
            eyre::eyre!("Failed to delete ownership code: {}", e)
        })?;

    // Check if any rows were deleted
    if deleted_rows == 0 {
        // Determine if the ownership_code exists but the caller is not the item_owner
        let exists: bool = ownership_codes::table
            .filter(ownership_codes::ownership_code.eq(&query.ownership_code))
            .select(diesel::dsl::count_star())
            .first::<i64>(conn)
            .map(|count| count > 0)
            .map_err(|e| {
                eprintln!(
                    "Failed to check existence of ownership code {}: {:?}",
                    query.ownership_code, e
                );
                eyre::eyre!("Database query error: {}", e)
            })?;

        return if exists {
            Err(eyre::eyre!("Caller is not the item owner"))
        } else {
            Err(eyre::eyre!("Ownership code not found"))
        }
    }

    // Return the ownership details
    Ok(OwnershipResponse {
        ownership_code: query.ownership_code.clone(),
    })
}
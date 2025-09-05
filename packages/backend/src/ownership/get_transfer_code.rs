use axum::{extract::{Query, State}, http::StatusCode, response::IntoResponse, Json};
use diesel::prelude::*;
use eyre::Result;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::config::app_state::AppState;
use crate::contract_models::OwnershipCode;
use crate::schema::ownership_codes;

#[derive(Deserialize, ToSchema)]
pub struct GetOwnershipCodeQuery {
    #[schema(example = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890")]
    pub ownership_code: String,
    #[schema(example = "0xabcdef1234567890abcdef1234567890abcdef12")]
    pub caller: String,
}

#[utoipa::path(
    get,
    path = "/api/get_transfer_code",
    params(
        ("ownership_code" = String, Query, description = "Ownership code to fetch", example = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"),
        ("caller" = String, Query, description = "Address of the caller (must match temp_owner)", example = "0xabcdef1234567890abcdef1234567890abcdef12")
    ),
    responses(
        (status = 200, description = "Ownership code found and caller is temp_owner", body = OwnershipCode, example = json!({
            "ownership_code": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "item_owner": "0x1234567890abcdef1234567890abcdef12345678",
            "temp_owner": "0xabcdef1234567890abcdef1234567890abcdef12",
            "created_at": "2025-08-26T00:37:12.345Z",
            "tnx_hash": ""
        })),
        (status = 400, description = "Invalid input (e.g., invalid ownership_code or caller format)"),
        (status = 404, description = "Ownership code not found or caller is not temp_owner"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Ownership"
)]
pub async fn get_ownership_code(
    Query(query): Query<GetOwnershipCodeQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match get_ownership_code_internal(&state, &query).await {
        Ok(ownership_code) => (
            StatusCode::OK,
            Json(ownership_code),
        ).into_response(),
        Err(e) => {
            eprintln!("Error fetching ownership code {}: {:?}", query.ownership_code, e);
            let (status, message) = match e.to_string().as_str() {
                "Invalid ownership_code format" => (StatusCode::BAD_REQUEST, e.to_string()),
                "Invalid caller address format" => (StatusCode::BAD_REQUEST, e.to_string()),
                "Ownership code not found or caller is not temp_owner" => (StatusCode::NOT_FOUND, e.to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal server error: {}", e)),
            };
            (status, Json(serde_json::json!({"error": message}))).into_response()
        }
    }
}

async fn get_ownership_code_internal(
    state: &Arc<AppState>,
    query: &GetOwnershipCodeQuery,
) -> Result<OwnershipCode> {
    
    // Validate ownership_code format (must start with 0x and be 66 characters long)
    if !query.ownership_code.starts_with("0x") || query.ownership_code.len() != 66 {
        return Err(eyre::eyre!("Invalid ownership_code format"));
    }

    // Validate caller address format
    if !query.caller.starts_with("0x") || query.caller.len() != 42 {
        return Err(eyre::eyre!("Invalid caller address format"));
    }

    let conn = &mut state.db_pool.get().map_err(|e| {
        eprintln!("Failed to get DB connection: {:?}", e);
        eyre::eyre!("Failed to get DB connection: {}", e)
    })?;

    // Fetch ownership code and check if caller is temp_owner
    let ownership_code = ownership_codes::table
        .filter(ownership_codes::ownership_code.eq(&query.ownership_code))
        .filter(ownership_codes::temp_owner.eq(query.caller.clone()))
        .select(OwnershipCode::as_select())
        .first(conn)
        .optional()
        .map_err(|e| {
            eprintln!("Error fetching ownership code {}: {:?}", query.ownership_code, e);
            eyre::eyre!("Failed to fetch ownership code: {}", e)
        })?;

    ownership_code.ok_or_else(|| eyre::eyre!("Ownership code not found or caller is not temp_owner"))
}

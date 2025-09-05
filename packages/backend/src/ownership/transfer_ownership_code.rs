use crate::config::app_state::AppState;
use crate::contract_models::OwnershipCode;
use crate::schema::{items, ownership_codes, users_info};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use eyre::Result;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct OwnershipCodeResponse {
    ownership_code: String,
}

#[derive(Deserialize, ToSchema)]
pub struct GenerateOwnershipCodeQuery {
    item_id: String,
    caller: String,
    temp_owner: String,
}

#[utoipa::path(
    get,
    path = "/api/transfer_ownership",
    params(
        ("item_id" = String, Query, description = "ID of the item", example = "item_001"),
        ("caller" = String, Query, description = "Address of the caller", example = "0x1234567890abcdef1234567890abcdef12345678"),
        ("temp_owner" = String, Query, description = "Address of the temporary owner", example = "0xabcdef1234567890abcdef1234567890abcdef12")
    ),
    responses(
        (status = 200, description = "Ownership code generated successfully", body = OwnershipCodeResponse, example = json!({
            "ownership_code": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        })),
        (status = 400, description = "Invalid input (e.g., caller is temp_owner or caller not registered)"),
        (status = 404, description = "Item not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Ownership"
)]
pub async fn transfer_ownership_code(
    Query(query): Query<GenerateOwnershipCodeQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match generate_ownership_code_internal(&state, &query).await {
        Ok(ownership_code) => (
            StatusCode::OK,
            Json(OwnershipCodeResponse { ownership_code }),
        )
            .into_response(),
        Err(e) => {
            eprintln!(
                "Error generating ownership code for item {}: {:?}",
                query.item_id, e
            );
            let (status, message) = match e.to_string().as_str() {
                "Caller cannot be the temporary owner" => (StatusCode::BAD_REQUEST, e.to_string()),
                "Caller is not registered" => (StatusCode::BAD_REQUEST, e.to_string()),
                "Item not found" => (StatusCode::NOT_FOUND, e.to_string()),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal server error: {}", e),
                ),
            };
            (status, Json(serde_json::json!({"error": message}))).into_response()
        }
    }
}

async fn generate_ownership_code_internal(
    state: &Arc<AppState>,
    query: &GenerateOwnershipCodeQuery,
) -> Result<String> {
    if query.caller == query.temp_owner {
        return Err(eyre::eyre!("Caller cannot be the temporary owner"));
    }

    let conn = &mut state.db_pool.get().map_err(|e| {
        eprintln!("Failed to get DB connection: {:?}", e);
        eyre::eyre!("Failed to get DB connection: {}", e)
    })?;

    if !users_info::table
        .filter(users_info::user_address.eq(query.caller.clone()))
        .filter(users_info::is_registered.eq(true))
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .map(|count| count > 0)?
    {
        return Err(eyre::eyre!("Caller is not registered"));
    }
    // Check if item exists and caller is the owner
    let item_exists_and_owned = items::table
        .filter(items::item_id.eq(&query.item_id))
        .filter(items::owner.eq(query.caller.clone()))
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .map(|count| count > 0)
        .map_err(|e| {
            eprintln!(
                "Error checking item existence and ownership {}: {:?}",
                query.item_id, e
            );
            eyre::eyre!("Failed to check item existence and ownership: {}", e)
        })?;

    if !item_exists_and_owned {
        return Err(eyre::eyre!("Item not found or caller is not the owner"));
    }

    // Generate keccak256 hash of caller, temp_owner, item_id, and current timestamp
    let hash_input = format!(
        "{}{}{}{}",
        query.caller,
        query.temp_owner,
        query.item_id,
        Utc::now().to_rfc3339()
    );
    let hash = Keccak256::digest(hash_input.as_bytes());
    let ownership_code = format!("0x{}", hex::encode(hash));
    // Save to ownership_codes table
    diesel::insert_into(ownership_codes::table)
        .values(OwnershipCode {
            ownership_code: ownership_code.clone(),
            item_id: query.item_id.clone(),
            item_owner: query.caller.clone(),
            temp_owner: query.temp_owner.clone(),
            created_at: Utc::now().to_rfc3339(),
        })
        .execute(conn)
        .map_err(|e| {
            eprintln!(
                "Error inserting ownership code for item {}: {:?}",
                query.item_id, e
            );
            eyre::eyre!("Failed to insert ownership code: {}", e)
        })?;

    Ok(ownership_code)
}

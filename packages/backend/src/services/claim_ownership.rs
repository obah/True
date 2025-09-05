use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    Json as AxumJson,
};
use diesel::prelude::*;
use ethers::{
    prelude::*,
    types::{Address, U256},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use utoipa::ToSchema;
use crate::ownership::ownership_abi::TrueOwnership;
use crate::config::app_state::AppState;
use crate::schema::ownership_codes;

// Define the input struct for the endpoint
#[derive(Deserialize, ToSchema)]
pub struct ClaimOwnershipRequest {
    #[schema(example = "item123")]
    pub ownership_code: String,
    #[schema(example = "0x1234567890abcdef1234567890abcdef12345678")]
    pub caller: String,
}

// Define the response struct for successful transaction
#[derive(Serialize, ToSchema)]
pub struct ClaimOwnershipResponse {
    transaction_hash: String,
}

// Define the error response struct
#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[utoipa::path(
    post,
    path = "/api/ownership/claim",
    request_body = ClaimOwnershipRequest,
    responses(
        (status = 200, description = "Ownership claimed successfully", body = ClaimOwnershipResponse, example = json!({
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        })),
        (status = 400, description = "Invalid input (e.g., empty item ID or invalid caller address)", body = ErrorResponse, example = json!({"error": "Invalid caller address"})),
        (status = 403, description = "Unauthorized (e.g., caller does not match temp_owner)", body = ErrorResponse, example = json!({"error": "Caller does not match temp_owner"})),
        (status = 404, description = "Item ID not found in ownership_codes", body = ErrorResponse, example = json!({"error": "Item ID not found"})),
        (status = 500, description = "Internal server error (e.g., contract interaction or database failure)", body = ErrorResponse, example = json!({"error": "Internal server error: Failed to send transaction"}))
    ),
    tag = "Ownership"
)]
pub async fn claim_ownership(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ClaimOwnershipRequest>,
) -> impl IntoResponse {
    match claim_ownership_internal(&state, &request).await {
        Ok(response) => (
            StatusCode::OK,
            AxumJson(response),
        ).into_response(),
        Err(e) => {
            eprintln!("Error claiming ownership for item {}: {:?}", request.ownership_code, e);
            let (status, message) = match e.to_string().as_str() {
                s if s.contains("Item ID cannot be empty") => (StatusCode::BAD_REQUEST, e.to_string()),
                s if s.contains("Invalid caller address") => (StatusCode::BAD_REQUEST, e.to_string()),
                s if s.contains("Item ID not found") => (StatusCode::NOT_FOUND, e.to_string()),
                s if s.contains("Caller does not match temp_owner") => (StatusCode::FORBIDDEN, e.to_string()),
                s if s.contains("Failed to query database") => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
                s if s.contains("ADDRESS_ZERO") => (StatusCode::BAD_REQUEST, "Caller address cannot be zero".to_string()),
                s if s.contains("AUTHENTICITY_NOT_SET") => (StatusCode::INTERNAL_SERVER_ERROR, "Authenticity contract not set".to_string()),
                s if s.contains("Caller not authorized") => (StatusCode::FORBIDDEN, "Caller not authorized to claim ownership".to_string()),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal server error: {}", e),
                ),
            };
            (status, AxumJson(json!({"error": message}))).into_response()
        }
    }
}

async fn claim_ownership_internal(
    state: &Arc<AppState>,
    request: &ClaimOwnershipRequest,
) -> eyre::Result<ClaimOwnershipResponse> {
    // Validate item ID and caller
    if request.ownership_code.is_empty() {
        return Err(eyre::eyre!("Item ID cannot be empty"));
    }
    if request.caller.is_empty() {
        return Err(eyre::eyre!("Caller address cannot be empty"));
    }

    // Parse caller address
    let caller: Address = request
        .caller
        .parse()
        .map_err(|_| eyre::eyre!("Invalid caller address"))?;

    // Query the ownership_codes table to get temp_owner
    let connection = &mut state
        .db_pool
        .get()
        .map_err(|e| eyre::eyre!("Failed to get database connection: {}", e))?;

    let result = ownership_codes::table
        .filter(ownership_codes::item_id.eq(&request.ownership_code))
        .select(ownership_codes::temp_owner)
        .first::<String>(connection)
        .optional()
        .map_err(|e| eyre::eyre!("Failed to query database: {}", e))?;

    let temp_owner = result.ok_or_else(|| eyre::eyre!("Item ID not found"))?;

    // Compare caller with temp_owner (case-insensitive to handle checksummed addresses)
    if request.caller.to_lowercase() != temp_owner.to_lowercase() {
        return Err(eyre::eyre!("Caller is not the temp_owner"));
    }

    // Get the contract and wallet details
    let contract = &state.ownership_contract;
    let wallet_address = contract.client().address();

    // Log addresses for debugging
    eprintln!("Caller: {:?}", caller);
    eprintln!("Wallet (msg.sender): {:?}", wallet_address);
    eprintln!("Temp Owner (from DB): {}", temp_owner);

    // Check wallet balance
    let balance = contract
        .client()
        .get_balance(wallet_address, None)
        .await
        .map_err(|e| eyre::eyre!("Failed to check wallet balance: {}", e))?;

    // Estimate gas with a 20% buffer
    let gas_estimate = contract
        .new_owner_claim_ownership(
            request.ownership_code.clone(),
            caller,
        )
        .estimate_gas()
        .await
        .map_err(|e| {
            let revert_reason = e.decode_revert().unwrap_or_else(|| e.to_string());
            eyre::eyre!("Gas estimation failed: {}", revert_reason)
        })?;
    let gas_limit = gas_estimate * 120 / 100;

    // Get current gas price with a fallback
    let gas_price = contract
        .client()
        .get_gas_price()
        .await
        .unwrap_or(U256::from(2_000_000_000u64));

    // Check if sufficient funds are available
    let required_funds = gas_limit * gas_price;
    if balance < required_funds {
        return Err(eyre::eyre!(
            "Insufficient funds: have {} wei, need {} wei",
            balance, required_funds
        ));
    }

    // Prepare and send the transaction
    let call = contract
        .new_owner_claim_ownership(request.ownership_code.clone(), caller)
        .gas(gas_limit)
        .gas_price(gas_price);

    let pending_tx = call
        .send()
        .await
        .map_err(|e| {
            let revert_reason = e.decode_revert().unwrap_or_else(|| e.to_string());
            eyre::eyre!("Failed to send transaction: {}", revert_reason)
        })?;

    // Await transaction confirmation
    let receipt = pending_tx
        .await
        .map_err(|e| eyre::eyre!("Failed to confirm transaction: {}", e))?
        .ok_or_else(|| eyre::eyre!("Transaction receipt not found"))?;

    Ok(ClaimOwnershipResponse {
        transaction_hash: format!("0x{}", hex::encode(receipt.transaction_hash)),
    })
}
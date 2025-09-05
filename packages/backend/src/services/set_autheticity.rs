use crate::config::app_state::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    Json as AxumJson,
};
use ethers::{
    prelude::*
    ,
    types::{Address, U256},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use utoipa::ToSchema;

// Define the input struct for the endpoint
#[derive(Deserialize, ToSchema)]
pub struct SetAuthenticityRequest {
    #[schema(example = "0xabcdef1234567890abcdef1234567890abcdef12")]
    pub authenticity_address: String,
}

// Define the response struct for successful transaction
#[derive(Serialize, ToSchema)]
pub struct SetAuthenticityResponse {
    transaction_hash: String,
}

// Define the error response struct
#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[utoipa::path(
    post,
    path = "/api/set_authenticity",
    request_body = SetAuthenticityRequest,
    responses(
        (status = 200, description = "Authenticity address set successfully", body = SetAuthenticityResponse, example = json!({
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        })),
        (status = 400, description = "Invalid input (e.g., invalid authenticity address)", body = ErrorResponse, example = json!({"error": "Invalid authenticity address"})),
        (status = 403, description = "Caller is not the contract owner", body = ErrorResponse, example = json!({"error": "Caller is not the contract owner"})),
        (status = 500, description = "Internal server error (e.g., contract interaction failed)", body = ErrorResponse, example = json!({"error": "Internal server error: Failed to send transaction"}))
    ),
    tag = "Ownership"
)]
pub async fn set_authenticity(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SetAuthenticityRequest>,
) -> impl IntoResponse {
    match set_authenticity_internal(&state, &request).await {
        Ok(response) => (
            StatusCode::OK,
            AxumJson(response),
        ).into_response(),
        Err(e) => {
            eprintln!("Error setting authenticity address {}: {:?}", request.authenticity_address, e);
            let (status, message) = match e.to_string().as_str() {
                s if s.contains("Invalid authenticity address") => (StatusCode::BAD_REQUEST, e.to_string()),
                s if s.contains("ONLY_OWNER") => (StatusCode::FORBIDDEN, "Caller is not the contract owner".to_string()),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal server error: {}", e),
                ),
            };
            (status, AxumJson(json!({"error": message}))).into_response()
        }
    }
}

async fn set_authenticity_internal(
    state: &Arc<AppState>,
    request: &SetAuthenticityRequest,
) -> eyre::Result<SetAuthenticityResponse> {
    // Validate authenticity address
    let authenticity_address: Address = request
        .authenticity_address
        .parse()
        .map_err(|_| eyre::eyre!("Invalid authenticity address"))?;

    // Get the contract and wallet details
    let contract = &state.ownership_contract;
    let wallet_address = contract.client().address();

    // Check wallet balance
    let balance = contract
        .client()
        .get_balance(wallet_address, None)
        .await
        .map_err(|e| eyre::eyre!("Failed to check wallet balance: {}", e))?;

    // Estimate gas with a 20% buffer
    let gas_estimate = contract
        .set_authenticity(authenticity_address)
        .estimate_gas()
        .await
        .map_err(|e| eyre::eyre!("Gas estimation failed: {}", e))?;
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
        .set_authenticity(authenticity_address)
        .gas(gas_limit)
        .gas_price(gas_price);

    let pending_tx = call
        .send()
        .await
        .map_err(|e| {
            // Attempt to parse revert reason
            let revert_reason = e.to_string();
            eyre::eyre!("Failed to send transaction: {}", revert_reason)
        })?;

    // Await transaction confirmation
    let receipt = pending_tx
        .await
        .map_err(|e| eyre::eyre!("Failed to confirm transaction: {}", e))?
        .ok_or_else(|| eyre::eyre!("Transaction receipt not found"))?;

    Ok(SetAuthenticityResponse {
        transaction_hash: format!("0x{}", hex::encode(receipt.transaction_hash)),
    })
}
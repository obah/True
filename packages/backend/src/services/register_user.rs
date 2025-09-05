
use axum::{
    extract::{ State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use ethers::{
    prelude::*,
    providers::{Http, Provider},
    types::{Address, U256},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use utoipa::ToSchema;
use crate::ownership::ownership_abi::TrueOwnership;
use crate::config::app_state::AppState;

// Define the input struct for the endpoint
#[derive(Deserialize, ToSchema)]
pub struct UserRegisterRequest {
    #[schema(example = "alice")]
    pub username: String,
}

// Define the response struct for successful registration
#[derive(Serialize, ToSchema)]
pub struct UserRegisterResponse {
    transaction_hash: String,
}

// Define the error response struct
#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[utoipa::path(
    post,
    path = "/api/user/register",
    request_body = UserRegisterRequest,
    responses(
        (status = 200, description = "User registered successfully", body = UserRegisterResponse, example = json!({
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        })),
        (status = 400, description = "Invalid input (e.g., invalid username)", body = ErrorResponse, example = json!({"error": "Username cannot be empty"})),
        (status = 500, description = "Internal server error (e.g., contract interaction failed)", body = ErrorResponse, example = json!({"error": "Internal server error: Failed to send transaction"}))
    ),
    tag = "Users"
)]
pub async fn user_register(
    State(state): State<Arc<AppState>>,
    Json(request): Json<UserRegisterRequest>,
) -> impl IntoResponse {
    match register_user_internal(&state, &request).await {
        Ok(response) => (
            StatusCode::OK,
            Json(response),
        ).into_response(),
        Err(e) => {
            eprintln!("Error registering user with username {}: {:?}", request.username, e);
            let (status, message) = match e.to_string().as_str() {
                s if s.contains("Username cannot be empty") => (StatusCode::BAD_REQUEST, e.to_string()),
                s if s.contains("Username too long") => (StatusCode::BAD_REQUEST, e.to_string()),
                s if s.contains("ADDRESS_ZERO") => (StatusCode::BAD_REQUEST, "Caller address cannot be zero".to_string()),
                s if s.contains("AUTHENTICITY_NOT_SET") => (StatusCode::INTERNAL_SERVER_ERROR, "Authenticity contract not set".to_string()),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal server error: {}", e),
                ),
            };
            (status, Json(json!({"error": message}))).into_response()
        }
    }
}

async fn register_user_internal(
    state: &Arc<AppState>,
    request: &UserRegisterRequest,
) -> eyre::Result<UserRegisterResponse> {
    // Validate username
    if request.username.is_empty() {
        return Err(eyre::eyre!("Username cannot be empty"));
    }
    if request.username.len() > 32 {
        return Err(eyre::eyre!("Username too long (max 32 characters)"));
    }

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
        .user_registers(request.username.clone())
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
        .user_registers(request.username.clone())
        .gas(gas_limit)
        .gas_price(gas_price);

    let pending_tx = call
        .send()
        .await
        .map_err(|e| {
            // Attempt to parse revert reason
            eyre::eyre!("Failed to send transaction: {}", e.to_string())
        })?;

    // Await transaction confirmation
    let receipt = pending_tx
        .await
        .map_err(|e| eyre::eyre!("Failed to confirm transaction: {}", e))?
        .ok_or_else(|| eyre::eyre!("Transaction receipt not found"))?;

    Ok(UserRegisterResponse {
        transaction_hash: format!("0x{}", hex::encode(receipt.transaction_hash)),
    })
}
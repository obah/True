use crate::ownership::ownership_abi::TrueOwnership;
use crate::config::app_state::AppState;
use crate::ownership::ownership_abi;
use axum::{
    Json as AxumJson,
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use ethers::{
    prelude::*,
    types::{Address, U256},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use utoipa::ToSchema;



// Define the input struct for the endpoint
#[derive(Deserialize, ToSchema)]
pub struct CreateItemRequest {
    #[schema(example = "0x1234567890abcdef1234567890abcdef12345678")]
    pub caller: String,
    #[schema(example = "Widget")]
    pub name: String,
    #[schema(example = "item123")]
    pub unique_id: String,
    // #[schema(example = "SN123456")]
    // pub serial: String,
    // #[schema(example = 1693526400)]
    // pub date: u64,
    // #[schema(example = "0x1234567890abcdef1234567890abcdef12345678")]
    // pub owner: String,
    #[schema(example = json!(["color: blue", "size: medium"]))]
    pub metadata: Vec<String>,
    #[schema(example = "Acme Corp")]
    pub manufacturer_name: String,
}

// Define the response struct for successful transaction
#[derive(Serialize, ToSchema)]
pub struct CreateItemResponse {
    transaction_hash: String,
}

// Define the error response struct
#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[utoipa::path(
    post,
    path = "/api/item/create",
    request_body = CreateItemRequest,
    responses(
        (status = 200, description = "Item created successfully", body = CreateItemResponse, example = json!({
            "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        })),
        (status = 400, description = "Invalid input (e.g., empty fields or invalid addresses)", body = ErrorResponse, example = json!({"error": "Caller address is invalid"})),
        (status = 403, description = "Unauthorized (e.g., caller not allowed to create item)", body = ErrorResponse, example = json!({"error": "Caller not authorized to create item"})),
        (status = 500, description = "Internal server error (e.g., contract interaction failed)", body = ErrorResponse, example = json!({"error": "Internal server error: Failed to send transaction"}))
    ),
    tag = "Items"
)]
pub async fn create_item(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateItemRequest>,
) -> impl IntoResponse {
    match create_item_internal(&state, &request).await {
        Ok(response) => (StatusCode::OK, AxumJson(response)).into_response(),
        Err(e) => {
            eprintln!(
                "Error creating item with unique_id {}: {:?}",
                request.unique_id, e
            );
            let (status, message) = match e.to_string().as_str() {
                s if s.contains("Caller address is invalid") => {
                    (StatusCode::BAD_REQUEST, e.to_string())
                }
                s if s.contains("Owner address is invalid") => {
                    (StatusCode::BAD_REQUEST, e.to_string())
                }
                s if s.contains("Metadata hash is invalid") => {
                    (StatusCode::BAD_REQUEST, e.to_string())
                }
                s if s.contains("Field cannot be empty") => {
                    (StatusCode::BAD_REQUEST, e.to_string())
                }
                s if s.contains("Date cannot be zero") => (StatusCode::BAD_REQUEST, e.to_string()),
                s if s.contains("ADDRESS_ZERO") => (
                    StatusCode::BAD_REQUEST,
                    "Caller or owner address cannot be zero".to_string(),
                ),
                s if s.contains("AUTHENTICITY_NOT_SET") => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Authenticity contract not set".to_string(),
                ),
                s if s.contains("UNAUTHORIZED") => (
                    StatusCode::FORBIDDEN,
                    "Caller not authorized to create item".to_string(),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal server error: {}", e),
                ),
            };
            (status, AxumJson(json!({"error": message}))).into_response()
        }
    }
}

async fn create_item_internal(
    state: &Arc<AppState>,
    request: &CreateItemRequest,
) -> eyre::Result<CreateItemResponse> {
    // Validate inputs
    if request.caller.is_empty() {
        return Err(eyre::eyre!("Caller address cannot be empty"));
    }
    if request.name.is_empty() {
        return Err(eyre::eyre!("Certificate name cannot be empty"));
    }
    if request.unique_id.is_empty() {
        return Err(eyre::eyre!("Certificate unique ID cannot be empty"));
    }
    // if request.serial.is_empty() {
    //     return Err(eyre::eyre!("Certificate serial cannot be empty"));
    // }
    // if request.date == 0 {
    //     return Err(eyre::eyre!("Certificate date cannot be zero"));
    // }
    // if request.owner.is_empty() {
    //     return Err(eyre::eyre!("Certificate owner address cannot be empty"));
    // }
    // if request.metadata_hash.is_empty() {
    //     return Err(eyre::eyre!("Certificate metadata hash cannot be empty"));
    // }
    if request.manufacturer_name.is_empty() {
        return Err(eyre::eyre!("Manufacturer name cannot be empty"));
    }

    // Parse addresses and metadata hash
    let caller: Address = request
        .caller
        .parse()
        .map_err(|_| eyre::eyre!("Caller address is invalid"))?;
    let owner: Address = "0xF2E7E2f51D7C9eEa9B0313C2eCa12f8e43bd1855"
        .parse()
        .map_err(|_| eyre::eyre!("Owner address is invalid"))?;
    let metadata_hash: [u8; 32] =
        hex::decode("0x5eaad066aaff0c6b04b35e501adc0a632a28e458c08f02309b866ead0e19f48f".trim_start_matches("0x"))
            .map_err(|_| eyre::eyre!("Metadata hash is invalid"))?
            .try_into()
            .map_err(|_| eyre::eyre!("Metadata hash must be 32 bytes"))?;

    // Get the contract and wallet details
    let contract = &state.ownership_contract;
    let wallet_address = contract.client().address();

    // Check wallet balance
    let balance = contract
        .client()
        .get_balance(wallet_address, None)
        .await
        .map_err(|e| eyre::eyre!("Failed to check wallet balance: {}", e))?;

    // Create the certificate struct for the contract call
    let certificate = ownership_abi::Certificate {
        name: request.name.clone(),
        unique_id: request.unique_id.clone(),
        serial: "12345".to_string(),
        date: U256::from(123456789),
        owner,
        metadata_hash,
        metadata: request.metadata.clone(),
    };

    // Estimate gas with a 20% buffer
    // let gas_estimate = contract
    //     .create_item(
    //         caller,
    //         certificate.clone(),
    //         request.manufacturer_name.clone(),
    //     )
    //     .estimate_gas()
    //     .await
    //     .map_err(|e| {
    //         let revert_reason = e.decode_revert().unwrap_or_else(|| e.to_string());
    //         eyre::eyre!("Gas estimation failed: {}", revert_reason)
    //     })?;
    // let gas_limit = gas_estimate * 120 / 100;

    // Get current gas price with a fallback
    let gas_price = contract
        .client()
        .get_gas_price()
        .await
        .unwrap_or(U256::from(2_000_000_000u64));

    // Check if sufficient funds are available
    // let required_funds = gas_limit * gas_price;
    // if balance < required_funds {
    //     return Err(eyre::eyre!(
    //         "Insufficient funds: have {} wei, need {} wei",
    //         balance,
    //         required_funds
    //     ));
    // }

    // Prepare and send the transaction
    let call = contract
        .create_item(caller, certificate, request.manufacturer_name.clone())
        // .gas(gas_limit)
        .gas_price(gas_price);

    let pending_tx = call.send().await.map_err(|e| {
        // Attempt to parse revert reason
        let revert_reason = e.decode_revert().unwrap_or_else(|| e.to_string());
        eyre::eyre!("Failed to send transaction: {}", revert_reason)
    })?;

    // Await transaction confirmation
    let receipt = pending_tx
        .await
        .map_err(|e| eyre::eyre!("Failed to confirm transaction: {}", e))?
        .ok_or_else(|| eyre::eyre!("Transaction receipt not found"))?;

    Ok(CreateItemResponse {
        transaction_hash: format!("0x{}", hex::encode(receipt.transaction_hash)),
    })
}

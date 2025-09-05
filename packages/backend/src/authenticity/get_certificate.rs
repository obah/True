use crate::config::app_state::AppState;
use crate::contract_models::Item;
use crate::schema::{items, manufacturers};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json as AxumJson,
};
use diesel::associations::HasTable;
use diesel::prelude::*;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use utoipa::ToSchema;
// use crate::schema::manufacturers::dsl::manufacturers;
// use crate::schema::users_info::username;

// Define the error response struct
#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}
// const cert = {
//     name: certificate.name,
//     uniqueId: certificate.uniqueId,
//     serial: certificate.serial,
//     date: timestamp,
//     owner: account,
//     metadataHash: ethers.keccak256(ethers.AbiCoder.defaultAbiCoder().encode(['string[]'], [metadata])),
//     metadata
// };
#[derive(Serialize, ToSchema)]
pub struct CertificateResponse {
    name: String,
    unique_id: String,
    serial: String,
    date: i64,
    owner: String,
    metadata: Vec<String>,
}

#[utoipa::path(
    get,
    path = "/api/certificate/get",
    params(
        ("item_id" = String, Path, description = "The unique ID of the item", example = "item123")
    ),
    responses(
        (status = 200, description = "Item retrieved successfully", body = CertificateResponse, example = json!({
            "name": "Widget",
            "unique_id": "item123",
            "serial": "SN123456",
            "date": 1693526400,
            "owner": "0x1234567890abcdef1234567890abcdef12345678",
            "metadata": ["color: blue", "size: medium"],
        })),
        (status = 404, description = "Item not found", body = ErrorResponse, example = json!({"error": "Item not found"})),
        (status = 500, description = "Internal server error (e.g., database failure)", body = ErrorResponse, example = json!({"error": "Internal server error: Failed to query database"}))
    ),
    tag = "Items"
)]
pub async fn fetch_certificate (
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<String>,
) -> impl IntoResponse {
    match get_certificate_internal(&state, &item_id).await {
        Ok(response) => (StatusCode::OK, AxumJson(response)).into_response(),
        Err(e) => {
            eprintln!("Error fetching item {}: {:?}", item_id, e);
            let (status, message) = match e.to_string().as_str() {
                s if s.contains("Item not found") => (StatusCode::NOT_FOUND, e.to_string()),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal server error: {}", e),
                ),
            };
            (status, AxumJson(json!({"error": message}))).into_response()
        }
    }
}

async fn get_certificate_internal(
    state: &Arc<AppState>,
    item_id: &str,
) -> eyre::Result<CertificateResponse> {
    // Validate item_id
    if item_id.is_empty() {
        return Err(eyre::eyre!("Item ID cannot be empty"));
    }

    // Get a database connection
    let conn = &mut state
        .db_pool
        .get()
        .map_err(|e| eyre::eyre!("Failed to get database connection: {}", e))?;

    // Query the items table
    let item = items::table
        .filter(items::item_id.eq(item_id))
        .select(Item::as_select())
        .first(conn)
        .optional()
        .map_err(|e| eyre::eyre!("Failed to query database: {}", e))?
        .ok_or_else(|| eyre::eyre!("Item not found"))?;

    let manufacturer_address = manufacturers::table
        .filter(manufacturers::manufacturer_name.eq(item.manufacturer))
        .select(manufacturers::manufacturer_address)
        .first::<String>(conn)
        .optional();

    let certificate = CertificateResponse {
        name: item.name,
        unique_id: item.item_id,
        serial: item.serial.clone(),
        date: item.date,
        owner: manufacturer_address?.unwrap(),
        metadata: item
            .metadata
            .iter()
            .filter_map(|opt| {
                opt.as_ref()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
            })
            .collect(),
    };

    Ok(certificate)
}

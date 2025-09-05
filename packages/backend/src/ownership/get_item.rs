use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json as AxumJson,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use utoipa::ToSchema;
use crate::config::app_state::AppState;
use crate::contract_models::Item;
use crate::schema::items;


// Define the error response struct
#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[utoipa::path(
    get,
    path = "/api/item/{item_id}",
    params(
        ("item_id" = String, Path, description = "The unique ID of the item", example = "item123")
    ),
    responses(
        (status = 200, description = "Item retrieved successfully", body = Item, example = json!({
            "id": 1,
            "item_id": "item123",
            "name": "Widget",
            "serial": "SN123456",
            "date": 1693526400,
            "owner": "0x1234567890abcdef1234567890abcdef12345678",
            "manufacturer": "Acme Corp",
            "metadata": ["color: blue", "size: medium"],
            "created_at": "2023-09-01T00:00:00Z",
            "tnx_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        })),
        (status = 404, description = "Item not found", body = ErrorResponse, example = json!({"error": "Item not found"})),
        (status = 500, description = "Internal server error (e.g., database failure)", body = ErrorResponse, example = json!({"error": "Internal server error: Failed to query database"}))
    ),
    tag = "Items"
)]
pub async fn get_item(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<String>,
) -> impl IntoResponse {
    match get_item_internal(&state, &item_id).await {
        Ok(response) => (
            StatusCode::OK,
            AxumJson(response),
        ).into_response(),
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

async fn get_item_internal(state: &Arc<AppState>, item_id: &str) -> eyre::Result<Item> {
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

    Ok(item)
}
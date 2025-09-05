use axum::{extract::{Query, State}, http::StatusCode, response::IntoResponse, Json};
use diesel::prelude::*;
use eyre::Result;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::config::app_state::AppState;
use crate::contract_models::{Item};
use crate::schema::items;

#[derive(Serialize, ToSchema)]
pub struct ItemsResponse {
    items: Vec<Item>,
}

#[derive(Deserialize, ToSchema)]
pub struct ItemQuery {
    #[schema(example = "0x1234567890abcdef1234567890abcdef12345678")]
    pub owner: String,
}

#[utoipa::path(
    get,
    path = "/api/items/owner",
    params(
        ("owner" = String, Query, description = "Owner's blockchain address", example = "0x1234567890abcdef1234567890abcdef12345678")
    ),
    responses(
        (status = 200, description = "Items found for the owner", body = ItemsResponse, example = json!({
            "items": [
                {
                    "id": 1,
                    "item_id": "item_001",
                    "name": "Luxury Watch",
                    "serial": "W12345",
                    "date": 1625097600,
                    "owner": "0x1234567890abcdef1234567890abcdef12345678",
                    "manufacturer": "SAMSUNG",
                    "metadata": ["color: gold", null],
                    "created_at": "2025-08-25T19:47:00Z",
                    "tnx_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                }
            ]
        })),
        (status = 400, description = "Owner address not provided"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Items"
)]
pub async fn get_owner_items(
    Query(query): Query<ItemQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match get_items_by_owner_internal(&state, &query).await {
        Ok(items) => (
            StatusCode::OK,
            Json(ItemsResponse { items }),
        ).into_response(),
        Err(e) => {
            eprintln!("Error fetching items for owner {}: {:?}", query.owner, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Internal server error: {}", e)})),
            ).into_response()
        }
    }
}

async fn get_items_by_owner_internal(
    state: &Arc<AppState>,
    query: &ItemQuery,
) -> Result<Vec<Item>> {
    if query.owner.is_empty() {
        return Err(eyre::eyre!("Owner address must be provided"));
    }

    let conn = &mut state.db_pool.get().map_err(|e| {
        eprintln!("Failed to get DB connection: {:?}", e);
        eyre::eyre!("Failed to get DB connection: {}", e)
    })?;

    let items = items::table
        .select(Item::as_select())
        .filter(items::owner.eq(query.owner.clone()))
        .load::<Item>(conn)
        .map_err(|e| {
            eprintln!("Failed to fetch items for owner {}: {:?}", query.owner, e);
            eyre::eyre!("Failed to fetch items: {}", e)
        })?;

    Ok(items)
}
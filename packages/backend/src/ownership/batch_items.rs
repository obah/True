use axum::{
    extract::{Json, State},
    http::StatusCode
    ,
};
use diesel::{prelude::*, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};

// Assuming AppState contains the Diesel connection pool
use crate::config::app_state::AppState;
use crate::schema::items;

// Request payload
#[derive(Deserialize, ToSchema)]
pub struct BatchItemsPayload {
    item_ids: Vec<String>,
}

// Response structs
#[derive(Serialize, ToSchema, Queryable, Selectable)]
#[diesel(table_name = items)]
pub struct ItemResponse {
    pub item_id: String,
    pub name: String,
    pub serial: String,
    pub date: i64,
    pub owner: String,
    pub manufacturer: String,
    pub metadata: Vec<Option<String>>,
    pub created_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct BatchItemsResponse {
    pub items: Vec<ItemResponse>,
}

// Axum handler for /api/items/batch
#[utoipa::path(
    post,
    path = "/api/items/batch",
    request_body = BatchItemsPayload,
    responses(
        (status = 200, description = "Successfully retrieved item details", body = BatchItemsResponse),
        (status = 400, description = "Invalid or empty item_ids list"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Items"
)]
pub async fn batch_items(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<BatchItemsPayload>,
) -> Result<Json<BatchItemsResponse>, StatusCode> {
    use crate::schema::items::dsl::*;

    // Validate input
    if payload.item_ids.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let conn = &mut state.db_pool.get().map_err(|e| {
        eprintln!("Failed to get DB connection: {:?}", e);
        eyre::eyre!("Failed to get DB connection: {}", e)
    }).unwrap();

    // Query items by item_id
    let item_list: Vec<ItemResponse> = items
        .filter(item_id.eq_any(&payload.item_ids))
        .select(ItemResponse::as_select())
        .load(conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(BatchItemsResponse { items: item_list }))
}

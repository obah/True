use crate::config::app_state::AppState;
use crate::contract_models::{Manufacturer, User};
// Assuming AppState contains the Diesel connection pool
use crate::schema::manufacturers;
use axum::{extract::{Json, State}, http::StatusCode};
use diesel::{prelude::*, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};

// Request payload
#[derive(Deserialize, ToSchema)]
pub struct SyncPayload {
    address: String,
}
#[derive(Serialize, ToSchema, Queryable, Selectable)]
#[diesel(table_name = manufacturers)]
pub struct ManufacturerResponse {
    pub manufacturer_address: String,
    pub manufacturer_name: String,
    pub is_registered: bool,
    pub registered_at: String,
    pub tnx_hash: String,
}

#[derive(Serialize, ToSchema)]
pub struct SyncResponse {
    pub user: Option<User>,
    pub manufacturer: Option<Manufacturer>,
}

#[utoipa::path(
    post,
    path = "/api/sync",
    request_body = SyncPayload,
    responses(
        (status = 200, description = "Successfully synced user and manufacturer data", body = SyncResponse),
        (status = 400, description = "Invalid address format"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Sync"
)]
pub async fn sync(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SyncPayload>,
) -> Result<Json<SyncResponse>, StatusCode> {
    use crate::schema::users_info::dsl::*;
    use crate::schema::manufacturers::dsl::*;

    // Validate address format (basic check for Ethereum address)
    if !payload.address.starts_with("0x") || payload.address.len() != 42 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let conn = &mut state.db_pool.get().map_err(|e| {
        eprintln!("Failed to get DB connection: {:?}", e);
        eyre::eyre!("Failed to get DB connection: {}", e)
    }).unwrap();
    // Query user
    let user: Option<User> = users_info
        .filter(user_address.eq(&payload.address))
        .select(User::as_select())
        .first(conn)
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Query manufacturer
    let manufacturer: Option<Manufacturer> = manufacturers
        .filter(manufacturer_address.eq(&payload.address))
        .select(Manufacturer::as_select())
        .first(conn)
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SyncResponse { user, manufacturer }))
}
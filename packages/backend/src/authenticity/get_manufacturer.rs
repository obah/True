use crate::config::app_state::AppState;
use crate::contract_models::{Manufacturer, ManufacturerQuery};
use crate::schema::manufacturers;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use diesel::RunQueryDsl;
use diesel::prelude::*;
use eyre::Result;
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};

#[utoipa::path(
    get,
    path = "/api/manufacturer",
    params(
        (
            "address" = Option<String>, 
            Query, description = "Manufacturer's blockchain address", 
            example = "0x1234567890abcdef1234567890abcdef12345678"
        ),
        (
            "username" = Option<String>, 
            Query, description = "Manufacturer's username", 
            example = "SAMSUNG"
        )
    ),
    responses(
        (
            status = 200, description = "Manufacturer found", 
            body = Manufacturer, 
                example = json!({
                "manufacturer_address": "0x1234567890abcdef1234567890abcdef12345678",
                "manufacturer_name": "SAMSUNG",
                "is_registered": true,
                "registered_at": "2025-08-24T12:04:00Z",
            })
        ),
        (status = 400, description = "Neither address nor username provided"),
        (status = 404, description = "Manufacturer not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Manufacturers"
)]
pub async fn get_manufacturer(
    Query(query): Query<ManufacturerQuery>,
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    match get_manufacturer_internal(&state, &query).await {
        Ok(Some(fetched_manufacturer)) => {
            (StatusCode::OK, Json(fetched_manufacturer)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Manufacturer not found"})),
        )
            .into_response(),
        Err(e) => {
            eprintln!("Error fetching manufacturer: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Internal server error: {}", e)})),
            )
                .into_response()
        }
    }
}

async fn get_manufacturer_internal(
    state: &Arc<AppState>,
    query: &ManufacturerQuery,
) -> Result<Option<Manufacturer>> {
    let mut conn = state.db_pool.get().map_err(|e| {
        eprintln!("Failed to get DB connection: {:?}", e);
        eyre::eyre!("Failed to get DB connection: {}", e)
    })?;

    if query.address.is_none() && query.username.is_none() {
        return Err(eyre::eyre!("Either address or username must be provided"));
    }

    let mut query_builder = manufacturers::table
        .select(Manufacturer::as_select())
        .into_boxed();

    if let Some(ref address) = query.address {
        query_builder = query_builder.filter(manufacturers::manufacturer_address.eq(address));
    }

    if let Some(ref username) = query.username {
        query_builder = query_builder.or_filter(manufacturers::manufacturer_name.eq(username));
    }

    let fetched_manufacturer = query_builder
        .first::<Manufacturer>(&mut conn)
        .optional()
        .map_err(|e| {
            eprintln!(
                "Failed to fetch manufacturer with address {:?} or username {:?}: {:?}",
                query.address, query.username, e
            );
            eyre::eyre!("Failed to fetch manufacturer: {}", e)
        })?;

    Ok(fetched_manufacturer)
}
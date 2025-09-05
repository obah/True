// use crate::authenticity::get_certificate::CertificateResponse;
use crate::config::app_state::AppState;
use crate::schema::{certificates, manufacturers};
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use diesel::prelude::*;
use diesel::{PgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};

#[derive(Queryable, Insertable, Selectable, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::certificates)]
pub struct Certificates {
    pub unique_id: String,
    pub name: String,
    pub serial: String,
    pub date: i64,
    pub owner: String,
    pub metadata_hash: String,
    pub metadata: Vec<Option<String>>,
    pub signature: String
}
// Struct for GET query
#[derive(Deserialize, Serialize, ToSchema)]
pub struct CertificateDTO {
    pub unique_id: String,
}

// Error response schema for Swagger
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

// Axum handler to save certificate and signature
#[utoipa::path(
    post,
    path = "/api/certificate/create",
    request_body = Certificates,
    responses(
        (status = 200, description = "Certificate and signature saved successfully", body = Certificates, example = json!({
            "name": "iPhone 15 Pro",
            "unique_id": "123",
            "serial": "SN123456",
            "date": 1695868800,
            "owner": "0x1234567890abcdef1234567890abcdef12345678",
            "metadata_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "metadata": ["Black", "128GB", "Pro Model"]
        })),
        (status = 400, description = "Invalid input (e.g., empty unique_id or invalid owner)", body = ErrorResponse, example = json!({"error": "Unique ID cannot be empty"})),
        (status = 404, description = "Manufacturer not found", body = ErrorResponse, example = json!({"error": "Manufacturer not found"})),
        (status = 500, description = "Internal server error", body = ErrorResponse, example = json!({"error": "Failed to save certificate: Database error"}))
    ),
    tag = "Certificates"
)]
pub async fn save_certificate(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Certificates>,
) -> axum::response::Result<Json<CertificateDTO>> {

    let conn = &mut state.db_pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to get DB connection: {}", e) })),
        )
    })?;

    // Validate certificate
    if payload.unique_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Unique ID cannot be empty" })),
        )
            .into());
    }

    // Verify manufacturer exists
    let manufacturer_address = manufacturers::table
        .filter(manufacturers::manufacturer_address.eq(&payload.owner))
        .select(manufacturers::manufacturer_address)
        .first::<String>(conn)
        .optional()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Database error: {}", e) })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Manufacturer not found" })),
            )
        })?;

    // Ensure owner matches manufacturer_address
    if manufacturer_address.to_lowercase() != payload.owner.to_lowercase() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Owner address does not match registered manufacturer" })),
        ).into());
    }

    // Prepare certificate for insertion
    // let certificate = Certificates {
    //     unique_id: payload.unique_id.clone(),
    //     name: payload.name,
    //     serial: payload.serial,
    //     date: payload.date,
    //     owner: payload.owner,
    //     metadata_hash: payload.metadata_hash,
    //     metadata: payload.metadata,
    //     signature: payload.signature,
    // };

    // Insert and retrieve the saved certificate
    diesel::insert_into(certificates::table)
        .values(&payload)
        .execute(conn)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to save certificate: {}", e) })),
            )
        })?;

    let response = CertificateDTO {
        unique_id: payload.unique_id,
    };

    Ok(Json(response))
}

// Axum handler to fetch certificate by unique_id
#[utoipa::path(
    get,
    path = "/api/certificate/{item_id}",
    params(
        ("unique_id" = String, Query, description = "Unique ID of the certificate", example = "123")
    ),
    responses(
        (status = 200, description = "Certificate and signature retrieved successfully", body = (Certificates, String), example = json!([
            {
                "name": "iPhone 15 Pro",
                "unique_id": "123",
                "serial": "SN123456",
                "date": 1695868800,
                "owner": "0x1234567890abcdef1234567890abcdef12345678",
                "metadata_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                "metadata": ["Black", "128GB", "Pro Model"]
            },
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
        ])),
        (status = 400, description = "Invalid input (e.g., empty unique_id)", body = ErrorResponse, example = json!({"error": "Unique ID cannot be empty"})),
        (status = 404, description = "Certificate not found", body = ErrorResponse, example = json!({"error": "Certificate not found"})),
        (status = 500, description = "Internal server error", body = ErrorResponse, example = json!({"error": "Database error"}))
    ),
    tag = "Certificates"
)]
pub async fn get_certificate(
    Query(query): Query<CertificateDTO>,
    State(state): State<Arc<AppState>>,
) -> axum::response::Result<Json<Certificates>> {
    let conn = &mut state.db_pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to get DB connection: {}", e) })),
        )
    })?;

    if query.unique_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Unique ID cannot be empty" })),
        )
            .into());
    }

    let cert = certificates::table
        .filter(certificates::unique_id.eq(query.unique_id))
        .select(Certificates::as_select())
        .first::<Certificates>(conn)
        .optional()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Database error: {}", e) })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Certificate not found" })),
            )
        })?;

    Ok(Json(cert))
}

use diesel::{AsChangeset, Insertable, Queryable};
use ethabi::ethereum_types::{Address, H256, U256, H160};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::schema::manufacturers;
use chrono::DateTime;
use diesel::prelude::*;
use chrono::Utc;


#[derive(Queryable, Insertable, AsChangeset, Serialize, ToSchema)]
#[diesel(table_name = crate::schema::manufacturers)]
pub struct NewManufacturer {
    pub manufacturer_address: String,
    pub manufacturer_name: String,
    pub tnx_hash: String
}

#[derive(Queryable, serde::Serialize)]
pub struct Manu {
    pub id: i32,
    pub manufacturer_address: String,
    pub manufacturer_name: String,
    pub timestamp: Option<DateTime<chrono::Utc>>, // Nullable timestamp
    pub tnx_hash: Option<String>, // Assuming NOT NULL; use Option<String> if nullable

}



// #[derive(Queryable, Selectable, Serialize)]
// #[diesel(check_for_backend(diesel::pg::Pg))]
// pub struct Manu {
//     pub id: i32,
//     pub manufacturer_address: String,
//     pub manufacturer_name: String,
//     pub timestamp: Option<DateTime<Utc>>,
//     pub tnx_hash: String,
// }
#[derive(Queryable, Insertable, AsChangeset, Serialize, ToSchema)]
#[diesel(table_name = crate::schema::contracts)]
pub struct NewContractCreated {
    pub contract_address: String,
    pub owner: String,
}


#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Certificate {
    name: String,
    unique_id: String,
    serial: String,
    date: U256,
    owner: Address,
    metadata_hash: H256,
}

// Request bodies
#[derive(serde::Deserialize)]
struct RegisterManufacturerRequest {
    name: String,
}

#[derive(serde::Deserialize)]
struct GetManufacturerByNameRequest {
    manufacturer_name: String,
}

#[derive(serde::Deserialize)]
struct GetManufacturerRequest {
    user_address: String,
}

#[derive(serde::Deserialize)]
struct GetManufacturerAddressRequest {
    expected_manufacturer: String,
}

#[derive(serde::Deserialize)]
struct VerifySignatureRequest {
    certificate: Certificate,
    signature: String, // hex string
}

#[derive(serde::Deserialize)]
struct HashTypedDataV4Request {
    struct_hash: String, // hex string
}

#[derive(serde::Deserialize)]
struct UserClaimOwnershipRequest {
    certificate: Certificate,
    signature: String, // hex string
}

#[derive(serde::Deserialize)]
struct VerifyAuthenticityRequest {
    certificate: Certificate,
    signature: String, // hex string
}
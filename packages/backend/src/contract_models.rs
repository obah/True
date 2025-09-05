use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::contracts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Contract {
    pub contract_address: String,
    pub owner: String,
    pub tnx_hash: String,
    pub created_at: String,
}
#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::contracts)]
pub struct NewContract {
    pub contract_address: String,
    pub owner: String,
    pub tnx_hash: String,
    pub created_at: String,
}

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::users_info)]
pub struct UserInfo {
    pub user_address: String,
    pub username: String,
    pub is_registered: bool,
    pub created_at: String,
    pub tnx_hash: String,
}
#[derive(Serialize, Deserialize, ToSchema, Queryable, Selectable)]
#[diesel(table_name = crate::schema::users_info)]
pub struct User {
    pub user_address: String,
    pub username: String,
    pub is_registered: bool,
    pub created_at: String,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, ToSchema)]
#[diesel(table_name = crate::schema::manufacturers)]
#[serde(rename_all = "camelCase")]
pub struct Manufacturer {
    #[schema(example = "0x1234567890abcdef1234567890abcdef12345678")]
    manufacturer_address: String,
    #[schema(example = "SAMSUNG")]
    manufacturer_name: String,
    #[schema(example = true)]
    is_registered: bool,
    #[schema(example = "2025-08-24T12:04:00Z")]
    registered_at: String,
}


#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::manufacturers)]
pub struct NewManufacturer {
    pub manufacturer_address: String,
    pub manufacturer_name: String,
    pub is_registered: bool,
    pub registered_at: String,
    pub tnx_hash: String
}

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::ownership_codes)]
pub struct OwnershipCode {
    pub ownership_code: String,
    pub item_id: String,
    pub item_owner: String,
    pub temp_owner: String,
    pub created_at: String,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::items)]
pub struct Item {
    pub item_id: String,
    pub name: String,
    pub serial: String,
    pub date: i64,
    pub owner: String,
    pub manufacturer: String,
    #[schema(nullable = true, value_type = Vec<Option<String>>)]
    pub metadata: Vec<Option<String>>,
    pub created_at: String,
}


#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::items)]
pub struct NewItem {
    pub item_id: String,
    pub name: String,
    pub serial: String,
    pub date: i64,
    pub owner: String,
    pub manufacturer: String,
    pub metadata: Vec<String>,
    pub created_at: String,
    pub tnx_hash: String,
}

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ownership_claims)]
pub struct OwnershipClaim {
    pub id: i32,
    pub item_id: String,
    pub new_owner: String,
    pub old_owner: String,
    pub tnx_hash: String,
    pub created_at: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ownership_claims)]
pub struct NewOwnershipClaim {
    pub item_id: String,
    pub new_owner: String,
    pub old_owner: String,
    pub tnx_hash: String,
    pub created_at: String,
}

#[derive(Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::code_revokations)]
pub struct CodeRevokation {
    pub id: i32,
    pub item_hash: String,
    pub tnx_hash: String,
    pub created_at: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::code_revokations)]
pub struct NewCodeRevokation {
    pub item_hash: String,
    pub tnx_hash: String,
    pub created_at: String,
}

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::authenticity_settings)]
pub struct AuthenticitySetting {
    pub id: i32,
    pub authenticity_address: String,
    pub tnx_hash: String,
    pub created_at: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::authenticity_settings)]
pub struct NewAuthenticitySetting {
    pub authenticity_address: String,
    pub tnx_hash: String,
    pub created_at: String,
}


#[derive(Deserialize, ToSchema)]
pub struct ManufacturerQuery {
    #[schema(example = "0x1234567890abcdef1234567890abcdef12345678")]
    pub(crate) address: Option<String>,
    #[schema(example = "john_doe")]
    pub(crate) username: Option<String>,
}

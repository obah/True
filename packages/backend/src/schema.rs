// @generated automatically by Diesel CLI.

diesel::table! {
    authenticity_settings (id) {
        id -> Int4,
        authenticity_address -> Text,
        tnx_hash -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    certificates (unique_id) {
        unique_id -> Text,
        name -> Text,
        serial -> Text,
        date -> Int8,
        owner -> Text,
        metadata_hash -> Text,
        metadata -> Array<Nullable<Text>>,
        signature -> Text,
    }
}

diesel::table! {
    code_revokations (id) {
        id -> Int4,
        item_hash -> Text,
        tnx_hash -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    contracts (contract_address) {
        contract_address -> Text,
        owner -> Text,
        tnx_hash -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    items (id) {
        id -> Int4,
        item_id -> Text,
        name -> Text,
        serial -> Text,
        date -> Int8,
        owner -> Text,
        manufacturer -> Text,
        metadata -> Array<Nullable<Text>>,
        created_at -> Text,
        tnx_hash -> Text,
    }
}

diesel::table! {
    manufacturers (manufacturer_address) {
        manufacturer_address -> Text,
        manufacturer_name -> Text,
        is_registered -> Bool,
        registered_at -> Text,
        tnx_hash -> Text,
    }
}

diesel::table! {
    ownership_claims (id) {
        id -> Int4,
        item_id -> Text,
        old_owner -> Text,
        new_owner -> Text,
        tnx_hash -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    ownership_codes (ownership_code) {
        ownership_code -> Text,
        item_id -> Text,
        item_owner -> Text,
        temp_owner -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    users_info (user_address) {
        user_address -> Text,
        username -> Text,
        is_registered -> Bool,
        created_at -> Text,
        tnx_hash -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    authenticity_settings,
    certificates,
    code_revokations,
    contracts,
    items,
    manufacturers,
    ownership_claims,
    ownership_codes,
    users_info,
);

use crate::config::app_state::AppState;
use crate::contract_models::{
    NewAuthenticitySetting, NewContract, NewItem, NewOwnershipClaim, UserInfo,
};
use crate::ownership::ownership_abi::{
    AuthenticitySetFilter, ItemCreatedFilter, OwnershipCreatedFilter, OwnershipTransferredFilter,
    UserRegisteredFilter,
};
use crate::ownership::ownership_abi::{TrueOwnership, TrueOwnershipEvents};
use crate::schema::{
    authenticity_settings, contracts, items, ownership_claims, ownership_codes, users_info,
};
use chrono::Utc;
use diesel::prelude::*;
use diesel::{PgConnection, RunQueryDsl};
use ecdsa::SigningKey;
use ethers::core::k256::Secp256k1;
use ethers::core::utils::to_checksum;
use ethers::prelude::*;
use eyre::Result;
use std::sync::Arc;

pub async fn listen_for_ownership_events(state: &Arc<AppState>) -> Result<()> {
    let contract = state.ownership_contract.clone();
    let client = contract.client();

    // Fetch historical events from the last 1,000 blocks in chunks
    let latest_block = client.get_block_number().await.map_err(|e| {
        eprintln!("Failed to get latest block: {:?}", e.to_string());
        eyre::eyre!("Failed to get latest block: {}", e)
    })?;
    let from_block = latest_block.saturating_sub(U64::from(20));
    let chunk_size = U64::from(4);

    // Process historical events in chunks
    let mut current_block = from_block;
    while current_block < latest_block {
        let to_block = (current_block + chunk_size).min(latest_block);
        eprintln!(
            "Querying Ownership historical events from block {} to {} (range: {})",
            current_block,
            to_block,
            to_block - current_block + 1
        );

        // Event filters for the chunk
        let ownership_created_filter = contract
            .event::<OwnershipCreatedFilter>()
            .from_block(current_block)
            .to_block(to_block);
        let user_registered_filter = contract
            .event::<UserRegisteredFilter>()
            .from_block(current_block)
            .to_block(to_block);
        let item_created_filter = contract
            .event::<ItemCreatedFilter>()
            .from_block(current_block)
            .to_block(to_block);
        let ownership_transferred_filter = contract
            .event::<OwnershipTransferredFilter>()
            .from_block(current_block)
            .to_block(to_block);
        let authenticity_set_filter = contract
            .event::<AuthenticitySetFilter>()
            .from_block(current_block)
            .to_block(to_block);

        // Fetch historical events with metadata
        let ownership_created_logs =
            ownership_created_filter
                .query_with_meta()
                .await
                .map_err(|e| {
                    eprintln!(
                        "Failed to query OwnershipCreated events for blocks {} to {}: {:?}",
                        current_block,
                        to_block,
                        e.to_string()
                    );
                    eyre::eyre!("Failed to query OwnershipCreated events: {}", e)
                })?;
        let user_registered_logs = user_registered_filter
            .query_with_meta()
            .await
            .map_err(|e| {
                eprintln!(
                    "Failed to query UserRegistered events for blocks {} to {}: {:?}",
                    current_block,
                    to_block,
                    e.to_string()
                );
                eyre::eyre!("Failed to query UserRegistered events: {}", e)
            })?;
        let item_created_logs = item_created_filter.query_with_meta().await.map_err(|e| {
            eprintln!(
                "Failed to query ItemCreated events for blocks {} to {}: {:?}",
                current_block,
                to_block,
                e.to_string()
            );
            eyre::eyre!("Failed to query ItemCreated events: {}", e)
        })?;
        let ownership_transferred_logs = ownership_transferred_filter
            .query_with_meta()
            .await
            .map_err(|e| {
                eprintln!(
                    "Failed to query OwnershipTransferred events for blocks {} to {}: {:?}",
                    current_block,
                    to_block,
                    e.to_string()
                );
                eyre::eyre!("Failed to query OwnershipTransferred events: {}", e)
            })?;
        let authenticity_set_logs =
            authenticity_set_filter
                .query_with_meta()
                .await
                .map_err(|e| {
                    eprintln!(
                        "Failed to query AuthenticitySet events for blocks {} to {}: {:?}",
                        current_block,
                        to_block,
                        e.to_string()
                    );
                    eyre::eyre!("Failed to query AuthenticitySet events: {}", e)
                })?;

        // Process historical events
        let conn = &mut state.db_pool.get().map_err(|e| {
            eprintln!("Failed to get DB connection: {:?}", e);
            eyre::eyre!("Failed to get DB connection: {}", e)
        })?;
        for (event, meta) in ownership_created_logs {
            let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
            process_ownership_created_event(&event, conn, txn_hash)?;
        }
        for (event, meta) in user_registered_logs {
            let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
            process_user_registered_event(&event, conn, txn_hash, &contract).await?;
        }
        for (event, meta) in item_created_logs {
            let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
            process_item_created_event(&event, conn, txn_hash, &contract).await?;
        }
        for (event, meta) in ownership_transferred_logs {
            let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
            process_ownership_transferred_event(&event, conn, txn_hash)?;
        }
        for (event, meta) in authenticity_set_logs {
            let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
            process_authenticity_set_event(&event, conn, txn_hash)?;
        }

        current_block = to_block + 1;
    }

    // Stream future events
    eprintln!(
        "Starting Ownership event stream from block {}",
        latest_block + 1
    );
    let events = contract.events().from_block(latest_block + 1);
    let mut stream = events.stream_with_meta().await.map_err(|e| {
        eprintln!("Failed to create event stream: {:?}", e.to_string());
        eyre::eyre!("Failed to create event stream: {}", e)
    })?;

    loop {
        match stream.next().await {
            Some(Ok((TrueOwnershipEvents::OwnershipCreatedFilter(event), meta))) => {
                let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
                let conn = &mut state.db_pool.get().map_err(|e| {
                    eprintln!("Failed to get DB connection: {:?}", e);
                    eyre::eyre!("Failed to get DB connection: {}", e)
                })?;
                process_ownership_created_event(&event, conn, txn_hash)?;
            }
            Some(Ok((TrueOwnershipEvents::UserRegisteredFilter(event), meta))) => {
                let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
                let conn = &mut state.db_pool.get().map_err(|e| {
                    eprintln!("Failed to get DB connection: {:?}", e);
                    eyre::eyre!("Failed to get DB connection: {}", e)
                })?;
                process_user_registered_event(&event, conn, txn_hash, &contract).await?;
            }
            Some(Ok((TrueOwnershipEvents::ItemCreatedFilter(event), meta))) => {
                let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
                let conn = &mut state.db_pool.get().map_err(|e| {
                    eprintln!("Failed to get DB connection: {:?}", e);
                    eyre::eyre!("Failed to get DB connection: {}", e)
                })?;
                process_item_created_event(&event, conn, txn_hash, &contract).await?;
            }
            Some(Ok((TrueOwnershipEvents::OwnershipTransferredFilter(event), meta))) => {
                let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
                let conn = &mut state.db_pool.get().map_err(|e| {
                    eprintln!("Failed to get DB connection: {:?}", e);
                    eyre::eyre!("Failed to get DB connection: {}", e)
                })?;
                process_ownership_transferred_event(&event, conn, txn_hash)?;
            }
            Some(Ok((TrueOwnershipEvents::AuthenticitySetFilter(event), meta))) => {
                let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
                let conn = &mut state.db_pool.get().map_err(|e| {
                    eprintln!("Failed to get DB connection: {:?}", e);
                    eyre::eyre!("Failed to get DB connection: {}", e)
                })?;
                process_authenticity_set_event(&event, conn, txn_hash)?;
            }
            Some(Err(e)) => {
                eprintln!("Event stream error: {:?}", e.to_string());
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
            None => {
                eprintln!("Event stream ended unexpectedly");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        }
    }
}

fn process_ownership_created_event(
    event: &OwnershipCreatedFilter,
    conn: &mut PgConnection,
    txn_hash: Option<String>,
) -> Result<()> {
    let contract_address = to_checksum(&event.contract_address, None);
    let owner = to_checksum(&event.owner, None);

    // Check if contract exists
    let exists: bool = contracts::table
        .filter(contracts::contract_address.eq(&contract_address))
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .map(|count| count > 0)
        .map_err(|e| {
            eprintln!(
                "Failed to check existing contract {}: {:?}",
                contract_address, e
            );
            eyre::eyre!("Failed to check existing contract: {}", e)
        })?;

    if exists {
        eprintln!(
            "Skipping duplicate contract created for {} (tx: {:?})",
            contract_address, txn_hash
        );
        return Ok(());
    }

    // Insert the contract
    diesel::insert_into(contracts::table)
        .values(NewContract {
            contract_address,
            owner,
            tnx_hash: txn_hash.unwrap(),
            created_at: Utc::now().to_rfc3339(),
        })
        .execute(conn)
        .map_err(|e| {
            eprintln!("Failed to insert contract: {:?}", e);
            eyre::eyre!("Failed to insert contract: {}", e)
        })?;

    Ok(())
}

async fn process_user_registered_event(
    event: &UserRegisteredFilter,
    conn: &mut PgConnection,
    txn_hash: Option<String>,
    ownership_contract: &TrueOwnership<SignerMiddleware<Provider<Http>, Wallet<SigningKey<Secp256k1>>>>,
) -> Result<()> {
    let user_address = to_checksum(&event.user_address, None);

    // Check if user exists
    let exists: bool = users_info::table
        .filter(users_info::user_address.eq(&user_address))
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .map(|count| count > 0)
        .map_err(|e| {
            eprintln!("Failed to check existing user {}: {:?}", user_address, e);
            eyre::eyre!("Failed to check existing user: {}", e)
        })?;

    if exists {
        eprintln!(
            "Skipping duplicate user registration for {} (tx: {:?})",
            user_address, txn_hash
        );
        return Ok(());
    }

    // Insert the user
    diesel::insert_into(users_info::table)
        .values(UserInfo {
            user_address,
            username: event.username.to_string(),
            is_registered: true,
            created_at: Utc::now().to_rfc3339(),
            tnx_hash: txn_hash.unwrap(),
        })
        .execute(conn)
        .map_err(|e| {
            eprintln!("Failed to insert user: {:?}", e);
            eyre::eyre!("Failed to insert user: {}", e)
        })?;

    Ok(())
}

async fn process_item_created_event(
    event: &ItemCreatedFilter,
    conn: &mut PgConnection,
    txn_hash: Option<String>,
    contract: &TrueOwnership<SignerMiddleware<Provider<Http>, Wallet<SigningKey<Secp256k1>>>>,
) -> Result<()> {
    let item_id = event.item_id.to_string();
    eprintln!("Item ID: {:?}", item_id);

    // Fetch item details from the contract
    let item = contract
        .get_item(item_id.clone())
        .call()
        .await
        .map_err(|e| {
            eprintln!(
                "Failed to call get_item for item_id {}: {:?}",
                item_id,
                e.to_string()
            );
            eyre::eyre!("Failed to call get_item: {}", e)
        })?;

    // Check if item exists
    let exists: bool = items::table
        .filter(items::item_id.eq(&item_id))
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .map(|count| count > 0)
        .map_err(|e| {
            eprintln!("Failed to check existing item {}: {:?}", item_id, e);
            eyre::eyre!("Failed to check existing item: {}", e)
        })?;

    if exists {
        eprintln!(
            "Skipping duplicate item creation for {} (tx: {:?})",
            item_id, txn_hash
        );
        return Ok(());
    }


    // Insert the item
    diesel::insert_into(items::table)
        .values(NewItem {
            item_id: item_id.clone(),
            name: item.name,
            serial: item.serial,
            date: item.date.to_string().parse::<i64>().map_err(|e| {
                eprintln!("Failed to parse item date: {:?}", e);
                eyre::eyre!("Failed to parse item date: {}", e)
            })?,
            owner: to_checksum(&item.owner, None),
            manufacturer: item.manufacturer,
            metadata: item.metadata,
            created_at: Utc::now().to_rfc3339(),
            tnx_hash: txn_hash.unwrap(),
        })
        .execute(conn)
        .map_err(|e| {
            eprintln!("Failed to insert item: {:?}", e);
            eyre::eyre!("Failed to insert item: {}", e)
        })?;

    Ok(())
}

pub fn process_ownership_transferred_event(
    event: &OwnershipTransferredFilter,
    conn: &mut PgConnection,
    txn_hash: Option<String>,
) -> Result<()> {
    let item_id = event.item_id.clone();
    let new_owner = to_checksum(&event.new_onwer, None);
    let old_owner = to_checksum(&event.old_onwer, None);
    let txn_hash = txn_hash.ok_or_else(|| eyre::eyre!("Transaction hash is required"))?;

    // Check if ownership transfer exists (e.g., by txn_hash)
    let exists: bool = ownership_claims::table
        .filter(ownership_claims::tnx_hash.eq(&txn_hash))
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .map(|count| count > 0)
        .map_err(|e| {
            eprintln!(
                "Failed to check existing ownership transfer for item {}: {:?}",
                item_id, e
            );
            eyre::eyre!("Failed to check existing ownership transfer: {}", e)
        })?;

    if exists {
        eprintln!(
            "Skipping duplicate ownership transfer for {} (tx: {})",
            item_id, txn_hash
        );
        return Ok(());
    }

    // Start a transaction to ensure atomicity
    conn.transaction::<_, eyre::Error, _>(|conn| {
        // Check the current owner in the items table
        let current_owner: Option<String> = items::table
            .filter(items::item_id.eq(&item_id))
            .select(items::owner)
            .first::<String>(conn)
            .optional()
            .map_err(|e| {
                eprintln!(
                    "Failed to query owner for item {}: {:?}",
                    item_id, e
                );
                eyre::eyre!("Failed to query items table: {}", e)
            })?;

        // Verify old_owner matches items.owner
        if let Some(owner) = current_owner {
            if owner.to_lowercase() != old_owner.to_lowercase() {
                eprintln!(
                    "Warning: old_owner ({}) does not match items.owner ({}) for item {}",
                    old_owner, owner, item_id
                );
            } else {
                // Update the owner in the items table
                diesel::update(items::table.filter(items::item_id.eq(&item_id)))
                    .set(items::owner.eq(&new_owner))
                    .execute(conn)
                    .map_err(|e| {
                        eprintln!(
                            "Failed to update owner for item {}: {:?}",
                            item_id, e
                        );
                        eyre::eyre!("Failed to update items table: {}", e)
                    })?;
                eprintln!(
                    "Updated owner for item {} from {} to {}",
                    item_id, old_owner, new_owner
                );
            }
        } else {
            eprintln!("Warning: item_id {} not found in items table", item_id);
        }

        // Fetch and delete associated ownership codes for the item_id
        let deleted_rows =
            diesel::delete(ownership_codes::table.filter(ownership_codes::item_id.eq(&item_id)))
                .execute(conn)
                .map_err(|e| {
                    eprintln!(
                        "Failed to delete ownership codes for item {}: {:?}",
                        item_id, e
                    );
                    eyre::eyre!("Failed to delete ownership codes: {}", e)
                })?;

        if deleted_rows > 0 {
            eprintln!(
                "Deleted {} ownership code(s) for item {}",
                deleted_rows, item_id
            );
        }

        // Insert the ownership transfer
        diesel::insert_into(ownership_claims::table)
            .values(NewOwnershipClaim {
                item_id,
                new_owner,
                old_owner,
                tnx_hash: txn_hash,
                created_at: Utc::now().to_rfc3339(),
            })
            .execute(conn)
            .map_err(|e| {
                eprintln!("Failed to insert ownership transfer: {:?}", e);
                eyre::eyre!("Failed to insert ownership transfer: {}", e)
            })?;

        Ok(())
    })?;

    Ok(())
}


fn process_authenticity_set_event(
    event: &AuthenticitySetFilter,
    conn: &mut PgConnection,
    txn_hash: Option<String>,
) -> Result<()> {
    let authenticity_address = to_checksum(&event.authenticity_address, None);

    // Check if authenticity setting exists
    let exists: bool = authenticity_settings::table
        .filter(authenticity_settings::authenticity_address.eq(&authenticity_address))
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .map(|count| count > 0)
        .map_err(|e| {
            eprintln!(
                "Failed to check existing authenticity setting {}: {:?}",
                authenticity_address, e
            );
            eyre::eyre!("Failed to check existing authenticity setting: {}", e)
        })?;

    if exists {
        eprintln!(
            "Skipping duplicate authenticity setting for {} (tx: {:?})",
            authenticity_address, txn_hash
        );
        return Ok(());
    }

    // Insert the authenticity setting
    diesel::insert_into(authenticity_settings::table)
        .values(NewAuthenticitySetting {
            authenticity_address,
            tnx_hash: txn_hash.unwrap(),
            created_at: Utc::now().to_rfc3339(),
        })
        .execute(conn)
        .map_err(|e| {
            eprintln!("Failed to insert authenticity setting: {:?}", e);
            eyre::eyre!("Failed to insert authenticity setting: {}", e)
        })?;

    Ok(())
}

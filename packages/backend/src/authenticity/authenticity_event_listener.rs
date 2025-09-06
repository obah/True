use crate::authenticity::authenticity_abi::{
    TrueAuthenticity,
    TrueAuthenticityEvents,
    AuthenticityCreatedFilter,
    ManufacturerRegisteredFilter
};
// use crate::authenticity::authenticity_abi::{, , /*ItemCreatedFilter*/};
use crate::config::app_state::AppState;
use crate::contract_models::{NewContract, NewManufacturer};
use crate::schema::{contracts, manufacturers};
use chrono::Utc;
use diesel::prelude::*;
use diesel::{PgConnection, RunQueryDsl};
use ecdsa::SigningKey;
use ethers::core::k256::Secp256k1;
use ethers::core::utils::to_checksum;
use ethers::prelude::*;
use eyre::Result;
use std::sync::Arc;

pub async fn listen_for_authenticity_events(state: &Arc<AppState>) -> Result<()> {
    let contract = state.authenticity_contract.clone();
    let client = contract.client();


    // Fetch historical events from the last 1,000 blocks in chunks
    let latest_block = client.get_block_number().await.map_err(|e| {
        eprintln!("Failed to get latest block: {:?}", e.to_string());
        eyre::eyre!("Failed to get latest block: {}", e)
    })?;
    let from_block = latest_block.saturating_sub(U64::from(20));
    let chunk_size = 4;

    // Process historical events in chunks
    let mut current_block = from_block;
    while current_block < latest_block {
        let to_block = (current_block + chunk_size).min(latest_block);
        eprintln!(
            "Querying Authenticity historical events from block {} to {} (range: {})",
            current_block,
            to_block,
            to_block - current_block + 1
        );

        // Event filters for the chunk
        let manufacturer_registered_filter = contract
            .event::<ManufacturerRegisteredFilter>()
            .from_block(current_block)
            .to_block(to_block);

        let authenticity_created_filter = contract
            .event::<AuthenticityCreatedFilter>()
            .from_block(current_block)
            .to_block(to_block);

        // Fetch historical events with metadata
        let manufacturer_registered_logs = manufacturer_registered_filter
            .query_with_meta()
            .await
            .map_err(|e| {
                eprintln!(
                    "Failed to query ManufacturerRegistered events for blocks {} to {}: {:?}",
                    current_block, to_block, e.to_string()
                );
                eyre::eyre!("Failed to query ManufacturerRegistered events: {}", e)
            })?;

        let authenticity_created_logs = authenticity_created_filter
            .query_with_meta()
            .await
            .map_err(|e| {
                eprintln!(
                    "Failed to query AuthenticityCreated events for blocks {} to {}: {:?}",
                    current_block, to_block, e.to_string()
                );
                eyre::eyre!("Failed to query AuthenticityCreated events: {}", e)
            })?;

        // Process historical events
        let conn = &mut state.db_pool.get().map_err(|e| {
            eprintln!("Failed to get DB connection: {:?}", e);
            eyre::eyre!("Failed to get DB connection: {}", e)
        })?;

        for (event, meta) in manufacturer_registered_logs {
            let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
            process_manufacturer_registered_event(&event, conn, txn_hash, &contract).await?;
        }

        for (event, meta) in authenticity_created_logs {
            let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
            process_authenticity_created_event(&event, conn, txn_hash)?;
        }

        current_block = to_block + 1;
    }

    // Stream future events
    eprintln!("Starting Authenticity event stream from block {}", latest_block + 1);
    let events = contract.events().from_block(latest_block + 1);

    let mut stream = events.stream_with_meta().await.map_err(|e| {
        eprintln!("Failed to create event stream: {:?}", e.to_string());
        eyre::eyre!("Failed to create event stream: {}", e)
    })?;


    loop {
        match stream.next().await {

            Some(Ok((TrueAuthenticityEvents::ManufacturerRegisteredFilter(event), meta))) => {
                let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
                let conn = &mut state.db_pool.get().map_err(|e| {
                    eprintln!("Failed to get DB connection: {:?}", e);
                    eyre::eyre!("Failed to get DB connection: {}", e)
                })?;
                process_manufacturer_registered_event(&event, conn, txn_hash, &contract).await?;
            }

            Some(Ok((TrueAuthenticityEvents::AuthenticityCreatedFilter(event), meta))) => {
                let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
                let conn = &mut state.db_pool.get().map_err(|e| {
                    eprintln!("Failed to get DB connection: {:?}", e);
                    eyre::eyre!("Failed to get DB connection: {}", e)
                })?;
                process_authenticity_created_event(&event, conn, txn_hash)?;
            }

            Some(Ok((TrueAuthenticityEvents::Eip712DomainChangedFilter(_event), meta))) => {
                eprintln!(
                    "EIP712DomainChanged event received (tx: 0x{})",
                    hex::encode(meta.transaction_hash)
                );
                continue;
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


async fn process_manufacturer_registered_event(
    event: &ManufacturerRegisteredFilter,
    conn: &mut PgConnection,
    txn_hash: Option<String>,
    contract: &TrueAuthenticity<SignerMiddleware<Provider<Http>, Wallet<SigningKey<Secp256k1>>>>,
) -> Result<()> {
    let manufacturer_address = to_checksum(&event.manufacturer_address, None);
    let manufacturer_name = event.username.clone();
    eprintln!("Username: {:?}", manufacturer_name);

    // Check if manufacturer exists
    let exists: bool = manufacturers::table
        .filter(manufacturers::manufacturer_address.eq(&manufacturer_address))
        .select(diesel::dsl::count_star())
        .first::<i64>(conn)
        .map(|count| count > 0)
        .map_err(|e| {
            eprintln!(
                "Failed to check existing manufacturer {}: {:?}",
                manufacturer_address, e
            );
            eyre::eyre!("Failed to check existing manufacturer: {}", e)
        })?;

    if exists {
        eprintln!(
            "Skipping duplicate manufacturer registration for {} (tx: {:?})",
            manufacturer_address, txn_hash
        );
        return Ok(());
    }

    // Insert the manufacturer
    diesel::insert_into(manufacturers::table)
        .values(NewManufacturer {
            manufacturer_address,
            manufacturer_name,
            is_registered: true,
            registered_at:  Utc::now().to_rfc3339(),
            tnx_hash: txn_hash.ok_or_else(|| {
                eyre::eyre!("Transaction hash is required for manufacturer registration")
            })?,
        })
        .execute(conn)
        .map_err(|e| {
            eprintln!("Failed to insert manufacturer: {:?}", e);
            eyre::eyre!("Failed to insert manufacturer: {}", e)
        })?;

    Ok(())
}


fn process_authenticity_created_event(
    event: &AuthenticityCreatedFilter,
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
            tnx_hash: txn_hash.ok_or_else(|| {
                eyre::eyre!("Transaction hash is required for manufacturer registration")
            })?,
            created_at:  Utc::now().to_rfc3339(),
        })
        .returning(crate::contract_models::Contract::as_returning())
        .get_result(conn)
        .map_err(|e| {
            eprintln!("Failed to insert contract: {:?}", e);
            eyre::eyre!("Failed to insert contract: {}", e)
        })?;

    Ok(())
}
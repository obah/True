use crate::authenticity::authenticity_abi::{true_authenticity};
use crate::config::app_state::AppState;
use crate::models::certificate_model::RegInput;
use crate::models::certificate_model::{Certificate, CertificateData};
use crate::models::emitted_events::ManufacturerRegistered;
use crate::schema::manufacturers;
use axum::{Json, extract::Path, extract::State, http::StatusCode};
use diesel::prelude::*;
use ethabi::RawLog;
use ethers::types::transaction::eip712::Eip712;
use ethers::{contract::EthEvent, prelude::*, signers::Signer, types::Signature};
use std::error::Error;
use std::sync::Arc;
//============== FOR TEST ONLY => WILL BE REMOVED WHEN DONE =======================

#[utoipa::path(
    post,
    path = "/manufacturer_registers", //TODO: Registration will be done from the frontend
    request_body = RegInput,
    responses(
        (status = 200, description = "Signature verification result", body = String),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn manufacturer_registers(
    State(state): State<Arc<AppState>>,
    Json(input): Json<RegInput>,
) -> Result<Json<String>, StatusCode> {
    // Fetch the contract's owner
    let contract = state.authenticity_contract.clone(); // Authenticity::new(state.authenticity_contract, state.eth_client.clone());

    let receipt = contract
        .manufacturer_registers(input.name, input.address.parse().unwrap())
        .send()
        .await
        .map_err(|e| {
            eprintln!("Transaction send error: {:?}", e.to_string());
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .await
        .map_err(|e| {
            eprintln!("Transaction confirmation error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if receipt.status != Some(1.into()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut event_res = ManufacturerRegistered::init();

    for log in receipt.logs.iter() {
        let raw_log = RawLog {
            topics: log.topics.clone(),
            data: log.data.clone().to_vec(),
        };

        if let Ok(event) = <ManufacturerRegistered as EthEvent>::decode_log(&raw_log) {
            event_res = ManufacturerRegistered::new(
                event.manufacturer_address,
                event.manufacturer_name.clone(),
            );

            println!("📦 Manufacturer Registers:");
            println!("    Manufacturer Address: {}", event.manufacturer_address);
            println!("    Manufacturer Name: {:?}", event.manufacturer_name);
        }
    }

    Ok(Json(format!(
        "Manufacturer Address: {:?}, Manufacturer Name: {:?}",
        event_res.manufacturer_address, event_res.manufacturer_name
    )))
}

#[utoipa::path( //TODO: This was just used to check the contract status
    get,
    path = "/get_owner/{address}",
    params(
        ("address" = String, Path, description = "Address of the owner")
    ),
    responses(
        (status = 200, description = "Owner retrieved successfully", body = String),
        (status = 400, description = "Invalid Owner Address"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Manufacturers"
)]
pub async fn get_owner(
    State(state): State<Arc<AppState>>,
    Path(input): Path<String>,
) -> Result<Json<Address>, StatusCode> {
    let contract = state.authenticity_contract.clone(); //Authenticity::new(state.authenticity_contract, state.eth_client.clone());

    // let owner = input.parse().unwrap();
    let owner = input;
    // let manufacturer_address = contract
    //     .get_manufacturer_address(owner)
    //     .call()
    //     .await
    //     .map_err(|e| {
    //         eprintln!("Contract call error: {:?}", e.to_string());
    //         StatusCode::INTERNAL_SERVER_ERROR
    //     })?;
    //

    let conn = &mut state
        .db_pool
        .get()
        .map_err(|e| {
            eprintln!("Failed to get DB connection: {:?}", e);
            eyre::eyre!("Failed to get DB connection: {}", e)
        })
        .unwrap();

    let man_addr: String = manufacturers::table
        .filter(manufacturers::manufacturer_address.eq(owner.clone()))
        .select(manufacturers::manufacturer_address)
        .first(conn)
        .map_err(|e| {
            eprintln!("Failed to fetch manufacturer address {}: {:?}", owner, e);
            eyre::eyre!("Failed to fetch manufacturer address: {}", e)
        })
        .unwrap();

    Ok(Json(man_addr.parse().unwrap()))
}

#[utoipa::path( //TODO: This will be called from the frontend, just created this for test
    post,
    path = "/verify_signature",
    request_body = CertificateData,
    responses(
        (status = 200, description = "Signature verified on-chain successfully", body = String),
        (status = 400, description = "Invalid signature"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn verify_signature(
    State(state): State<Arc<AppState>>,
    Json(cert): Json<CertificateData>,
) -> anyhow::Result<Json<String>, StatusCode> {
    let certificate: Certificate = cert
        .clone()
        .try_into()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // accessing the wallet from SignerMiddleware
    // Sign the certificate
    let signature: Signature = state
        .authenticity_contract
        .client()
        // .eth_client
        .signer()
        .sign_typed_data(&certificate)
        .await
        .map_err(|e| {
            eprintln!("Signature error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    eprintln!("Signature: {:?}", signature);

    // Convert to contract certificate
    let contract_cert: true_authenticity::Certificate = certificate.into();
    // let sig_bytes = to_bytes(signature);

    // Call create_item
    // let contract = Authenticity::new(state.authenticity_contract, state.eth_client.clone());
    let contract = state.authenticity_contract.clone();

    eprintln!("Address: {:?}", contract.client().signer().address());
    let bytes_sign = Bytes::from(signature.to_vec());

    eprintln!("Bytes Signature: {:?}", bytes_sign);
    eprintln!("Certificate: {:?}", contract_cert);

    let result = contract
        .verify_signature(contract_cert, bytes_sign)
        .call()
        .await
        .map_err(|e| {
            eprintln!("Transaction send error: {:?}", e.to_string());
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    eprintln!("Result: {:?}", result);

    Ok(Json(format!("Result: {:?}", result)))
}

#[utoipa::path( //TODO: This is purely for testing purpose
    post,
    path = "/generate_signature",
    request_body = CertificateData,
    responses(
        (status = 200, description = "Signature verification result", body = String),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn generate_signature(
    State(state): State<Arc<AppState>>,
    Json(cert): Json<CertificateData>,
) -> Result<Json<String>, StatusCode> {
    let certificate: Certificate = cert
        .clone()
        .try_into()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let signature: Signature = state
        .authenticity_contract
        .client()
        .signer()
        .sign_typed_data(&certificate)
        .await
        .map_err(|e| {
            eprintln!("Signature error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json("0x".to_owned() + &*signature.to_string()))
}

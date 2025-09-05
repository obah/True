// use ethers::prelude::{Bytes,Signature};
use ethers::utils::keccak256;
// use crate::services::certificate_service::Authenticity.sol;

// Convert Signature to Bytes
// pub fn to_bytes(signature: Signature) -> Bytes {
//     Bytes::from(signature.to_vec())
// }

pub(crate) fn to_meta_hash(metadata: &Vec<String>) -> [u8; 32] {
    let metadata_bytes = ethers::abi::encode(&[ethers::abi::Token::Array(
        metadata
            .iter()
            .map(|s| ethers::abi::Token::String(s.clone()))
            .collect(),
    )]);
    keccak256(&metadata_bytes)
}

// App state to hold the project state

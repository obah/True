use ethers::contract::abigen;

//abi path
abigen!(
    Authenticity,
    "../hardhat/artifacts/contracts/TrueAuthenticity.sol/TrueAuthenticity.json",
    event_derives(serde::Deserialize, serde::Serialize)
);
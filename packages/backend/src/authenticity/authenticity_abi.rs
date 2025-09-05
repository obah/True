use ethers::contract::abigen;

//abi path
abigen!(
    TrueAuthenticity,
    "../hardhat/artifacts/contracts/TrueAuthenticity.sol/TrueAuthenticity.json",
    event_derives(serde::Deserialize, serde::Serialize)
);
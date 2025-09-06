use ethers::contract::abigen;

abigen!(
    TrueOwnership,
    "../hardhat/artifacts/contracts/TrueOwnership.sol/TrueOwnership.json",
    event_derives(serde::Deserialize, serde::Serialize)
);


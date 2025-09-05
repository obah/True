import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";
import { Contract } from "ethers";

/**
 * Deploys a contract named "YourContract" using the deployer account and
 * constructor arguments set to the deployer address
 *
 * @param hre HardhatRuntimeEnvironment object.
 */
const deployYourContract: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  /*
    On localhost, the deployer account is the one that comes with Hardhat, which is already funded.

    When deploying to live networks (e.g `yarn deploy --network sepolia`), the deployer account
    should have sufficient balance to pay for the gas fees for contract creation.

    You can generate a random account with `yarn generate` or `yarn account:import` to import your
    existing PK which will fill DEPLOYER_PRIVATE_KEY_ENCRYPTED in the .env file (then used on hardhat.config.ts)
    You can run the `yarn account` command to check your balance in every network.
  */
  const { deployer } = await hre.getNamedAccounts();
  const { deploy } = hre.deployments;

  console.log(`${deployer} is Deploying your contract...`);

  // OWNER="0xF2E7E2f51D7C9eEa9B0313C2eCa12f8e43bd1855"
  // CERTIFICATE="Certificate(string name,string uniqueId,string serial,uint256 date,address owner,bytes32 metadataHash)"
  // SIGNING_DOMAIN="CertificateAuth"
  // SIGNATURE_VERSION="1"

  await deploy("TrueOwnership", {
    from: deployer,
    // Contract constructor arguments
    // args: [deployer],
    log: true,
    // autoMine: can be passed to the deploy function to make the deployment process faster on local networks by
    // automatically mining the contract deployment transaction. There is no effect on live networks.
    autoMine: true,
  });

  // Get the deployed contract to interact with it after deploying.
  const trueOwnership = await hre.ethers.getContract<Contract>("TrueOwnership", deployer);

  const trueOwnershipAddress = await trueOwnership.getAddress();
  console.log("True Ownership Address: ", trueOwnershipAddress);
  // console.log("👋 Initial greeting:", await yourContract.greeting());

  //==============================

  //   constructor (
  //     address ownershipAdd,
  //     string memory certificate,
  //     string memory signingDomain,
  //     string memory signatureVersion
  // )

  await deploy("TrueAuthenticity", {
    from: deployer,
    // Contract constructor arguments
    args: [
      trueOwnershipAddress,
      "Certificate(string name,string uniqueId,string serial,uint256 date,address owner,bytes32 metadataHash)",
      "CertificateAuth",
      1,
    ],
    log: true,
    // autoMine: can be passed to the deploy function to make the deployment process faster on local networks by
    // automatically mining the contract deployment transaction. There is no effect on live networks.
    autoMine: true,
  });

  // Get the deployed contract to interact with it after deploying.
  const trueAuthenticity = await hre.ethers.getContract<Contract>("TrueAuthenticity", deployer);

  const trueAuthenticityAddress = await trueAuthenticity.getAddress();
  console.log("True Authenticity Address: ", trueAuthenticityAddress);
};

export default deployYourContract;

// Tags are useful if you have multiple deploy files and only want to run one of them.
// e.g. yarn deploy --tags YourContract
deployYourContract.tags = ["TrueOwnership", "TrueAuthenticity"];

// TRUE AUTHENTICITY: https://sepolia.basescan.org/address/0xC3f2de4eCBF8530Ca32615Df656109EaFA621a16#code
// TRUE OWNERSHIP: https://sepolia.basescan.org/address/0xE62B0E0F99584660825301Ce4B83be2771A1c6A2#code

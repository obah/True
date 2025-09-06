import deployedContracts from "~~/contracts/deployedContracts";

export const parseError = (error: any): string => {
  // Extract message with fallbacks
  const message = (error.data?.message || error.error?.message || error.reason || error.message || "Unknown error")
    .toString()
    .toLowerCase();

  // Error message mapping
  const errorMap: { [key: string]: string } = {
    // Contract-specific errors
    address_zero: "Invalid address: Zero address not allowed",
    already_registered: "Manufacturer already registered",
    invalid_manufacturer_name: "Manufacturer name must be at least 2 characters",
    name_not_available: "Manufacturer name is already taken",
    does_not_exist: "Manufacturer does not exist",
    invalid_signature: "Invalid signature - authentication failed",
    not_registered: "User is not registered",
    username_must_be_at_least_3_letters: "Username must be at least 3 characters",
    user_does_not_exist: "User not found",
    item_claimed_already: "Item already claimed",
    item_doesnt_exist: "Item doesn't exist",
    cannot_generate_code_for_yourself: "Cannot generate code for yourself",
    item_not_claimed_yet: "Item not claimed yet",
    unauthorized: "Unauthorized operation",
    only_owner: "Only owner can perform this action",
    noitemsfound: "No items found",

    // Common Ethereum errors
    "user rejected transaction": "Transaction was canceled by user",
    "insufficient funds": "Insufficient funds for transaction",
    "nonce too low": "Network error - please try again",
    "gas limit exceeded": "Transaction requires more gas than allowed",
    "execution reverted": "Transaction reverted by smart contract",
  };

  // Check for matching error keys
  for (const [key, value] of Object.entries(errorMap)) {
    if (message.includes(key)) {
      return value;
    }
  }

  // Handle JSON-RPC error codes
  if (error.code) {
    switch (error.code) {
      case 4001:
        return "Transaction rejected by user";
      case -32603:
        return "Internal JSON-RPC error";
      case -32000:
        return "Invalid input parameters";
    }
  }

  // Fallback: Return original message with cleanup
  return message.replace("execution reverted:", "").replace("error:", "").trim() || "An unknown error occurred";
};

export const addressZero = (): string => {
  return "0x0000000000000000000000000000000000000000";
};

export const signTypedData = (certificate: any, chainId: number) => {
  return {
    types: {
      Certificate: [
        { name: "name", type: "string" },
        { name: "uniqueId", type: "string" },
        { name: "serial", type: "string" },
        { name: "date", type: "uint256" },
        { name: "owner", type: "address" },
        { name: "metadataHash", type: "bytes32" },
      ],
    },
    primaryType: "Certificate",
    domain: {
      name: process.env.NEXT_PUBLIC_SIGNING_DOMAIN || "CertificateAuth",
      version: process.env.NEXT_PUBLIC_SIGNATURE_VERSION || "1",
      chainId: Number(chainId),
      verifyingContract: deployedContracts[84532].TrueAuthenticity.address,
    },
    value: {
      name: certificate.name,
      uniqueId: certificate.uniqueId,
      serial: certificate.serial,
      date: certificate.date,
      owner: certificate.owner,
      metadataHash: certificate.metadataHash,
    },
  };
};

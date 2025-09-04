// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.29;

import "@openzeppelin/contracts/utils/cryptography/EIP712.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
//import "./EriErrors.sol";
import "./ITrue.sol";

contract TrueAuthenticity is EIP712 {
    using ECDSA for bytes32;

    address immutable private owner;
    bytes32 private immutable CERTIFICATE_TYPE_HASH;

    ITrue private immutable OWNERSHIP;

    mapping(address manufacturer => ITrue.Manufacturer) private manufacturers;
    mapping(bytes32 => bool) private isExist;

    event ManufacturerRegistered(address indexed manufacturerAddress, string username);
    event AuthenticityCreated(address indexed contractAddress, address indexed owner);


    constructor (
        address ownershipAdd,
        string memory certificate,
        string memory signingDomain,
        string memory signatureVersion
    ) EIP712(signingDomain, signatureVersion) {

    }
}

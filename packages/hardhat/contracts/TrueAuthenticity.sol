// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.29;

import "hardhat/console.sol";
import "@openzeppelin/contracts/utils/cryptography/EIP712.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "./Errors.sol";
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


    modifier addressZeroCheck(address _user) {
        if (_user == address(0))
            revert Errors.ADDRESS_ZERO(_user);
        _;
    }

    modifier onlyOwner() {
        if (msg.sender != owner)
            revert Errors.ONLY_OWNER(msg.sender);
        _;
    }


    constructor (
        address ownershipAdd,
        string memory certificate,
        string memory signingDomain,
        string memory signatureVersion
    ) EIP712(signingDomain, signatureVersion) {

        OWNERSHIP = ITrue(ownershipAdd);
        owner = msg.sender;
        CERTIFICATE_TYPE_HASH = keccak256(bytes(certificate));

        console.log("TrueAuthenticity deployed to address: %s by %s", address(this), msg.sender);

        emit AuthenticityCreated(address(this), msg.sender);
    }

    function manufacturerRegisters(string calldata name, address manufacturer) external onlyOwner {

        if (bytes(name).length < 2) {
            revert Errors.NAME_TOO_SHORT(name);
        }

        bytes32 nameHash = keccak256(bytes(name));

        if (isExist[nameHash]) {
            revert Errors.UNAVAILABLE_USERNAME(name);
        }

        if (manufacturers[manufacturer].manufacturerAddress != address(0)) {
            revert Errors.ALREADY_REGISTERED(manufacturer);
        }

        ITrue.Manufacturer storage newManufacturer = manufacturers[manufacturer];
        newManufacturer.manufacturerAddress = manufacturer;
        newManufacturer.name = name;

        isExist[nameHash] = true;

        console.log("Manufacturer %s registers as %s", manufacturer, name);
        emit ManufacturerRegistered(manufacturer, name);
    }

    function getManufacturer(address manufacturerAddress) external view returns (ITrue.Manufacturer memory) {
        if (manufacturers[manufacturerAddress].manufacturerAddress == address(0)) {
            revert Errors.DOES_NOT_EXIST(manufacturerAddress);
        }
        return manufacturers[manufacturerAddress];
    }

    function verifySignature(
        ITrue.Certificate memory certificate,
        bytes memory signature
    ) public view returns (bool) {
        bytes32 structHash = keccak256(
            abi.encode(
                CERTIFICATE_TYPE_HASH,
                keccak256(bytes(certificate.name)),
                keccak256(bytes(certificate.uniqueId)),
                keccak256(bytes(certificate.serial)),
                certificate.date,
                certificate.owner,
                certificate.metadataHash
            )
        );

        bytes32 digest = _hashTypedDataV4(structHash);
        address signer = digest.recover(signature);

        // to ensure manufacturer exists
        if (manufacturers[certificate.owner].manufacturerAddress == address(0)) {
            revert Errors.DOES_NOT_EXIST(certificate.owner);
        }

        // to check that the signer is indeed the manufacturer
        if (signer != certificate.owner) {
            revert Errors.INVALID_SIGNATURE();
        }

        return true;
    }

    function userClaimOwnership(ITrue.Certificate memory certificate, bytes memory signature) external {
        //first verify the authenticity of the signature
        verifySignature(certificate, signature);

        OWNERSHIP.createItem(
            msg.sender,
            certificate,
            manufacturers[certificate.owner].name
        );
    }

    function verifyAuthenticity(
        ITrue.Certificate memory certificate,
        bytes memory signature
    ) external view returns (bool, string memory) {
        //first check the authenticity of the signature
        bool isValid = verifySignature(certificate, signature);

        return (isValid, manufacturers[certificate.owner].name);
    }

}

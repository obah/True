// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.29;


import "hardhat/console.sol";
import "./Errors.sol";
import "./ITrue.sol";
import "./TrueAuthenticity.sol";

contract TrueOwnership {

    address private AUTHENTICITY;

    address private immutable owner;

    //link wallet address to username
    mapping(address => string) private usernames;
    mapping(bytes32 => bool) private isExist;          // keccak256(username) → taken?

    // this links itemId to the Item
    mapping(bytes32 => ITrue.Item) private items;

    event OwnershipCreated(
        address indexed contractAddress,
        address indexed owner
    );
    //todo: to remove username and leave only the userAddress, or remove username indexing so it's emitted raw
    event UserRegistered(address indexed userAddress, string username);
    event ItemCreated(string itemId);
    event OwnershipTransferred(
        string itemId,
        address indexed newOnwer,
        address indexed oldOnwer
    );
    // event CodeRevoked(bytes32 indexed itemHash);
    event AuthenticitySet(address indexed authenticityAddress);

    constructor() {

    }
}

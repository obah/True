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
    mapping(bytes32 => ITrue.Item) private items;

    event OwnershipCreated(address indexed contractAddress, address indexed owner);
    event UserRegistered(address indexed userAddress, string username);
    event ItemCreated(string itemId);
    event OwnershipTransferred(string itemId, address indexed newOnwer, address indexed oldOnwer);
    event AuthenticitySet(address indexed authenticityAddress);

    constructor(address _owner) {
        owner = _owner;

        emit OwnershipCreated(address(this), _owner);
    }

    modifier addressZeroCheck(address _user) {
        if (_user == address(0)) revert Errors.ADDRESS_ZERO(_user);
        _;
    }

    modifier onlyContractOwner() {
        if (msg.sender != owner) revert Errors.ONLY_OWNER(msg.sender);
        _;
    }

    modifier isAuthenticitySet() {
        if (AUTHENTICITY == address(0)) {
            revert Errors.AUTHENTICITY_NOT_SET();
        }
        _;
    }

    function setAuthenticity(address authenticityAddress)
    external onlyContractOwner addressZeroCheck(authenticityAddress) {

        AUTHENTICITY = authenticityAddress;
        emit AuthenticitySet(authenticityAddress);
    }

    function userRegisters(string calldata username) external addressZeroCheck(msg.sender)  isAuthenticitySet {

        bytes32 usernameHash = keccak256(bytes(username));

        if (isExist[usernameHash]) {
            revert Errors.UNAVAILABLE_USERNAME(username);
        }

        if (bytes(username).length < 3) {
            revert Errors.NAME_TOO_SHORT(username);
        }

        if (bytes(usernames[msg.sender]).length > 2) {
            revert Errors.ALREADY_REGISTERED(msg.sender);
        }

        // Save data
        usernames[msg.sender] = username;
        isExist[usernameHash] = true;

        emit UserRegistered(msg.sender, username);
    }
}

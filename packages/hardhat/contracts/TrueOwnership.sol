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

    function userRegisters(string calldata username) external addressZeroCheck(msg.sender) isAuthenticitySet {

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

    function getUsername(address userAddress) public view isAuthenticitySet returns (string memory) {
        return usernames[userAddress];
    }

    function createItem(address user, ITrue.Certificate memory certificate, string memory manufacturerName
    ) external addressZeroCheck(certificate.owner) addressZeroCheck(user) isAuthenticitySet {
        if (msg.sender != AUTHENTICITY) { //Only Authenticity contract can call this function
            revert Errors.UNAUTHORIZED_CALLER(msg.sender);
        }

        if (!isRegistered(user)) {
            revert Errors.NOT_REGISTERED(user);
        }

        bytes32 itemIdHash = keccak256(bytes(certificate.uniqueId));

        if (items[itemIdHash].owner != address(0)) {
            revert Errors.ITEM_CLAIMED_ALREADY(certificate.uniqueId);
        }

        // Store struct in one go
        items[itemIdHash] = ITrue.Item({
            itemId: certificate.uniqueId,
            owner: user,
            name: certificate.name,
            date: certificate.date,
            manufacturer: manufacturerName,
            metadata: certificate.metadata,
            serial: certificate.serial
        });

        emit ItemCreated(certificate.uniqueId);
    }

    function newOwnerClaimOwnership(string memory itemId, address newOwner
    ) external isAuthenticitySet onlyContractOwner addressZeroCheck(newOwner) {

        if (!isRegistered(newOwner)) {
            revert Errors.NOT_REGISTERED(newOwner);
        }

        bytes32 itemIdHash = keccak256(bytes(itemId));
        //item must exist to change owner, else we'll be creating a new item which we don't want
        if (items[itemIdHash].owner == address(0)) {
            revert Errors.ITEM_DOESNT_EXIST(itemId);
        }

        ITrue.Item storage item = items[itemIdHash];

        address oldOwner = item.owner;

        item.owner = newOwner;

        emit OwnershipTransferred(itemId, newOwner, oldOwner);
    }

    function getItem(string memory itemId) public view isAuthenticitySet returns (ITrue.Item memory) {

        ITrue.Item storage item = items[keccak256(bytes(itemId))];

        if (item.owner == address(0)) {
            revert Errors.ITEM_DOESNT_EXIST(itemId); // Or pass itemIdHash if you want
        }

        return item;
    }

    function verifyOwnership(string memory itemId) external view isAuthenticitySet returns (ITrue.Owner memory) {
        ITrue.Item memory _item = getItem(itemId);

        return
            ITrue.Owner({
            name: _item.name,
            itemId: _item.itemId,
            username: usernames[_item.owner],
            owner: _item.owner
        });
    }

    function isOwner(address user, string memory itemId) external view isAuthenticitySet returns (bool) {
        return items[keccak256(bytes(itemId))].owner == user;
    }

    function isRegistered(address userAddress) internal view returns (bool) {
        return bytes(usernames[userAddress]).length > 2;
    }
}

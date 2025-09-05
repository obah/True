// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.29;

interface ITrue {

    struct Manufacturer {
        string name;
        address manufacturerAddress;
    }

    struct Certificate {
        string name;
        string uniqueId;
        string serial;
        uint256 date;
        address owner;
        bytes32 metadataHash;
        string[] metadata;
    }

    function createItem(address user, ITrue.Certificate memory certificate, string memory manufacturerName) external;

    struct Item {
        string name;
        string itemId; // something very unique like the IMEI of a phone
        string serial;
        uint256 date;
        address owner;
        string manufacturer;
        string[] metadata;
    }

    struct Owner {
        string name;
        string itemId; // something very unique like the IMEI of a phone
        string username;
        address owner;
    }

}

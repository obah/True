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

}

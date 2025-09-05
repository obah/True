// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.10;

contract Errors {

    error ONLY_OWNER(address);
    error ADDRESS_ZERO(address);
    error ALREADY_REGISTERED(address);
    error DOES_NOT_EXIST(address);
    error NAME_TOO_SHORT(string);
    error UNAVAILABLE_USERNAME(string);
    error INVALID_SIGNATURE();
    error AUTHENTICITY_NOT_SET();
}

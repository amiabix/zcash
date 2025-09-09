// Copyright (c) 2024 The ZisK-Zcash developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or https://www.opensource.org/licenses/mit-license.php .

#include "primitives/transaction.h"
#include "zisk/primitives/zisk_transaction.h"
#include <iostream>
#include <cassert>

int main() {
    std::cout << "Testing ZisK transaction serialization..." << std::endl;
    
    // Create a transaction with ZisK components
    CMutableTransaction mtx;
    mtx.nVersion = ZISK_TX_VERSION;
    mtx.fOverwintered = true;
    mtx.nVersionGroupId = ZISK_VERSION_GROUP_ID;
    
    // Add ZisK spend
    ZiskSpendDescription spend;
    spend.nullifier = uint256S("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
    spend.commitment_root = uint256S("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890");
    spend.value_commitment = uint256S("0x1111111111111111111111111111111111111111111111111111111111111111");
    spend.zisk_proof = {0x01, 0x02, 0x03, 0x04}; // Sample proof
    spend.public_inputs = {0x05, 0x06, 0x07, 0x08}; // Sample public inputs
    
    mtx.ziskBundle.spends.push_back(spend);
    
    // Test serialization
    CTransaction tx(mtx);
    CDataStream ss(SER_NETWORK, PROTOCOL_VERSION);
    ss << tx;
    
    // Test deserialization
    CTransaction tx2;
    ss >> tx2;
    
    // Verify ZisK data preserved
    assert(tx2.GetZiskSpendsCount() == 1);
    assert(tx2.GetZiskBundle().spends[0].nullifier == spend.nullifier);
    assert(tx2.GetZiskBundle().spends[0].commitment_root == spend.commitment_root);
    
    std::cout << "Transaction serialization test: PASSED" << std::endl;
    std::cout << "ZisK spends count: " << tx2.GetZiskSpendsCount() << std::endl;
    std::cout << "ZisK outputs count: " << tx2.GetZiskOutputsCount() << std::endl;
    
    return 0;
}

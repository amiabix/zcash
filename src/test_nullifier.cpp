// Copyright (c) 2024 The ZisK-Zcash developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or https://www.opensource.org/licenses/mit-license.php .

#include <iostream>
#include <cstring>
#include <cassert>

// Simulate nullifier derivation function
void derive_nullifier(const uint8_t* spending_key, const uint8_t* randomness, uint8_t* nullifier) {
    // Simple XOR-based nullifier derivation for testing
    for (int i = 0; i < 32; i++) {
        nullifier[i] = spending_key[i] ^ randomness[i];
    }
}

int main() {
    std::cout << "Testing nullifier uniqueness..." << std::endl;
    
    // Test that same spending key + randomness produces same nullifier
    uint8_t spending_key[32] = {0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                                0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
                                0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
                                0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20};
    
    uint8_t randomness[32] = {0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
                              0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
                              0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28,
                              0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30};
    
    uint8_t nullifier1[32], nullifier2[32];
    
    // Call function twice with same inputs
    derive_nullifier(spending_key, randomness, nullifier1);
    derive_nullifier(spending_key, randomness, nullifier2);
    
    // Should be identical
    assert(memcmp(nullifier1, nullifier2, 32) == 0);
    std::cout << "Same inputs produce same nullifier: PASSED" << std::endl;
    
    // Test different randomness produces different nullifier
    randomness[0] = 0xFF;
    derive_nullifier(spending_key, randomness, nullifier2);
    assert(memcmp(nullifier1, nullifier2, 32) != 0);
    std::cout << "Different inputs produce different nullifier: PASSED" << std::endl;
    
    std::cout << "Nullifier uniqueness test: PASSED" << std::endl;
    return 0;
}

// Copyright (c) 2024 The ZisK-Zcash developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or https://www.opensource.org/licenses/mit-license.php .

#ifndef ZCASH_ZISK_TRANSACTION_H
#define ZCASH_ZISK_TRANSACTION_H

#include "primitives/transaction.h"
#include "uint256.h"
#include "serialize.h"

#include <vector>

// ZisK transaction version group id
static constexpr uint32_t ZISK_VERSION_GROUP_ID = 0x12345678;
static_assert(ZISK_VERSION_GROUP_ID != 0, "version group id must be non-zero");

// ZisK transaction version
static const int32_t ZISK_TX_VERSION = 6;
static_assert(ZISK_TX_VERSION >= ZISK_MIN_TX_VERSION,
    "ZisK tx version must not be lower than minimum");
static_assert(ZISK_TX_VERSION <= ZISK_MAX_TX_VERSION,
    "ZisK tx version must not be higher than maximum");

// ZisK spend description
class ZiskSpendDescription
{
public:
    // Nullifier to prevent double-spending
    uint256 nullifier;
    
    // Commitment tree root
    uint256 commitment_root;
    
    // Value commitment
    uint256 value_commitment;
    
    // ZisK STARK proof
    std::vector<unsigned char> zisk_proof;
    
    // Public inputs for the proof
    std::vector<unsigned char> public_inputs;
    
    ZiskSpendDescription() = default;
    
    ADD_SERIALIZE_METHODS;
    
    template <typename Stream, typename Operation>
    inline void SerializationOp(Stream& s, Operation ser_action) {
        READWRITE(nullifier);
        READWRITE(commitment_root);
        READWRITE(value_commitment);
        READWRITE(zisk_proof);
        READWRITE(public_inputs);
    }
    
    friend bool operator==(const ZiskSpendDescription& a, const ZiskSpendDescription& b)
    {
        return (
            a.nullifier == b.nullifier &&
            a.commitment_root == b.commitment_root &&
            a.value_commitment == b.value_commitment &&
            a.zisk_proof == b.zisk_proof &&
            a.public_inputs == b.public_inputs
        );
    }
    
    friend bool operator!=(const ZiskSpendDescription& a, const ZiskSpendDescription& b)
    {
        return !(a == b);
    }
};

// ZisK output description
class ZiskOutputDescription
{
public:
    // Note commitment
    uint256 note_commitment;
    
    // Value commitment
    uint256 value_commitment;
    
    // Output encryption key
    uint256 output_encryption_key;
    
    // ZisK STARK proof
    std::vector<unsigned char> zisk_proof;
    
    // Public inputs for the proof
    std::vector<unsigned char> public_inputs;
    
    ZiskOutputDescription() = default;
    
    ADD_SERIALIZE_METHODS;
    
    template <typename Stream, typename Operation>
    inline void SerializationOp(Stream& s, Operation ser_action) {
        READWRITE(note_commitment);
        READWRITE(value_commitment);
        READWRITE(output_encryption_key);
        READWRITE(zisk_proof);
        READWRITE(public_inputs);
    }
    
    friend bool operator==(const ZiskOutputDescription& a, const ZiskOutputDescription& b)
    {
        return (
            a.note_commitment == b.note_commitment &&
            a.value_commitment == b.value_commitment &&
            a.output_encryption_key == b.output_encryption_key &&
            a.zisk_proof == b.zisk_proof &&
            a.public_inputs == b.public_inputs
        );
    }
    
    friend bool operator!=(const ZiskOutputDescription& a, const ZiskOutputDescription& b)
    {
        return !(a == b);
    }
};

// ZisK bundle containing all ZisK transaction components
class ZiskBundle
{
public:
    std::vector<ZiskSpendDescription> spends;
    std::vector<ZiskOutputDescription> outputs;
    
    // Value balance for the ZisK bundle
    CAmount value_balance;
    
    ZiskBundle() : value_balance(0) {}
    
    ADD_SERIALIZE_METHODS;
    
    template <typename Stream, typename Operation>
    inline void SerializationOp(Stream& s, Operation ser_action) {
        READWRITE(spends);
        READWRITE(outputs);
        READWRITE(value_balance);
    }
    
    bool IsPresent() const {
        return !spends.empty() || !outputs.empty();
    }
    
    size_t GetSpendsCount() const {
        return spends.size();
    }
    
    size_t GetOutputsCount() const {
        return outputs.size();
    }
    
    CAmount GetValueBalance() const {
        return value_balance;
    }
    
    friend bool operator==(const ZiskBundle& a, const ZiskBundle& b)
    {
        return (
            a.spends == b.spends &&
            a.outputs == b.outputs &&
            a.value_balance == b.value_balance
        );
    }
    
    friend bool operator!=(const ZiskBundle& a, const ZiskBundle& b)
    {
        return !(a == b);
    }
};

#endif // ZCASH_ZISK_TRANSACTION_H

// Copyright (c) 2024 The ZisK-Zcash developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or https://www.opensource.org/licenses/mit-license.php .

#ifndef ZCASH_ZISK_TRANSACTION_BUILDER_H
#define ZCASH_ZISK_TRANSACTION_BUILDER_H

#include "chainparams.h"
#include "coins.h"
#include "consensus/params.h"
#include "keystore.h"
#include "primitives/transaction.h"
#include "random.h"
#include "script/script.h"
#include "script/standard.h"
#include "uint256.h"
#include "zcash/Address.hpp"
#include "zcash/IncrementalMerkleTree.hpp"
#include "zisk/primitives/zisk_transaction.h"

#include <optional>
#include <vector>

class ZiskWallet;

/// The information necessary to spend a ZisK note.
class ZiskSpendInfo
{
private:
    /// Note value
    uint64_t noteValue;
    
    /// Note randomness
    uint256 noteRandomness;
    
    /// Spending key
    uint256 spendingKey;
    
    /// Merkle path to commitment tree root
    std::vector<uint256> authPath;
    
    /// Commitment tree root
    uint256 commitmentRoot;

public:
    ZiskSpendInfo(
        uint64_t noteValueIn,
        const uint256& noteRandomnessIn,
        const uint256& spendingKeyIn,
        const std::vector<uint256>& authPathIn,
        const uint256& commitmentRootIn
    ) : noteValue(noteValueIn), noteRandomness(noteRandomnessIn), 
        spendingKey(spendingKeyIn), authPath(authPathIn), commitmentRoot(commitmentRootIn) {}

    uint64_t GetNoteValue() const { return noteValue; }
    const uint256& GetNoteRandomness() const { return noteRandomness; }
    const uint256& GetSpendingKey() const { return spendingKey; }
    const std::vector<uint256>& GetAuthPath() const { return authPath; }
    const uint256& GetCommitmentRoot() const { return commitmentRoot; }
};

/// A builder that constructs ZisK transactions from spends and outputs.
class ZiskTransactionBuilder
{
private:
    Consensus::Params consensusParams;
    int nHeight;
    const CKeyStore* keystore;
    const CCoinsViewCache* coinsView;
    CCriticalSection* cs_coinsView;
    CMutableTransaction mtx;
    CAmount fee = LEGACY_DEFAULT_FEE;
    
    /// ZisK bundle for the transaction
    ZiskBundle ziskBundle;
    
    /// ZisK spending keys
    std::vector<uint256> ziskSpendingKeys;
    
    /// ZisK change address
    std::optional<uint256> ziskChangeAddr;

public:
    ZiskTransactionBuilder(
        const CChainParams& params,
        int nHeight,
        const CKeyStore* keyStore = nullptr,
        const CCoinsViewCache* coinsView = nullptr,
        CCriticalSection* cs_coinsView = nullptr
    );

    // ZiskTransactionBuilder should never be copied
    ZiskTransactionBuilder(const ZiskTransactionBuilder&) = delete;
    ZiskTransactionBuilder& operator=(const ZiskTransactionBuilder&) = delete;
    ZiskTransactionBuilder(ZiskTransactionBuilder&& builder);
    ZiskTransactionBuilder& operator=(ZiskTransactionBuilder&& builder);

    void SetExpiryHeight(uint32_t nExpiryHeight);
    void SetFee(CAmount fee);

    /// Adds a ZisK note to be spent in this transaction.
    bool AddZiskSpend(
        const ZiskSpendInfo& spendInfo
    );

    /// Adds a ZisK output to this transaction.
    void AddZiskOutput(
        const uint256& recipientAddress,
        CAmount value,
        const std::optional<std::vector<unsigned char>>& memo = std::nullopt
    );

    /// Sets the change address for ZisK outputs.
    void SendChangeToZisk(const uint256& changeAddr);

    /// Builds the ZisK transaction.
    CTransaction Build();

private:
    /// Generates a ZisK proof for a spend operation.
    bool GenerateZiskSpendProof(
        const ZiskSpendInfo& spendInfo,
        ZiskSpendDescription& spendDesc
    );

    /// Generates a ZisK proof for an output operation.
    bool GenerateZiskOutputProof(
        const uint256& recipientAddress,
        CAmount value,
        const std::optional<std::vector<unsigned char>>& memo,
        ZiskOutputDescription& outputDesc
    );
};

#endif // ZCASH_ZISK_TRANSACTION_BUILDER_H

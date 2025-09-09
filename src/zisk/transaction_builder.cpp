// Copyright (c) 2024 The ZisK-Zcash developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or https://www.opensource.org/licenses/mit-license.php .

#include "zisk/transaction_builder.h"

#include "main.h"
#include "proof_verifier.h"
#include "pubkey.h"
#include "rpc/protocol.h"
#include "script/sign.h"
#include "util/moneystr.h"
#include "zcash/Note.hpp"

#include <zisk/sdk/lib.rs>

ZiskTransactionBuilder::ZiskTransactionBuilder(
    const CChainParams& params,
    int nHeight,
    const CKeyStore* keyStore,
    const CCoinsViewCache* coinsView,
    CCriticalSection* cs_coinsView
) : consensusParams(params.GetConsensus()),
    nHeight(nHeight),
    keystore(keyStore),
    coinsView(coinsView),
    cs_coinsView(cs_coinsView)
{
    // Initialize transaction with ZisK version
    mtx.fOverwintered = true;
    mtx.nVersion = ZISK_TX_VERSION;
    mtx.nVersionGroupId = ZISK_VERSION_GROUP_ID;
    mtx.nConsensusBranchId = CurrentEpochBranchId(nHeight, consensusParams);
}

ZiskTransactionBuilder::ZiskTransactionBuilder(ZiskTransactionBuilder&& builder) :
    consensusParams(std::move(builder.consensusParams)),
    nHeight(std::move(builder.nHeight)),
    keystore(std::move(builder.keystore)),
    coinsView(std::move(builder.coinsView)),
    cs_coinsView(std::move(builder.cs_coinsView)),
    mtx(std::move(builder.mtx)),
    fee(std::move(builder.fee)),
    ziskBundle(std::move(builder.ziskBundle)),
    ziskSpendingKeys(std::move(builder.ziskSpendingKeys)),
    ziskChangeAddr(std::move(builder.ziskChangeAddr))
{
}

ZiskTransactionBuilder& ZiskTransactionBuilder::operator=(ZiskTransactionBuilder&& builder)
{
    if (this != &builder) {
        consensusParams = std::move(builder.consensusParams);
        nHeight = std::move(builder.nHeight);
        keystore = std::move(builder.keystore);
        coinsView = std::move(builder.coinsView);
        cs_coinsView = std::move(builder.cs_coinsView);
        mtx = std::move(builder.mtx);
        fee = std::move(builder.fee);
        ziskBundle = std::move(builder.ziskBundle);
        ziskSpendingKeys = std::move(builder.ziskSpendingKeys);
        ziskChangeAddr = std::move(builder.ziskChangeAddr);
    }
    return *this;
}

void ZiskTransactionBuilder::SetExpiryHeight(uint32_t nExpiryHeight)
{
    mtx.nExpiryHeight = nExpiryHeight;
}

void ZiskTransactionBuilder::SetFee(CAmount fee)
{
    this->fee = fee;
}

bool ZiskTransactionBuilder::AddZiskSpend(const ZiskSpendInfo& spendInfo)
{
    ZiskSpendDescription spendDesc;
    
    // Generate the ZisK proof for this spend
    if (!GenerateZiskSpendProof(spendInfo, spendDesc)) {
        return false;
    }
    
    // Add to bundle
    ziskBundle.spends.push_back(spendDesc);
    
    // Update value balance
    ziskBundle.value_balance += spendInfo.GetNoteValue();
    
    return true;
}

void ZiskTransactionBuilder::AddZiskOutput(
    const uint256& recipientAddress,
    CAmount value,
    const std::optional<std::vector<unsigned char>>& memo
) {
    ZiskOutputDescription outputDesc;
    
    // Generate the ZisK proof for this output
    if (GenerateZiskOutputProof(recipientAddress, value, memo, outputDesc)) {
        ziskBundle.outputs.push_back(outputDesc);
        
        // Update value balance
        ziskBundle.value_balance -= value;
    }
}

void ZiskTransactionBuilder::SendChangeToZisk(const uint256& changeAddr)
{
    ziskChangeAddr = changeAddr;
}

CTransaction ZiskTransactionBuilder::Build()
{
    // Set the ZisK bundle in the transaction
    mtx.ziskBundle = ziskBundle;
    
    // Create the transaction
    return CTransaction(mtx);
}

bool ZiskTransactionBuilder::GenerateZiskSpendProof(
    const ZiskSpendInfo& spendInfo,
    ZiskSpendDescription& spendDesc
) {
    // This would interface with the actual ZisK prover
    // For now, this is a placeholder implementation
    // In a real implementation, this would:
    // 1. Prepare the private inputs (note value, randomness, auth path, spending key)
    // 2. Prepare the public inputs (nullifier, commitment root, value commitment)
    // 3. Call the ZisK prover to generate the STARK proof
    // 4. Set the proof data in the spend description
    
    // Placeholder implementation
    spendDesc.nullifier = uint256(); // Would be computed from spending key and randomness
    spendDesc.commitment_root = spendInfo.GetCommitmentRoot();
    spendDesc.value_commitment = uint256(); // Would be computed from note value
    spendDesc.zisk_proof = std::vector<unsigned char>(); // Would contain the STARK proof
    spendDesc.public_inputs = std::vector<unsigned char>(); // Would contain the public inputs
    
    return true;
}

bool ZiskTransactionBuilder::GenerateZiskOutputProof(
    const uint256& recipientAddress,
    CAmount value,
    const std::optional<std::vector<unsigned char>>& memo,
    ZiskOutputDescription& outputDesc
) {
    // This would interface with the actual ZisK prover
    // For now, this is a placeholder implementation
    // In a real implementation, this would:
    // 1. Prepare the private inputs (note value, randomness, recipient address, memo)
    // 2. Prepare the public inputs (note commitment, value commitment, output encryption key)
    // 3. Call the ZisK prover to generate the STARK proof
    // 4. Set the proof data in the output description
    
    // Placeholder implementation
    outputDesc.note_commitment = uint256(); // Would be computed from note value and randomness
    outputDesc.value_commitment = uint256(); // Would be computed from note value
    outputDesc.output_encryption_key = uint256(); // Would be computed from recipient address
    outputDesc.zisk_proof = std::vector<unsigned char>(); // Would contain the STARK proof
    outputDesc.public_inputs = std::vector<unsigned char>(); // Would contain the public inputs
    
    return true;
}

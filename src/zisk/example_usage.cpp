// Copyright (c) 2024 The ZisK-Zcash developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or https://www.opensource.org/licenses/mit-license.php .

// Example usage of ZisK-Zcash integration

#include "zisk/transaction_builder.h"
#include "zisk/proving_service.h"
#include "zisk/proof_verifier.h"
#include "chainparams.h"
#include "coins.h"
#include "keystore.h"
#include "main.h"

#include <iostream>
#include <memory>

void ExampleZiskTransaction()
{
    std::cout << "=== ZisK-Zcash Integration Example ===" << std::endl;

    // 1. Initialize ZisK proving service
    ZiskProvingService provingService("http://localhost:8080");
    
    // Load circuit parameters
    provingService.LoadCircuitParams("shielded_spend", "circuits/spend_circuit/params.bin");
    provingService.LoadCircuitParams("shielded_output", "circuits/output_circuit/params.bin");

    // 2. Create ZisK transaction builder
    const CChainParams& chainParams = Params();
    int nHeight = 1000000; // Example height
    CKeyStore keystore;
    CCoinsViewDummy dummy;
    CCoinsViewCache coinsView(&dummy);
    CCriticalSection cs_coinsView;
    
    ZiskTransactionBuilder builder(chainParams, nHeight, &keystore, &coinsView, &cs_coinsView);

    // 3. Add ZisK spend
    std::cout << "Adding ZisK spend..." << std::endl;
    
    // Example spend data
    uint64_t noteValue = 1000000; // 0.01 ZEC in zatoshis
    uint256 noteRandomness = GetRandHash();
    uint256 spendingKey = GetRandHash();
    std::vector<uint256> authPath = {GetRandHash(), GetRandHash(), GetRandHash()};
    uint256 commitmentRoot = GetRandHash();
    
    ZiskSpendInfo spendInfo(noteValue, noteRandomness, spendingKey, authPath, commitmentRoot);
    
    if (!builder.AddZiskSpend(spendInfo)) {
        std::cerr << "Failed to add ZisK spend" << std::endl;
        return;
    }

    // 4. Add ZisK output
    std::cout << "Adding ZisK output..." << std::endl;
    
    uint256 recipientAddress = GetRandHash();
    CAmount outputValue = 900000; // 0.009 ZEC in zatoshis
    std::vector<unsigned char> memo = {'H', 'e', 'l', 'l', 'o', ' ', 'Z', 'i', 's', 'K', '!'};
    
    builder.AddZiskOutput(recipientAddress, outputValue, memo);

    // 5. Set change address
    uint256 changeAddress = GetRandHash();
    builder.SendChangeToZisk(changeAddress);

    // 6. Build transaction
    std::cout << "Building ZisK transaction..." << std::endl;
    
    CTransaction tx = builder.Build();
    
    std::cout << "Transaction built successfully!" << std::endl;
    std::cout << "ZisK spends: " << tx.GetZiskSpendsCount() << std::endl;
    std::cout << "ZisK outputs: " << tx.GetZiskOutputsCount() << std::endl;
    std::cout << "ZisK value balance: " << tx.GetValueBalanceZisk() << std::endl;

    // 7. Verify ZisK proofs
    std::cout << "Verifying ZisK proofs..." << std::endl;
    
    ZiskProofVerifier verifier = ZiskProofVerifier::Strict();
    CValidationState state;
    
    if (Consensus::VerifyZiskProofs(tx, state, verifier, 100)) {
        std::cout << "ZisK proofs verified successfully!" << std::endl;
    } else {
        std::cerr << "ZisK proof verification failed: " << state.GetRejectReason() << std::endl;
    }
}

void ExampleDistributedProving()
{
    std::cout << "\n=== Distributed Proving Example ===" << std::endl;

    // Initialize ZisK proving service with distributed server
    ZiskProvingService provingService("http://zisk-prover-cluster:8080");

    // Prepare batch proof requests
    std::vector<ProveRequest> requests;
    
    // Spend proof request
    ProveRequest spendRequest;
    spendRequest.circuit_type = "shielded_spend";
    spendRequest.private_inputs = {1, 2, 3, 4, 5}; // Example data
    spendRequest.public_inputs = {6, 7, 8, 9, 10}; // Example data
    requests.push_back(spendRequest);
    
    // Output proof request
    ProveRequest outputRequest;
    outputRequest.circuit_type = "shielded_output";
    outputRequest.private_inputs = {11, 12, 13, 14, 15}; // Example data
    outputRequest.public_inputs = {16, 17, 18, 19, 20}; // Example data
    requests.push_back(outputRequest);

    // Generate proofs in parallel
    std::cout << "Generating " << requests.size() << " proofs in parallel..." << std::endl;
    
    std::vector<ProveResponse> responses;
    if (provingService.ProveBatch(requests, responses)) {
        std::cout << "Batch proof generation completed!" << std::endl;
        
        for (size_t i = 0; i < responses.size(); i++) {
            if (responses[i].success) {
                std::cout << "Proof " << i << " generated successfully" << std::endl;
            } else {
                std::cerr << "Proof " << i << " failed: " << responses[i].error_message << std::endl;
            }
        }
    } else {
        std::cerr << "Batch proof generation failed" << std::endl;
    }
}

int main()
{
    try {
        ExampleZiskTransaction();
        ExampleDistributedProving();
        
        std::cout << "\n=== ZisK-Zcash Integration Example Complete ===" << std::endl;
        return 0;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }
}

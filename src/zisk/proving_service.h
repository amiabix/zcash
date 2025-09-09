// Copyright (c) 2024 The ZisK-Zcash developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or https://www.opensource.org/licenses/mit-license.php .

#ifndef ZCASH_ZISK_PROVING_SERVICE_H
#define ZCASH_ZISK_PROVING_SERVICE_H

#include "zisk/primitives/zisk_transaction.h"
#include "zisk/sdk/lib.rs.h"

#include <memory>
#include <string>
#include <vector>

/// ZisK proving service for distributed proof generation
class ZiskProvingService
{
private:
    /// ZisK server configuration
    std::string server_url;
    /// Circuit parameters for different circuit types
    std::map<std::string, CircuitParams> circuit_params;

public:
    ZiskProvingService(const std::string& server_url);
    
    // ZiskProvingService should never be copied
    ZiskProvingService(const ZiskProvingService&) = delete;
    ZiskProvingService& operator=(const ZiskProvingService&) = delete;
    ZiskProvingService(ZiskProvingService&&);
    ZiskProvingService& operator=(ZiskProvingService&&);

    /// Load circuit parameters for a specific circuit type
    bool LoadCircuitParams(const std::string& circuit_type, const std::string& params_path);

    /// Generate a ZisK proof for a spend operation
    bool ProveSpend(
        const std::vector<unsigned char>& private_inputs,
        const std::vector<unsigned char>& public_inputs,
        std::vector<unsigned char>& proof_data
    );

    /// Generate a ZisK proof for an output operation
    bool ProveOutput(
        const std::vector<unsigned char>& private_inputs,
        const std::vector<unsigned char>& public_inputs,
        std::vector<unsigned char>& proof_data
    );

    /// Generate multiple proofs in parallel
    bool ProveBatch(
        const std::vector<ProveRequest>& requests,
        std::vector<ProveResponse>& responses
    );

    /// Verify a ZisK proof
    bool VerifyProof(
        const std::vector<unsigned char>& proof_data,
        const std::vector<unsigned char>& public_inputs,
        const std::string& circuit_type
    );

private:
    /// Send request to ZisK proving server
    bool SendProveRequest(
        const std::string& circuit_type,
        const std::vector<unsigned char>& private_inputs,
        const std::vector<unsigned char>& public_inputs,
        std::vector<unsigned char>& proof_data
    );
};

/// Request for proof generation
struct ProveRequest
{
    std::string circuit_type;
    std::vector<unsigned char> private_inputs;
    std::vector<unsigned char> public_inputs;
};

/// Response from proof generation
struct ProveResponse
{
    bool success;
    std::vector<unsigned char> proof_data;
    std::string error_message;
};

#endif // ZCASH_ZISK_PROVING_SERVICE_H

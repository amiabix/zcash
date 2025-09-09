// Copyright (c) 2024 The ZisK-Zcash developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or https://www.opensource.org/licenses/mit-license.php .

#include "zisk/proof_verifier.h"

#include <zisk/sdk/lib.rs>

ZiskProofVerifier ZiskProofVerifier::Strict() {
    return ZiskProofVerifier(true);
}

ZiskProofVerifier ZiskProofVerifier::Disabled() {
    return ZiskProofVerifier(false);
}

bool ZiskProofVerifier::VerifyZiskSpend(
    const ZiskSpendDescription& spend
) {
    if (!perform_verification) {
        return true;
    }

    // This would interface with the actual ZisK verifier
    // For now, this is a placeholder implementation
    // In a real implementation, this would:
    // 1. Load the circuit parameters for the spend circuit
    // 2. Verify the STARK proof against the public inputs
    // 3. Return the verification result
    
    return true; // Placeholder
}

bool ZiskProofVerifier::VerifyZiskOutput(
    const ZiskOutputDescription& output
) {
    if (!perform_verification) {
        return true;
    }

    // This would interface with the actual ZisK verifier
    // For now, this is a placeholder implementation
    // In a real implementation, this would:
    // 1. Load the circuit parameters for the output circuit
    // 2. Verify the STARK proof against the public inputs
    // 3. Return the verification result
    
    return true; // Placeholder
}

bool ZiskProofVerifier::VerifyZiskBundle(
    const ZiskBundle& bundle
) {
    if (!perform_verification) {
        return true;
    }

    // Verify all spend proofs
    for (const auto& spend : bundle.spends) {
        if (!VerifyZiskSpend(spend)) {
            return false;
        }
    }

    // Verify all output proofs
    for (const auto& output : bundle.outputs) {
        if (!VerifyZiskOutput(output)) {
            return false;
        }
    }

    return true;
}

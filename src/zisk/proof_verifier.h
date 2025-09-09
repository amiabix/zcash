// Copyright (c) 2024 The ZisK-Zcash developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or https://www.opensource.org/licenses/mit-license.php .

#ifndef ZCASH_ZISK_PROOF_VERIFIER_H
#define ZCASH_ZISK_PROOF_VERIFIER_H

#include "zisk/primitives/zisk_transaction.h"
#include "uint256.h"

#include <vector>

class ZiskProofVerifier {
private:
    bool perform_verification;

    ZiskProofVerifier(bool perform_verification) : perform_verification(perform_verification) { }

public:
    // ZiskProofVerifier should never be copied
    ZiskProofVerifier(const ZiskProofVerifier&) = delete;
    ZiskProofVerifier& operator=(const ZiskProofVerifier&) = delete;
    ZiskProofVerifier(ZiskProofVerifier&&);
    ZiskProofVerifier& operator=(ZiskProofVerifier&&);

    // Creates a verification context that strictly verifies
    // all ZisK proofs.
    static ZiskProofVerifier Strict();

    // Creates a verification context that performs no
    // verification, used when avoiding duplicate effort
    // such as during reindexing.
    static ZiskProofVerifier Disabled();

    // Verifies that a ZisK spend proof is correct.
    bool VerifyZiskSpend(
        const ZiskSpendDescription& spend
    );

    // Verifies that a ZisK output proof is correct.
    bool VerifyZiskOutput(
        const ZiskOutputDescription& output
    );

    // Verifies all ZisK proofs in a bundle.
    bool VerifyZiskBundle(
        const ZiskBundle& bundle
    );
};

#endif // ZCASH_ZISK_PROOF_VERIFIER_H

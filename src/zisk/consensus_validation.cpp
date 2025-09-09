// Copyright (c) 2024 The ZisK-Zcash developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or https://www.opensource.org/licenses/mit-license.php .

#include "zisk/consensus_validation.h"

#include "main.h"
#include "coins.h"
#include "consensus/upgrades.h"
#include "primitives/transaction.h"
#include "script/interpreter.h"
#include "zcash/Address.hpp"

namespace Consensus {

bool CheckTxZiskInputs(
    const CTransaction& tx,
    CValidationState& state,
    const CCoinsViewCache& view,
    int dosLevel)
{
    // Check if transaction has ZisK components
    if (tx.GetZiskSpendsCount() == 0) {
        return true;
    }

    const ZiskBundle& ziskBundle = tx.GetZiskBundle();

    // Check nullifiers
    for (const auto& spend : ziskBundle.spends) {
        // Check that nullifier is not already spent
        if (view.GetNullifier(spend.nullifier, SAPLING)) {
            return state.DoS(dosLevel, false, REJECT_DUPLICATE, "zisk-nullifier-already-spent");
        }
    }

    // Check commitment roots
    for (const auto& spend : ziskBundle.spends) {
        // Check that commitment root exists in the commitment tree
        if (!view.HaveAnchor(spend.commitment_root)) {
            return state.DoS(dosLevel, false, REJECT_INVALID, "zisk-invalid-commitment-root");
        }
    }

    return true;
}

bool CheckTxZiskOutputs(
    const CTransaction& tx,
    CValidationState& state,
    int dosLevel)
{
    // Check if transaction has ZisK components
    if (tx.GetZiskOutputsCount() == 0) {
        return true;
    }

    const ZiskBundle& ziskBundle = tx.GetZiskBundle();

    // Check note commitments
    for (const auto& output : ziskBundle.outputs) {
        // Check that note commitment is not null
        if (output.note_commitment.IsNull()) {
            return state.DoS(dosLevel, false, REJECT_INVALID, "zisk-invalid-note-commitment");
        }
    }

    // Check value commitments
    for (const auto& output : ziskBundle.outputs) {
        // Check that value commitment is not null
        if (output.value_commitment.IsNull()) {
            return state.DoS(dosLevel, false, REJECT_INVALID, "zisk-invalid-value-commitment");
        }
    }

    return true;
}

bool VerifyZiskProofs(
    const CTransaction& tx,
    CValidationState& state,
    ZiskProofVerifier& verifier,
    int dosLevel)
{
    // Check if transaction has ZisK components
    if (tx.GetZiskSpendsCount() == 0 && tx.GetZiskOutputsCount() == 0) {
        return true;
    }

    const ZiskBundle& ziskBundle = tx.GetZiskBundle();

    // Verify all ZisK proofs
    if (!verifier.VerifyZiskBundle(ziskBundle)) {
        return state.DoS(dosLevel, false, REJECT_INVALID, "zisk-proof-verification-failed");
    }

    return true;
}

} // namespace Consensus

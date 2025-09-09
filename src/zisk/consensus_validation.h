// Copyright (c) 2024 The ZisK-Zcash developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or https://www.opensource.org/licenses/mit-license.php .

#ifndef ZCASH_ZISK_CONSENSUS_VALIDATION_H
#define ZCASH_ZISK_CONSENSUS_VALIDATION_H

#include "primitives/transaction.h"
#include "zisk/primitives/zisk_transaction.h"
#include "zisk/proof_verifier.h"

#include <vector>

namespace Consensus {

/**
 * Check whether all ZisK inputs of this transaction are valid.
 *
 * This checks that:
 * - The nullifiers in the transaction do not exist in the given view.
 * - The commitment roots in the transaction exist in the given view.
 *
 * This does not modify the view to add the nullifiers to the spent set.
 * This does not check proofs or signatures.
 */
bool CheckTxZiskInputs(
    const CTransaction& tx,
    CValidationState& state,
    const CCoinsViewCache& view,
    int dosLevel);

/**
 * Check whether all ZisK outputs of this transaction are valid.
 *
 * This checks that:
 * - The note commitments are properly formatted.
 * - The value commitments are properly formatted.
 *
 * This does not check proofs or signatures.
 */
bool CheckTxZiskOutputs(
    const CTransaction& tx,
    CValidationState& state,
    int dosLevel);

/**
 * Verify all ZisK proofs in a transaction.
 *
 * This checks that:
 * - All ZisK spend proofs are valid.
 * - All ZisK output proofs are valid.
 */
bool VerifyZiskProofs(
    const CTransaction& tx,
    CValidationState& state,
    ZiskProofVerifier& verifier,
    int dosLevel);

} // namespace Consensus

#endif // ZCASH_ZISK_CONSENSUS_VALIDATION_H

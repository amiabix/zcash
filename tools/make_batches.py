#!/usr/bin/env python3
"""
ZisK Zcash Validator - Batch Generator
Generates mixed batches (transparent + shielded) with proper Zcash transaction format
"""

import os
import sys
import struct
import hashlib
import random
from typing import List, Tuple, Dict

# --------- Configuration ----------
OUT_DIR = "batches"
BATCH_SIZE = 1000              # number of txs per batch
NUM_BATCHES = 3                # how many batches to generate
TRANSPARENT_RATIO = 0.6        # fraction of txs that are transparent
INITIAL_UTXOS = 2000           # initial UTXOs to build tree from
# ---------------------------------

def sha256(b: bytes) -> bytes:
    """SHA256 hash"""
    return hashlib.sha256(b).digest()

def double_sha(b: bytes) -> bytes:
    """Double SHA256 hash (Bitcoin/Zcash style)"""
    return sha256(sha256(b))

def leaf_bytes(prev_txid: bytes, prev_index: int, value: int, script_hash: bytes) -> bytes:
    """Canonical UTXO leaf serialization matching validator"""
    return prev_txid + struct.pack("<I", prev_index) + struct.pack("<Q", value) + script_hash

def build_merkle_tree(leaves: List[bytes]) -> Tuple[bytes, List[List[bytes]]]:
    """
    Build binary Merkle tree and return root + proofs for each leaf.
    Matches the verify_merkle_branch function in the validator.
    """
    if not leaves:
        return double_sha(b""), []
    
    # Compute leaf hashes
    level = [double_sha(leaf) for leaf in leaves]
    proofs = [[] for _ in range(len(leaves))]
    
    # Build tree level by level
    current_level = level[:]
    indices = list(range(len(leaves)))
    
    while len(current_level) > 1:
        next_level = []
        next_indices = []
        
        # Process pairs
        for i in range(0, len(current_level), 2):
            left = current_level[i]
            right = current_level[i + 1] if i + 1 < len(current_level) else current_level[i]
            
            # Compute parent hash
            parent = double_sha(left + right)
            next_level.append(parent)
            next_indices.append(i // 2)
            
            # Record sibling proofs
            left_idx = indices[i]
            right_idx = indices[i + 1] if i + 1 < len(indices) else indices[i]
            
            # Add sibling to proof for left child
            if left_idx < len(proofs):
                proofs[left_idx].append(right)
            # Add sibling to proof for right child  
            if right_idx < len(proofs):
                proofs[right_idx].append(left)
        
        current_level = next_level
        indices = next_indices
    
    root = current_level[0] if current_level else double_sha(b"")
    return root, proofs

def random_txid() -> bytes:
    """Generate random transaction ID"""
    return os.urandom(32)

def make_initial_utxos(n: int) -> List[Dict]:
    """Create initial UTXO set"""
    utxos = []
    for i in range(n):
        prev_txid = random_txid()
        prev_index = random.randint(0, 1000)
        value = random.randint(100000, 1000000)  # 0.001 to 0.01 ZEC in zatoshis
        script = b'\x76\xa9' + os.urandom(20) + b'\x88\xac'  # P2PKH-like script
        script_hash = sha256(script)
        
        utxos.append({
            "prev_txid": prev_txid,
            "prev_index": prev_index,
            "value": value,
            "script": script,
            "script_hash": script_hash
        })
    return utxos

def create_zcash_transaction(utxos: List[Dict], outputs: List[Dict]) -> bytes:
    """
    Create a proper Zcash transaction in the format expected by the validator.
    Matches the parse_zcash_transaction function structure.
    """
    data = bytearray()
    
    # Version (4 bytes) - Zcash v4
    data.extend(struct.pack('<I', 4))
    
    # Version group ID (4 bytes) - 0x892F2085 for Zcash
    data.extend(struct.pack('<I', 0x892F2085))
    
    # Lock time (4 bytes)
    data.extend(struct.pack('<I', 0))
    
    # Expiry height (4 bytes)
    data.extend(struct.pack('<I', 0))
    
    # Input count (1 byte)
    data.append(len(utxos))
    
    # Parse each transparent input
    for utxo in utxos:
        # Previous output hash (32 bytes)
        data.extend(utxo["prev_txid"])
        
        # Previous output index (4 bytes)
        data.extend(struct.pack('<I', utxo["prev_index"]))
        
        # ScriptSig length (1 byte)
        script_sig = b'\x30' + os.urandom(64)  # Fake signature
        data.append(len(script_sig))
        
        # ScriptSig (variable length)
        data.extend(script_sig)
        
        # Sequence (4 bytes)
        data.extend(struct.pack('<I', 0xFFFFFFFF))
    
    # Output count (1 byte)
    data.append(len(outputs))
    
    # Parse each transparent output
    for output in outputs:
        # Value (8 bytes)
        data.extend(struct.pack('<Q', output["value"]))
        
        # ScriptPubKey length (1 byte)
        data.append(len(output["script"]))
        
        # ScriptPubKey (variable length)
        data.extend(output["script"])
    
    # Sapling bundle (simplified - just counts)
    data.append(0)  # Sapling input count
    data.append(0)  # Sapling output count
    
    # Orchard bundle (simplified - just counts)
    data.append(0)  # Orchard action count
    
    return bytes(data)

def create_shielded_transaction() -> Tuple[bytes, Dict]:
    """Create a structural shielded transaction with placeholder proof"""
    # Create structural Sapling spend data
    cmu = os.urandom(32)
    cv = os.urandom(32)
    ephemeral_key = os.urandom(32)
    nullifier = os.urandom(32)
    anchor = os.urandom(32)
    
    # Create a minimal Zcash transaction with Sapling bundle
    data = bytearray()
    
    # Version (4 bytes) - Zcash v4
    data.extend(struct.pack('<I', 4))
    
    # Version group ID (4 bytes) - 0x892F2085 for Zcash
    data.extend(struct.pack('<I', 0x892F2085))
    
    # Lock time (4 bytes)
    data.extend(struct.pack('<I', 0))
    
    # Expiry height (4 bytes)
    data.extend(struct.pack('<I', 0))
    
    # Input count (1 byte) - no transparent inputs
    data.append(0)
    
    # Output count (1 byte) - no transparent outputs
    data.append(0)
    
    # Sapling bundle
    data.append(1)  # Sapling input count
    data.append(1)  # Sapling output count
    
    # Sapling spend (simplified structure)
    data.extend(nullifier)  # nullifier
    data.extend(cv)         # value commitment
    data.extend(anchor)     # anchor
    
    # Sapling output (simplified structure)
    data.extend(cmu)        # note commitment
    data.extend(cv)         # value commitment
    data.extend(ephemeral_key)  # ephemeral key
    
    # Placeholder Sapling proof (96 bytes)
    data.extend(b'\x00' * 96)
    
    # Orchard bundle (simplified - just counts)
    data.append(0)  # Orchard action count
    
    meta = {
        "cmu": cmu,
        "nullifier": nullifier,
        "value": 50000  # 0.0005 ZEC
    }
    
    return bytes(data), meta

def write_batch_file(batch_index: int, prior_root: bytes, tx_list: List[bytes], 
                    utxo_entries: List[Dict], proofs: List[List[bytes]], outpath: str):
    """
    Write batch file in the streaming format expected by the validator.
    Matches the read_transaction_data function structure.
    """
    tx_batch = b"".join(tx_list)
    tx_batch_hash = sha256(tx_batch)
    
    with open(outpath, "wb") as f:
        # Prior state root (32 bytes)
        f.write(prior_root)
        
        # Transaction batch hash (32 bytes)
        f.write(tx_batch_hash)
        
        # UTXO count (4 bytes)
        f.write(struct.pack("<I", len(utxo_entries)))
        
        # UTXO entries
        for i, utxo in enumerate(utxo_entries):
            # Build UTXO entry bytes
            entry = bytearray()
            entry.extend(utxo["prev_txid"])
            entry.extend(struct.pack("<I", utxo["prev_index"]))
            entry.extend(struct.pack("<Q", utxo["value"]))
            entry.extend(struct.pack("<I", len(utxo["script"])))
            entry.extend(utxo["script"])
            
            # Merkle path
            proof = proofs[i] if i < len(proofs) else []
            entry.extend(struct.pack("<I", len(proof)))
            for ph in proof:
                entry.extend(ph)
            
            # Leaf index
            entry.extend(struct.pack("<Q", i))
            
            # Write UTXO entry length and data
            f.write(struct.pack("<I", len(entry)))
            f.write(entry)
        
        # Transaction batch length (4 bytes)
        f.write(struct.pack("<I", len(tx_batch)))
        
        # Transaction batch bytes
        f.write(tx_batch)
        
        # Aux count (4 bytes) - 0 for now
        f.write(struct.pack("<I", 0))

def main():
    """Generate test batches"""
    print("🔧 ZisK Zcash Validator - Batch Generator")
    print("=" * 50)
    print(f"📊 Configuration:")
    print(f"   - Batch size: {BATCH_SIZE} transactions")
    print(f"   - Number of batches: {NUM_BATCHES}")
    print(f"   - Transparent ratio: {TRANSPARENT_RATIO:.1%}")
    print(f"   - Initial UTXOs: {INITIAL_UTXOS}")
    print()
    
    # Create output directory
    os.makedirs(OUT_DIR, exist_ok=True)
    
    # Create initial UTXO universe
    print("🏗️  Creating initial UTXO set...")
    utxos = make_initial_utxos(INITIAL_UTXOS)
    
    # Build Merkle tree from UTXOs
    print("🌳 Building Merkle tree...")
    leaves = [leaf_bytes(u["prev_txid"], u["prev_index"], u["value"], u["script_hash"]) for u in utxos]
    root, proofs = build_merkle_tree(leaves)
    prior_root = root
    
    print(f"✅ Initial state root: {root.hex()}")
    print(f"✅ Merkle tree built with {len(leaves)} leaves")
    print()
    
    utxo_index = 0
    total_transactions = 0
    total_utxos_used = 0
    
    for batch_num in range(NUM_BATCHES):
        print(f"📦 Generating batch {batch_num}...")
        
        tx_list = []
        used_utxos = []
        used_proofs = []
        
        # Generate transactions for this batch
        for tx_num in range(BATCH_SIZE):
            if random.random() < TRANSPARENT_RATIO and utxo_index < len(utxos):
                # Transparent transaction
                utxo = utxos[utxo_index]
                
                # Create output (simplified - just one output)
                output_value = utxo["value"] - 1000  # Small fee
                output_script = b'\x76\xa9' + os.urandom(20) + b'\x88\xac'
                outputs = [{"value": output_value, "script": output_script}]
                
                # Create Zcash transaction
                tx_bytes = create_zcash_transaction([utxo], outputs)
                tx_list.append(tx_bytes)
                used_utxos.append(utxo)
                used_proofs.append(proofs[utxo_index])
                utxo_index += 1
            else:
                # Shielded transaction
                tx_bytes, meta = create_shielded_transaction()
                tx_list.append(tx_bytes)
                # Shielded transactions don't consume transparent UTXOs
        
        # Write batch file
        outpath = os.path.join(OUT_DIR, f"batch_{batch_num}.bin")
        write_batch_file(batch_num, prior_root, tx_list, used_utxos, used_proofs, outpath)
        
        batch_size = len(tx_list)
        utxos_used = len(used_utxos)
        total_transactions += batch_size
        total_utxos_used += utxos_used
        
        print(f"   ✅ Batch {batch_num}: {batch_size} transactions, {utxos_used} UTXOs used")
        print(f"   📁 Written to: {outpath}")
        
        # Update prior root for next batch (simplified chaining)
        prior_root = sha256(prior_root + struct.pack("<I", batch_num + 1))
    
    print()
    print("🎉 Batch generation complete!")
    print(f"📊 Summary:")
    print(f"   - Total transactions: {total_transactions}")
    print(f"   - Total UTXOs used: {total_utxos_used}")
    print(f"   - Batches generated: {NUM_BATCHES}")
    print(f"   - Output directory: {OUT_DIR}/")
    print()
    print("🚀 Ready for ZisK validation testing!")

if __name__ == "__main__":
    main()
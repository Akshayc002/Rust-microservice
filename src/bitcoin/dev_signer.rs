use bitcoin::{
    psbt::Psbt,
    secp256k1::{Secp256k1, Message},
    EcdsaSighashType,
    sighash::SighashCache,
};
use bitcoin::hashes::Hash;
use crate::bitcoin::dev_keys::DevRole;

/// ⚠️ REAL SIGNING (DEV ONLY)
/// Signs the PSBT correctly so it can be broadcast on Regtest
pub fn sign_psbt_dev(
    mut psbt: Psbt,
    role: DevRole,
) -> Psbt {
    let secp = Secp256k1::new();
    let (sk, pk) = crate::bitcoin::dev_keys::dev_keypair(role);

    // Extract the unsigned transaction to sign
    let unsigned_tx = psbt.unsigned_tx.clone();
    let mut sighasher = SighashCache::new(&unsigned_tx);

    for (i, input) in psbt.inputs.iter_mut().enumerate() {
        // We need the witness script (redeem script for P2WSH)
        if let Some(witness_script) = &input.witness_script {
            // Calculate Sighash
            let sighash = sighasher.p2wsh_signature_hash(
                i,
                witness_script,
                input.witness_utxo.as_ref().unwrap().value,
                EcdsaSighashType::All,
            ).expect("failed to create sighash");

            // Sign
            let msg = Message::from_digest(sighash.to_byte_array());
            let sig = secp.sign_ecdsa(&msg, &sk);

            // Insert Partial Signature
            input.partial_sigs.insert(
                pk,
                bitcoin::ecdsa::Signature {
                    sig,
                    hash_ty: EcdsaSighashType::All,
                },
            );
        }
    }

    psbt
}

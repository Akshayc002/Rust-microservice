use bitcoin::psbt::Psbt;
use bitcoin::{PublicKey, Transaction};
use base64::Engine;
use std::collections::HashSet;

#[derive(Debug)]
pub enum PsbtVerificationError {
    InvalidBase64,
    InvalidPsbt,
    NotEnoughSignatures,
    TooManySignatures,
    UnknownSigner,
    #[allow(dead_code)]
    InvalidOutput,
}

pub fn verify_2of3_psbt(
    psbt_base64: &str,
    allowed_pubkeys: &[PublicKey],
) -> Result<(), PsbtVerificationError> {

    // Decode Base64
    let psbt_bytes = base64::engine::general_purpose::STANDARD.decode(psbt_base64)
        .map_err(|_| PsbtVerificationError::InvalidBase64)?;

    // Parse PSBT
    let psbt: Psbt = Psbt::deserialize(&psbt_bytes)
        .map_err(|_| PsbtVerificationError::InvalidPsbt)?;

    let mut unique_signers = HashSet::new();

    // Iterate over inputs and partial signatures
    for input in &psbt.inputs {
        for pubkey in input.partial_sigs.keys() {
            // Signer must be one of the allowed 3
            if !allowed_pubkeys.contains(pubkey) {
                return Err(PsbtVerificationError::UnknownSigner);
            }

            unique_signers.insert(pubkey.to_string());
        }
    }

    match unique_signers.len() {
        2 => Ok(()),
        0 | 1 => Err(PsbtVerificationError::NotEnoughSignatures),
        _ => Err(PsbtVerificationError::TooManySignatures),
    }
}

pub fn extract_tx_from_psbt(psbt_base64: &str) -> Result<Transaction, PsbtVerificationError> {
    let psbt_bytes = base64::engine::general_purpose::STANDARD.decode(psbt_base64)
        .map_err(|_| PsbtVerificationError::InvalidBase64)?;

    let psbt: Psbt = Psbt::deserialize(&psbt_bytes)
        .map_err(|_| PsbtVerificationError::InvalidPsbt)?;

    psbt.extract_tx().map_err(|_| PsbtVerificationError::InvalidPsbt)
}

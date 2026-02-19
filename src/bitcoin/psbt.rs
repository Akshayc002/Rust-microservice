use bitcoin::{Transaction, psbt::Psbt};
use base64::Engine;

#[derive(Debug)]
pub enum PsbtCreateError {
    InvalidHex,
    InvalidTransaction,
    MergeError,
}

pub fn create_psbt_from_hex(tx_hex: &str) -> Result<Psbt, PsbtCreateError> {
    let tx_bytes = hex::decode(tx_hex)
        .map_err(|_| PsbtCreateError::InvalidHex)?;

    let tx: Transaction = bitcoin::consensus::deserialize(&tx_bytes)
        .map_err(|_| PsbtCreateError::InvalidTransaction)?;

    Psbt::from_unsigned_tx(tx)
        .map_err(|_| PsbtCreateError::InvalidTransaction)
}

pub fn add_signature_to_psbt(original_psbt: &mut Psbt, signed_psbt_base64: &str) -> Result<(), PsbtCreateError> {
    let signed_bytes = base64::engine::general_purpose::STANDARD.decode(signed_psbt_base64)
        .map_err(|_| PsbtCreateError::InvalidHex)?;

    let signed_psbt: Psbt = Psbt::deserialize(&signed_bytes)
        .map_err(|_| PsbtCreateError::InvalidTransaction)?;

    original_psbt.combine(signed_psbt)
        .map_err(|_| PsbtCreateError::MergeError)
}

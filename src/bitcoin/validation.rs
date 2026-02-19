use bitcoin::{Transaction, Address, Amount, PublicKey};
use bitcoin::psbt::Psbt;
use base64::Engine;
use std::str::FromStr;
use crate::bitcoin::psbt_verify::{verify_2of3_psbt, extract_tx_from_psbt};

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidPublicKey,
    #[allow(dead_code)]
    InvalidAddress,
    InvalidOutput,
    FeeTooHigh,
    #[allow(dead_code)]
    ScriptMismatch,
    InvalidPsbt,
    InvalidSignature,
    MissingInputInfo,
}

#[derive(Debug, PartialEq)]
pub enum FundingStatus {
    Pending,
    Confirmed,
    Underfunded,
    WrongAddress,
}

pub fn validate_pubkey(pubkey_hex: &str) -> Result<PublicKey, ValidationError> {
    PublicKey::from_str(pubkey_hex).map_err(|_| ValidationError::InvalidPublicKey)
}

#[allow(dead_code)]
pub fn validate_funding(
    tx: &Transaction,
    escrow_address: &Address,
    min_amount_sat: u64
) -> bool {
    tx.output.iter().any(|o| {
        Address::from_script(
            &o.script_pubkey,
            *escrow_address.network()
        )
        .map(|addr| {
            addr == *escrow_address &&
            o.value >= Amount::from_sat(min_amount_sat)
        })
        .unwrap_or(false)
    })
}

pub fn check_funding_status(
    tx: &Transaction,
    confirmations: u32,
    escrow_address: &Address,
    expected_sats: u64,
    min_confirmations: u32
) -> FundingStatus {
    let mut total_sent = 0;
    let mut found_address = false;

    for output in &tx.output {
        if let Ok(addr) = Address::from_script(&output.script_pubkey, *escrow_address.network()) {
            if addr == *escrow_address {
                found_address = true;
                total_sent += output.value.to_sat();
            }
        }
    }

    if !found_address {
        return FundingStatus::WrongAddress;
    }

    if total_sent < expected_sats {
        return FundingStatus::Underfunded;
    }

    if confirmations < min_confirmations {
        return FundingStatus::Pending;
    }

    FundingStatus::Confirmed
}

pub fn validate_signed_tx(
    tx: &Transaction,
    expected_output_address: &Address,
    max_fee_sat: u64,
    input_total_sat: u64
) -> Result<(), ValidationError> {
    // 1. Verify output correctness
    let mut found_output = false;
    let mut output_total = 0;

    for output in &tx.output {
        output_total += output.value.to_sat();
        if let Ok(addr) = Address::from_script(&output.script_pubkey, *expected_output_address.network()) {
            if addr == *expected_output_address {
                found_output = true;
            }
        }
    }

    if !found_output {
        return Err(ValidationError::InvalidOutput);
    }

    // 2. Verify Fee
    if input_total_sat < output_total {
        return Err(ValidationError::FeeTooHigh);
    }

    let fee = input_total_sat - output_total;
    if fee > max_fee_sat {
        return Err(ValidationError::FeeTooHigh);
    }

    Ok(())
}

pub fn validate_psbt_and_finalize(
    psbt_base64: &str,
    allowed_pubkeys: &[PublicKey],
    expected_output_address: &Address,
    max_fee_sat: u64
) -> Result<Transaction, ValidationError> {
    // 1. Verify Signatures (2-of-3)
    verify_2of3_psbt(psbt_base64, allowed_pubkeys)
        .map_err(|_| ValidationError::InvalidSignature)?;

    // 2. Decode PSBT to get input amounts
    let psbt_bytes = base64::engine::general_purpose::STANDARD.decode(psbt_base64)
        .map_err(|_| ValidationError::InvalidPsbt)?;
    let psbt: Psbt = Psbt::deserialize(&psbt_bytes)
        .map_err(|_| ValidationError::InvalidPsbt)?;

    let mut input_total_sat = 0;
    for input in &psbt.inputs {
        if let Some(utxo) = &input.witness_utxo {
            input_total_sat += utxo.value.to_sat();
        } else if let Some(_utxo) = &input.non_witness_utxo {
            // For non-witness, we need to find the output index.
            // This is more complex as we need the outpoint index from the unsigned tx.
            // For MVP, we assume witness_utxo is present (SegWit).
            return Err(ValidationError::MissingInputInfo);
        } else {
            return Err(ValidationError::MissingInputInfo);
        }
    }

    // 3. Extract Transaction
    let tx = extract_tx_from_psbt(psbt_base64)
        .map_err(|_| ValidationError::InvalidPsbt)?;

    // 4. Validate Content (Outputs & Fee)
    validate_signed_tx(&tx, expected_output_address, max_fee_sat, input_total_sat)?;

    Ok(tx)
}

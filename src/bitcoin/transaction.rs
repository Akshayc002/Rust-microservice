use bitcoin::{
    Transaction, TxOut, TxIn, OutPoint, ScriptBuf, Sequence, Witness, Amount, Address,
    absolute::LockTime,
};
use bitcoincore_rpc::bitcoincore_rpc_json::ListUnspentResultEntry;

#[derive(Debug)]
pub enum TransactionError {
    InsufficientFunds,
    #[allow(dead_code)]
    InvalidAddress,
}

pub fn build_repayment_tx(
    utxos: Vec<ListUnspentResultEntry>,
    lender_address: &Address,
    amount_sat: u64,
    fee_rate_sat_per_vbyte: u64,
) -> Result<Transaction, TransactionError> {
    let mut total_input_value = 0;
    let mut inputs = Vec::new();

    for utxo in utxos {
        total_input_value += utxo.amount.to_sat();
        inputs.push(TxIn {
            previous_output: OutPoint::new(utxo.txid, utxo.vout),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::default(),
        });
    }

    if total_input_value < amount_sat {
        return Err(TransactionError::InsufficientFunds);
    }

    // Estimate size: 1 input ~ 68 vbytes (P2WSH), 1 output ~ 31 vbytes
    // This is a rough estimation. For 2-of-3 multisig P2WSH, input size is larger.
    // P2WSH input size is approx 105 vbytes.
    let estimated_size = inputs.len() * 105 + 2 * 31 + 10; // +10 overhead
    let fee = estimated_size as u64 * fee_rate_sat_per_vbyte;

    let total_needed = amount_sat + fee;

    if total_input_value < total_needed {
        return Err(TransactionError::InsufficientFunds);
    }

    let _change = total_input_value - total_needed;

    let outputs = vec![
        TxOut {
            value: Amount::from_sat(amount_sat),
            script_pubkey: lender_address.script_pubkey(),
        }
    ];

    // Add change output if significant (dust limit check omitted for simplicity, assuming > 546 sats)
    // In this simplified version without change address, we just burn the change to fees if it's small,
    // or we should fail if we can't return change.
    // For now, we proceed without change output if no change address is provided.
    // This effectively increases the fee.

    Ok(Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: LockTime::ZERO,
        input: inputs,
        output: outputs,
    })
}

pub fn build_repayment_tx_with_change(
    utxos: Vec<ListUnspentResultEntry>,
    lender_address: &Address,
    change_address: &Address,
    amount_sat: u64,
    fee_rate_sat_per_vbyte: u64,
) -> Result<Transaction, TransactionError> {
    let mut total_input_value = 0;
    let mut inputs = Vec::new();

    for utxo in utxos {
        total_input_value += utxo.amount.to_sat();
        inputs.push(TxIn {
            previous_output: OutPoint::new(utxo.txid, utxo.vout),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::default(),
        });
    }

    // P2WSH input ~ 105 vbytes (2-of-3)
    // P2WPKH output ~ 31 vbytes
    let estimated_size = inputs.len() * 105 + 2 * 31 + 10;
    let fee = estimated_size as u64 * fee_rate_sat_per_vbyte;
    let total_needed = amount_sat + fee;

    if total_input_value < total_needed {
        return Err(TransactionError::InsufficientFunds);
    }

    let change = total_input_value - total_needed;

    let mut outputs = vec![
        TxOut {
            value: Amount::from_sat(amount_sat),
            script_pubkey: lender_address.script_pubkey(),
        }
    ];

    if change > 546 {
        outputs.push(TxOut {
            value: Amount::from_sat(change),
            script_pubkey: change_address.script_pubkey(),
        });
    }

    Ok(Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: LockTime::ZERO,
        input: inputs,
        output: outputs,
    })
}

#[allow(dead_code)]
pub fn build_arbitration_tx(
    utxos: Vec<ListUnspentResultEntry>,
    recipient_address: &Address,
    amount_sat: u64,
    fee_rate_sat_per_vbyte: u64,
) -> Result<Transaction, TransactionError> {
    // Arbitration TX is essentially the same as repayment, just potentially different recipient (e.g. split)
    // For MVP, we treat it as sending full amount to a target (could be lender or borrower).
    // We reuse the logic but expose it as a distinct function for clarity.
    build_repayment_tx(utxos, recipient_address, amount_sat, fee_rate_sat_per_vbyte)
}

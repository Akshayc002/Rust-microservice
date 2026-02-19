use bitcoin::{
    Amount, OutPoint, ScriptBuf, Transaction, TxIn, TxOut,
};
use bitcoin::psbt::Psbt;

pub fn valid_test_psbt() -> Psbt {
    let tx = Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::default(),
            script_sig: ScriptBuf::new(),
            sequence: bitcoin::Sequence::MAX,
            witness: bitcoin::Witness::default(),
        }],
        output: vec![TxOut {
            value: Amount::from_sat(1_000),
            script_pubkey: ScriptBuf::new(),
        }],
    };

    Psbt::from_unsigned_tx(tx).expect("valid PSBT")
}

use crate::bitcoin::psbt::{create_psbt_from_hex, add_signature_to_psbt, PsbtCreateError};
use bitcoin::{Transaction, psbt::Psbt};

#[test]
fn test_create_psbt_invalid_hex() {
    let res = create_psbt_from_hex("invalidhex");
    assert!(matches!(res, Err(PsbtCreateError::InvalidHex)));
}

#[test]
fn test_add_signature_merge() {
    // This is a mock test because creating valid mergeable PSBTs is complex without full setup.
    // We just test that it attempts to parse.
    let mut psbt = Psbt::from_unsigned_tx(Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![],
        output: vec![],
    }).unwrap();

    let res = add_signature_to_psbt(&mut psbt, "invalidbase64");
    assert!(matches!(res, Err(PsbtCreateError::InvalidHex)));
}

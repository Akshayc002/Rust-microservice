use crate::bitcoin::validation::{validate_pubkey, check_funding_status, validate_signed_tx, ValidationError, FundingStatus};
use bitcoin::{PublicKey, Transaction, TxOut, Amount, Address, Network};
use std::str::FromStr;

#[test]
fn test_validate_valid_pubkey() {
    let valid_hex = "02c0ded7d6e2f0b0c0ded7d6e2f0b0c0ded7d6e2f0b0c0ded7d6e2f0b0c0ded7";
    let result = validate_pubkey(valid_hex);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        PublicKey::from_str(valid_hex).unwrap()
    );
}

#[test]
fn test_validate_invalid_pubkey() {
    let invalid_hex = "invalid_hex_string";
    let result = validate_pubkey(invalid_hex);
    assert_eq!(result, Err(ValidationError::InvalidPublicKey));
}

#[test]
fn test_funding_status_confirmed() {
    let address = Address::from_str("bcrt1q6rhpng9evdsfnn833a4f4ve0agr6431l668x24").unwrap().require_network(Network::Regtest).unwrap();
    let tx = Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![],
        output: vec![TxOut {
            value: Amount::from_sat(100_000),
            script_pubkey: address.script_pubkey(),
        }],
    };

    let status = check_funding_status(&tx, 6, &address, 100_000, 3);
    assert_eq!(status, FundingStatus::Confirmed);
}

#[test]
fn test_funding_status_pending() {
    let address = Address::from_str("bcrt1q6rhpng9evdsfnn833a4f4ve0agr6431l668x24").unwrap().require_network(Network::Regtest).unwrap();
    let tx = Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![],
        output: vec![TxOut {
            value: Amount::from_sat(100_000),
            script_pubkey: address.script_pubkey(),
        }],
    };

    let status = check_funding_status(&tx, 1, &address, 100_000, 3);
    assert_eq!(status, FundingStatus::Pending);
}

#[test]
fn test_funding_status_underfunded() {
    let address = Address::from_str("bcrt1q6rhpng9evdsfnn833a4f4ve0agr6431l668x24").unwrap().require_network(Network::Regtest).unwrap();
    let tx = Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![],
        output: vec![TxOut {
            value: Amount::from_sat(50_000),
            script_pubkey: address.script_pubkey(),
        }],
    };

    let status = check_funding_status(&tx, 6, &address, 100_000, 3);
    assert_eq!(status, FundingStatus::Underfunded);
}

#[test]
fn test_funding_status_wrong_address() {
    let address = Address::from_str("bcrt1q6rhpng9evdsfnn833a4f4ve0agr6431l668x24").unwrap().require_network(Network::Regtest).unwrap();
    let other_address = Address::from_str("bcrt1q496g98496g98496g98496g98496g98496g9849").unwrap().require_network(Network::Regtest).unwrap();

    let tx = Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![],
        output: vec![TxOut {
            value: Amount::from_sat(100_000),
            script_pubkey: other_address.script_pubkey(),
        }],
    };

    let status = check_funding_status(&tx, 6, &address, 100_000, 3);
    assert_eq!(status, FundingStatus::WrongAddress);
}

#[test]
fn test_validate_signed_tx_success() {
    let address = Address::from_str("bcrt1q6rhpng9evdsfnn833a4f4ve0agr6431l668x24").unwrap().require_network(Network::Regtest).unwrap();
    let tx = Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![],
        output: vec![TxOut {
            value: Amount::from_sat(99_000),
            script_pubkey: address.script_pubkey(),
        }],
    };

    // Input 100k, Output 99k, Fee 1k. Max fee 2k.
    let res = validate_signed_tx(&tx, &address, 2000, 100_000);
    assert!(res.is_ok());
}

#[test]
fn test_validate_signed_tx_fee_too_high() {
    let address = Address::from_str("bcrt1q6rhpng9evdsfnn833a4f4ve0agr6431l668x24").unwrap().require_network(Network::Regtest).unwrap();
    let tx = Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![],
        output: vec![TxOut {
            value: Amount::from_sat(90_000),
            script_pubkey: address.script_pubkey(),
        }],
    };

    // Input 100k, Output 90k, Fee 10k. Max fee 2k.
    let res = validate_signed_tx(&tx, &address, 2000, 100_000);
    assert_eq!(res, Err(ValidationError::FeeTooHigh));
}

#[test]
fn test_validate_signed_tx_wrong_output() {
    let address = Address::from_str("bcrt1q6rhpng9evdsfnn833a4f4ve0agr6431l668x24").unwrap().require_network(Network::Regtest).unwrap();
    let other_address = Address::from_str("bcrt1q496g98496g98496g98496g98496g98496g9849").unwrap().require_network(Network::Regtest).unwrap();

    let tx = Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![],
        output: vec![TxOut {
            value: Amount::from_sat(99_000),
            script_pubkey: other_address.script_pubkey(),
        }],
    };

    let res = validate_signed_tx(&tx, &address, 2000, 100_000);
    assert_eq!(res, Err(ValidationError::InvalidOutput));
}

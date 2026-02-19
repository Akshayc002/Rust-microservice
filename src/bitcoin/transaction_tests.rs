use crate::bitcoin::transaction::{build_repayment_tx_with_change, TransactionError};
use bitcoin::{Address, Amount, Network, Txid};
use bitcoin::hashes::Hash;
use bitcoincore_rpc::bitcoincore_rpc_json::ListUnspentResultEntry;
use std::str::FromStr;

#[test]
fn test_build_repayment_tx_success() {
    let lender_addr = Address::from_str("bcrt1q6rhpng9evdsfnn833a4f4ve0agr6431l668x24").unwrap().require_network(Network::Regtest).unwrap();
    let change_addr = Address::from_str("bcrt1q496g98496g98496g98496g98496g98496g9849").unwrap().require_network(Network::Regtest).unwrap();

    let utxo = ListUnspentResultEntry {
        txid: Txid::all_zeros(),
        vout: 0,
        address: None,
        label: None,
        redeem_script: None,
        witness_script: None,
        script_pub_key: Default::default(),
        amount: Amount::from_sat(100_000),
        confirmations: 1,
        spendable: true,
        solvable: true,
        descriptor: None,
        safe: true,
    };

    let tx = build_repayment_tx_with_change(
        vec![utxo],
        &lender_addr,
        &change_addr,
        50_000,
        1
    ).unwrap();

    assert_eq!(tx.output.len(), 2);
    assert_eq!(tx.output[0].value.to_sat(), 50_000);
    // Fee calculation: 1 input (105) + 2 outputs (62) + 10 = 177 vbytes approx
    // Fee = 177 sats. Change = 100000 - 50000 - 177 = 49823
    assert!(tx.output[1].value.to_sat() > 49000);
}

#[test]
fn test_build_repayment_tx_insufficient_funds() {
    let lender_addr = Address::from_str("bcrt1q6rhpng9evdsfnn833a4f4ve0agr6431l668x24").unwrap().require_network(Network::Regtest).unwrap();
    let change_addr = Address::from_str("bcrt1q496g98496g98496g98496g98496g98496g9849").unwrap().require_network(Network::Regtest).unwrap();

    let utxo = ListUnspentResultEntry {
        txid: Txid::all_zeros(),
        vout: 0,
        address: None,
        label: None,
        redeem_script: None,
        witness_script: None,
        script_pub_key: Default::default(),
        amount: Amount::from_sat(10_000),
        confirmations: 1,
        spendable: true,
        solvable: true,
        descriptor: None,
        safe: true,
    };

    let result = build_repayment_tx_with_change(
        vec![utxo],
        &lender_addr,
        &change_addr,
        50_000,
        1
    );

    assert!(matches!(result, Err(TransactionError::InsufficientFunds)));
}

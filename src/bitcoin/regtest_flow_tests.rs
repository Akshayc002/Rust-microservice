#[cfg(test)]
mod tests {
    use bitcoin::{
        Amount, Network,
        psbt::Psbt,
    };
    use base64::Engine;
    use bitcoincore_rpc::{Client, Auth, RpcApi};
    use crate::bitcoin::{
        multisig::create_2of3_multisig,
        dev_keys::{dev_keypair, DevRole},
        dev_signer::sign_psbt_dev,
        transaction::build_repayment_tx,
        validation::{validate_psbt_and_finalize, check_funding_status, FundingStatus},
        rpc::{get_utxos, broadcast_tx},
    };
    use serde_json::json;

    fn rpc() -> Client {
        Client::new(
            "http://127.0.0.1:18443",
            Auth::UserPass("linkbit".into(), "linkbitpass".into())
        ).unwrap()
    }

    fn ensure_wallet_and_funds(client: &Client) {
        let _ = client.create_wallet("default", None, None, None, None);
        let balance = client.get_balance(None, None).unwrap_or(Amount::ZERO);

        if balance.to_btc() < 1.0 {
            println!("Current balance: {} BTC. Mining 101 blocks to fund wallet...", balance.to_btc());
            let addr = client.get_new_address(None, None).unwrap().assume_checked();
            client.generate_to_address(101, &addr).expect("Failed to mine blocks");
            let new_balance = client.get_balance(None, None).unwrap();
            println!("New balance: {} BTC", new_balance.to_btc());
        } else {
            println!("Wallet already funded: {} BTC", balance.to_btc());
        }
    }

    #[test]
    #[ignore = "Requires running Bitcoin Regtest node"]
    fn test_full_escrow_lifecycle_on_regtest() {
        let client = rpc();
        ensure_wallet_and_funds(&client);

        // 1. Setup Keys
        let (_, borrower_pk) = dev_keypair(DevRole::Borrower);
        let (_, lender_pk) = dev_keypair(DevRole::Lender);
        let (_, escrow_pk) = dev_keypair(DevRole::Escrow);

        let mut all_pubkeys = vec![borrower_pk, lender_pk, escrow_pk];
        // Sort keys to match the order used in create_2of3_multisig
        all_pubkeys.sort_by_key(|k| k.to_string());

        // 2. Create Multisig Address
        let (escrow_address, redeem_script) =
            create_2of3_multisig(all_pubkeys.clone(), Network::Regtest);

        println!("Escrow Address: {}", escrow_address);

        // 3. Fund Escrow
        let fund_amount = Amount::from_sat(1_000_000);
        let txid = client.send_to_address(
            &escrow_address,
            fund_amount,
            None, None, None, None, None, None
        ).expect("Funding failed");

        let address = client.get_new_address(None, None).unwrap().assume_checked();
        client.generate_to_address(1, &address).unwrap();

        // 4. Verify Funding
        let raw_tx = client.get_raw_transaction(&txid, None).unwrap();
        let status = check_funding_status(&raw_tx, 1, &escrow_address, 1_000_000, 1);
        assert_eq!(status, FundingStatus::Confirmed);

        // 5. Import Address to Wallet
        // Use addmultisigaddress which is higher level and handles descriptors automatically
        // addmultisigaddress(nrequired, keys, label, address_type)
        let keys_json: Vec<String> = all_pubkeys.iter().map(|k| k.to_string()).collect();

        // We use "p2sh-segwit" (p2wsh wrapped in p2sh) or "bech32" (native segwit p2wsh)?
        // create_2of3_multisig creates a P2WSH address (native segwit).
        // So we should specify "bech32" for address_type.

        let res = client.call::<serde_json::Value>(
            "addmultisigaddress",
            &[
                json!(2),
                json!(keys_json),
                json!("escrow_multisig"),
                json!("bech32")
            ]
        );

        if let Err(e) = res {
            println!("addmultisigaddress failed: {:?}. Trying importaddress fallback...", e);
            let _ = client.import_address(&escrow_address, Some("escrow_legacy"), Some(true));
        }

        // Rescan to find the UTXO
        let _ = client.rescan_blockchain(Some(0), None);

        // Fetch UTXOs
        let utxos = get_utxos(&client, &escrow_address, 1).unwrap();

        // Debug info if empty
        if utxos.is_empty() {
            println!("DEBUG: No UTXOs found. Checking wallet info...");
            let wallet_info = client.get_wallet_info().unwrap();
            println!("Wallet info: {:?}", wallet_info);
        }

        assert!(!utxos.is_empty(), "No UTXOs found. Import failed.");

        // 6. Build Repayment
        let lender_payout_address = client.get_new_address(None, None).unwrap()
            .require_network(Network::Regtest).unwrap();

        let unsigned_tx = build_repayment_tx(
            utxos,
            &lender_payout_address,
            990_000,
            10
        ).unwrap();

        // 7. Sign & Finalize
        let mut psbt = Psbt::from_unsigned_tx(unsigned_tx).unwrap();

        for input in &mut psbt.inputs {
            input.witness_script = Some(redeem_script.clone());
            input.witness_utxo = Some(bitcoin::TxOut {
                value: fund_amount,
                script_pubkey: escrow_address.script_pubkey(),
            });
        }

        psbt = sign_psbt_dev(psbt, DevRole::Borrower);
        psbt = sign_psbt_dev(psbt, DevRole::Lender);

        let psbt_base64 = base64::engine::general_purpose::STANDARD.encode(psbt.serialize());

        let final_tx = validate_psbt_and_finalize(
            &psbt_base64,
            &all_pubkeys,
            &lender_payout_address,
            20_000
        ).expect("Validation failed");

        // 8. Broadcast
        let final_txid = broadcast_tx(&client, &final_tx).expect("Broadcast failed");
        println!("Release TXID: {}", final_txid);

        client.generate_to_address(1, &address).unwrap();
        let release_tx = client.get_transaction(&final_txid, None).unwrap();
        assert!(release_tx.info.confirmations > 0);
    }
}

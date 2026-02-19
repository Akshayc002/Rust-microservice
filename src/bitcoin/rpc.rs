use bitcoincore_rpc::{Auth, Client, RpcApi, Result};
use bitcoin::{Address, Txid, Transaction};

#[allow(dead_code)]
pub fn rpc_client() -> Client {
    let url = std::env::var("BTC_RPC_URL").unwrap();
    let user = std::env::var("BTC_RPC_USER").unwrap();
    let pass = std::env::var("BTC_RPC_PASS").unwrap();

    Client::new(&url, Auth::UserPass(user, pass)).unwrap()
}

pub fn get_utxos(
    client: &Client,
    address: &Address,
    min_confirmations: usize
) -> Result<Vec<bitcoincore_rpc::bitcoincore_rpc_json::ListUnspentResultEntry>> {
    client.list_unspent(
        Some(min_confirmations),
        Some(9999999),
        Some(&[address]),
        Some(true),
        None
    )
}

pub fn broadcast_tx(client: &Client, tx: &Transaction) -> Result<Txid> {
    client.send_raw_transaction(tx)
}

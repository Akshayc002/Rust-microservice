use bitcoin::Network;

pub fn bitcoin_network() -> Network {
    match std::env::var("BTC_NETWORK").as_deref() {
        Ok("regtest") => Network::Regtest,
        Ok("testnet") => Network::Testnet,
        Ok("mainnet") => Network::Bitcoin,
        _ => Network::Regtest, // SAFE DEFAULT
    }
}

pub fn dev_signing_enabled() -> bool {
    std::env::var("DEV_SIGNING")
        .map(|v| v == "true")
        .unwrap_or(false)
}

pub fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        // Default PostgreSQL connection for local development
        "postgresql://linkbit:linkbitpass@localhost:5432/linkbit_mvp".to_string()
    })
}

pub fn phase() -> Phase {
    match std::env::var("PHASE").as_deref() {
        Ok("phase2") => Phase::Phase2Testnet,
        Ok("phase3") => Phase::Phase3Mainnet,
        _ => Phase::Phase1Regtest, // SAFE DEFAULT - Phase 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Phase1Regtest,
    Phase2Testnet,
    Phase3Mainnet,
}

#[allow(dead_code)]
pub fn is_production() -> bool {
    phase() == Phase::Phase3Mainnet
}

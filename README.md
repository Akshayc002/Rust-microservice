# LinkBit Rust Microservice

## 📋 Project Overview
This is the **Bitcoin Oracle & Escrow Service** for the LinkBit platform. Built with Rust, it handles high-security and high-performance operations related to the Bitcoin network.

It serves two main purposes:
1.  **Price Oracle**: Fetches real-time Bitcoin prices from multiple exchanges (Kraken, Binance, CoinGecko) and pushes authorized price snapshots to the PostgreSQL database.
2.  **Escrow Management**: Generates and manages 2-of-3 Multisig P2SH addresses (Lender + Borrower + Platform) for securing loan collateral.

## 🛠️ Tech Stack
- **Language**: Rust (2021 Edition)
- **Web Framework**: Axum
- **Database**: sqlx (PostgreSQL)
- **Bitcoin**: `rust-bitcoin`, `bitcoind-rpc`
- **Runtime**: Tokio

## 🐳 Docker Ecosystem
This service runs as the `linkbit-rust-service` container in the LinkBit ecosystem.

- **Port**: `9000`
- **Dependencies**:
    - `linkbit-postgres`: For storing price snapshots.
    - `linkbit-bitcoind`: For broadcasting transactions and monitoring the blockchain.

## 🚀 Getting Started

### Prerequisites
- Docker & Docker Compose
- Rust & Cargo (for local dev)

### Running via Docker
This service is included in the root `docker-compose.yml`.

```bash
docker-compose up -d rust-service
```

### Local Development
To run this service locally:

1.  **Set Environment Variables** (create `.env`):
    ```env
    DATABASE_URL=postgresql://linkbit:linkbitpass@localhost:5432/linkbit_mvp
    BITCOIN_RPC_URL=http://localhost:18443
    BITCOIN_RPC_USER=linkbit
    BITCOIN_RPC_PASSWORD=linkbitpass
    RUST_LOG=debug
    ```

2.  **Run with Cargo**:
    ```bash
    cargo run
    ```

## 🔌 API Endpoints

| Method | Path | Description |
| :--- | :--- | :--- |
| `GET` | `/health` | Service health check |
| `POST` | `/api/oracle/update` | Trigger manual price update |
| `GET` | `/api/escrow/generate` | Generate new multisig address |
| `POST` | `/api/escrow/verify` | Verify collateral deposit |

## 🧪 Testing
Run the test suite:
```bash
cargo test
```

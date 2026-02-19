use axum::Router;
use tokio::net::TcpListener;
use tracing_subscriber::fmt::init;
use sqlx::PgPool;
use tower_http::cors::CorsLayer;

mod api;
mod bitcoin;
mod domain;
mod security;
mod config;
mod db;
mod loans;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    init();
    
    // Debug: Print DB configuration
    println!("🔌 DB URL Config: {}", config::database_url());


    // Initialize PostgreSQL connection pool
    let pool = db::get_pool()
        .await
        .expect("Failed to initialize database connection");

    let _state = AppState { db: pool.clone() };

    // CORS configuration for frontend access
    let cors = CorsLayer::permissive();

    let app = Router::new()
        .merge(api::routes())
        .layer(cors);

    let listener = TcpListener::bind("0.0.0.0:9000")
        .await
        .expect("Failed to bind port");

    println!("🚀 Linkbit Bitcoin Escrow Service running on 0.0.0.0:9000");
    println!("📊 Connected to PostgreSQL database");
    println!("🌍 Phase: {:?}", config::phase());

    // Spawn background scheduler
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        tracing::info!("🕒 Scheduler started");
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3));
        loop {
            interval.tick().await;
            tracing::info!("🔄 Running scheduled tasks...");

            // 1. Update Oracle Prices
            let enable_oracle = std::env::var("ENABLE_ORACLE_UPDATE").unwrap_or_else(|_| "true".to_string()) == "true";
            let simulation_mode = std::env::var("SIMULATION_MODE").unwrap_or_else(|_| "true".to_string()) == "true";

            if enable_oracle {
                if simulation_mode {
                    if let Err(e) = loans::oracle::simulate_and_store_prices(&pool_clone).await {
                        tracing::error!("❌ Failed to simulate price: {}", e);
                    }
                } else {
                    if let Err(e) = loans::oracle::fetch_and_store_prices(&pool_clone).await {
                        tracing::error!("❌ Failed to fetch prices: {}", e);
                    }
                }
            } else {
                tracing::debug!("⏩ Oracle update disabled by config");
            }

            // 2. Check Margin Calls
            match loans::loans::get_active_loans(&pool_clone).await {
                Ok(active_loans) => {
                    tracing::info!("🔎 Checking {} active loans for margin calls", active_loans.len());
                    for loan in active_loans {
                        match loans::oracle::calculate_required_collateral(&pool_clone, loan.principal_satoshis, loan.ltv_ratio).await {
                            Ok(required) => {
                                tracing::info!("💰 Loan {}: Principal={} sats, Required Collateral={} sats", loan.id, loan.principal_satoshis, required);
                                // Here we would compare with actual collateral
                            },
                            Err(e) => tracing::error!("❌ Failed to calc collateral for loan {}: {}", loan.id, e),
                        }
                    }
                },
                Err(e) => tracing::error!("❌ Failed to fetch active loans: {}", e),
            }
        }
    });

    axum::serve(listener, app)
        .await
        .unwrap();
}

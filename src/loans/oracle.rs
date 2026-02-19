use sqlx::PgPool;
use crate::db::models::{BtcPriceSnapshot, BtcPriceMedian};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use rand::Rng;

/// Simulate random price movement and store in database
#[allow(dead_code)]
pub async fn simulate_and_store_prices(pool: &PgPool) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    // 1. Get latest price or default
    let latest = get_latest_price_median(pool).await?;
    let current_price = match latest {
        Some(p) => p,
        None => 95000.0, // Default starting price
    };

    // 2. Generate random change between -0.5% and +0.5%
    let change_percent = {
        let mut rng = rand::thread_rng();
        rng.gen_range(-0.005..0.005)
    };
    let new_price = current_price * (1.0 + change_percent);

    // 3. Store new price
    sqlx::query(
        r#"
        INSERT INTO btc_price_snapshots (price_usd, price_inr, source, snapshot_at)
        VALUES ($1::NUMERIC, $2::NUMERIC, 'simulator', NOW()::TIMESTAMP)
        "#
    )
    .bind(new_price)
    .bind(new_price * 85.0) // Mock INR conversion
    .execute(pool)
    .await?;

    tracing::info!("🎲 Simulated price update: ${:.2} ({:+.2}%)", new_price, change_percent * 100.0);
    
    Ok(new_price)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct OraclePrice {
    pub price_usd: f64,
    pub source: String,
    pub timestamp: chrono::DateTime<Utc>,
}

/// Fetch Bitcoin price from Kraken API
#[allow(dead_code)]
pub async fn fetch_kraken_price() -> Result<OraclePrice, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.kraken.com/0/public/Ticker?pair=XBTUSDT")
        .send()
        .await?;

    let data: serde_json::Value = response.json().await?;
    let price_str = data["result"]["XXBTZUSD"]["c"][0]
        .as_str()
        .ok_or("Invalid Kraken response")?;
    let price_usd: f64 = price_str.parse()?;

    Ok(OraclePrice {
        price_usd,
        source: "kraken".to_string(),
        timestamp: Utc::now(),
    })
}

/// Fetch Bitcoin price from Binance API
#[allow(dead_code)]
pub async fn fetch_binance_price() -> Result<OraclePrice, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.binance.com/api/v3/ticker/price?symbol=BTCUSDT")
        .send()
        .await?;

    let data: serde_json::Value = response.json().await?;
    let price_str = data["price"]
        .as_str()
        .ok_or("Invalid Binance response")?;
    let price_usd: f64 = price_str.parse()?;

    Ok(OraclePrice {
        price_usd,
        source: "binance".to_string(),
        timestamp: Utc::now(),
    })
}

/// Fetch Bitcoin price from CoinGecko API
#[allow(dead_code)]
pub async fn fetch_coingecko_price() -> Result<OraclePrice, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd")
        .send()
        .await?;

    let data: serde_json::Value = response.json().await?;
    let price_usd = data["bitcoin"]["usd"]
        .as_f64()
        .ok_or("Invalid CoinGecko response")?;

    Ok(OraclePrice {
        price_usd,
        source: "coingecko".to_string(),
        timestamp: Utc::now(),
    })
}

/// Fetch all oracle prices and store in database
#[allow(dead_code)]
pub async fn fetch_and_store_prices(
    pool: &PgPool,
) -> Result<BtcPriceMedian, Box<dyn std::error::Error + Send + Sync>> {
    let mut prices = vec![];

    // Fetch from all three sources in parallel
    let kraken_result = fetch_kraken_price().await;
    let binance_result = fetch_binance_price().await;
    let coingecko_result = fetch_coingecko_price().await;

    // Collect successful prices
    if let Ok(price) = kraken_result {
        prices.push(price);
    }
    if let Ok(price) = binance_result {
        prices.push(price);
    }
    if let Ok(price) = coingecko_result {
        prices.push(price);
    }

    if prices.is_empty() {
        return Err("Failed to fetch prices from all oracles".into());
    }

    // Store prices in database
    for price in &prices {
        sqlx::query(
            r#"
            INSERT INTO btc_price_snapshots (price_usd, price_inr, source, snapshot_at)
            VALUES ($1::NUMERIC, $2::NUMERIC, $3, $4::TIMESTAMP)
            "#
        )
        .bind(price.price_usd)
        .bind(price.price_usd * 85.0) // Mock INR conversion
        .bind(&price.source)
        .bind(price.timestamp)
        .execute(pool)
        .await?;
    }

    // Calculate median
    prices.sort_by(|a, b| a.price_usd.partial_cmp(&b.price_usd).unwrap());
    let median_price_usd = if prices.len() % 2 == 0 {
        (prices[prices.len() / 2 - 1].price_usd + prices[prices.len() / 2].price_usd) / 2.0
    } else {
        prices[prices.len() / 2].price_usd
    };

    // Fetch stored snapshots for return
    let snapshots = sqlx::query_as::<_, BtcPriceSnapshot>(
        "SELECT id, price_usd::DOUBLE PRECISION, price_inr::DOUBLE PRECISION, source, snapshot_at AT TIME ZONE 'UTC' as snapshot_at, created_at AT TIME ZONE 'UTC' as created_at FROM btc_price_snapshots WHERE snapshot_at > NOW() - INTERVAL '1 hour' ORDER BY snapshot_at DESC"
    )
    .fetch_all(pool)
    .await?;

    tracing::info!(
        "💰 Bitcoin price snapshot: median=${:.2} from {} sources",
        median_price_usd,
        snapshots.len()
    );

    Ok(BtcPriceMedian {
        median_price_usd,
        prices: snapshots,
        snapshot_time: Utc::now(),
    })
}

/// Get latest Bitcoin price median
#[allow(dead_code)]
pub async fn get_latest_price_median(
    pool: &PgPool,
) -> Result<Option<f64>, Box<dyn std::error::Error + Send + Sync>> {
    // Get prices from last 6 hours
    let snapshots = sqlx::query_as::<_, BtcPriceSnapshot>(
        "SELECT id, price_usd::DOUBLE PRECISION, price_inr::DOUBLE PRECISION, source, snapshot_at AT TIME ZONE 'UTC' as snapshot_at, created_at AT TIME ZONE 'UTC' as created_at FROM btc_price_snapshots WHERE snapshot_at > NOW() - INTERVAL '6 hours' ORDER BY snapshot_at DESC"
    )
    .fetch_all(pool)
    .await?;

    if snapshots.is_empty() {
        return Ok(None);
    }

    // Extract prices and calculate median
    let mut prices: Vec<f64> = snapshots
        .iter()
        .map(|s| s.price_usd.to_string().parse::<f64>().unwrap_or(0.0))
        .collect();

    prices.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let median = if prices.len() % 2 == 0 {
        (prices[prices.len() / 2 - 1] + prices[prices.len() / 2]) / 2.0
    } else {
        prices[prices.len() / 2]
    };

    Ok(Some(median))
}

/// Calculate required collateral in satoshis based on LTV and current BTC price
#[allow(dead_code)]
pub async fn calculate_required_collateral(
    pool: &PgPool,
    principal_satoshis: i64,
    ltv_ratio: i32,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    let median_price = get_latest_price_median(pool).await?
        .ok_or("No price data available")?;

    // LTV ratio is in basis points (100 = 1%)
    // Required collateral = (principal_satoshis / price_usd) * (10000 / ltv_ratio)
    let principal_usd = principal_satoshis as f64 / 100_000_000.0 * median_price; // sats to BTC to USD
    let required_btc = principal_usd * (10000 as f64 / ltv_ratio as f64) / median_price;
    let required_satoshis = (required_btc * 100_000_000.0) as i64;

    tracing::info!(
        "💰 Required collateral: {} sats (based on LTV {}%, price ${})",
        required_satoshis,
        ltv_ratio / 100,
        median_price
    );

    Ok(required_satoshis)
}

/// Clean up old price snapshots (keep last 30 days)
#[allow(dead_code)]
pub async fn cleanup_old_prices(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM btc_price_snapshots WHERE created_at < NOW() - INTERVAL '30 days'"
    )
    .execute(pool)
    .await?;

    tracing::info!("🗑️  Cleaned up {} old price snapshots", result.rows_affected());
    Ok(result.rows_affected())
}

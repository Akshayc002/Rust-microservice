use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use crate::config::database_url;
/// Initialize PostgreSQL connection pool
pub async fn get_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = database_url();
    tracing::info!("Connecting to PostgreSQL: {}", database_url);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    // Run migrations
    run_migrations(&pool).await?;
    tracing::info!("? Database connected and migrations completed");
    Ok(pool)
}
/// Run database migrations
async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    tracing::info!("Running migrations...");
    // Create loans table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS loans (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            borrower_id UUID NOT NULL,
            lender_id UUID NOT NULL,
            principal_satoshis BIGINT NOT NULL,
            interest_rate_bps INT NOT NULL,
            tenure_days INT NOT NULL,
            ltv_ratio INT NOT NULL,
            margin_call_threshold INT NOT NULL,
            status VARCHAR(50) NOT NULL DEFAULT 'offer_created',
            escrow_id UUID,
            escrow_address VARCHAR(255),
            agreement_id UUID,
            agreement_hash VARCHAR(255),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;
    // Create escrows table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS escrows (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            loan_id UUID NOT NULL REFERENCES loans(id),
            address VARCHAR(255) NOT NULL UNIQUE,
            redeem_script TEXT NOT NULL,
            borrower_pubkey VARCHAR(255) NOT NULL,
            lender_pubkey VARCHAR(255) NOT NULL,
            platform_pubkey VARCHAR(255) NOT NULL,
            platform_key_id VARCHAR(255),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            lender_signed_at TIMESTAMP,
            borrower_signed_at TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;
    // Create agreements table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agreements (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            loan_id UUID NOT NULL REFERENCES loans(id),
            terms_hash VARCHAR(255) NOT NULL,
            terms_json JSONB NOT NULL,
            borrower_signed_at TIMESTAMP,
            lender_signed_at TIMESTAMP,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;
    // Create payment proofs table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS payment_proofs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            loan_id UUID NOT NULL REFERENCES loans(id),
            emi_number INT NOT NULL,
            proof_type VARCHAR(50) NOT NULL,
            amount_satoshis BIGINT NOT NULL,
            proof_data JSONB NOT NULL,
            submitted_by VARCHAR(50) NOT NULL,
            submitted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            acknowledged_at TIMESTAMP,
            disputed_at TIMESTAMP,
            verified_at TIMESTAMP,
            status VARCHAR(50) DEFAULT 'pending',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;
    // Create margin calls table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS margin_calls (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            loan_id UUID NOT NULL REFERENCES loans(id),
            triggered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            grace_period_until TIMESTAMP NOT NULL,
            shortfall_satoshis BIGINT NOT NULL,
            acknowledged_at TIMESTAMP,
            resolved_at TIMESTAMP,
            resolution_type VARCHAR(50),
            status VARCHAR(50) DEFAULT 'active',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;
    // Create bitcoin price snapshots table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS btc_price_snapshots (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            price_usd DECIMAL(16, 2) NOT NULL,
            price_inr DECIMAL(38, 2) NOT NULL,
            source VARCHAR(50) NOT NULL,
            snapshot_at TIMESTAMP NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;
    // Create audit logs table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS audit_logs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            loan_id UUID,
            event_type VARCHAR(100) NOT NULL,
            actor_id UUID,
            actor_role VARCHAR(50),
            details JSONB,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;
    tracing::info!("? All migrations completed");
    Ok(())
}

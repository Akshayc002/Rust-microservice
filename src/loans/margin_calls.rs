use uuid::Uuid;
use sqlx::PgPool;
use crate::db::models::MarginCall;
use chrono::Utc;

/// Trigger margin call when collateral falls below threshold
#[allow(dead_code)]
pub async fn trigger_margin_call(
    pool: &PgPool,
    loan_id: Uuid,
    grace_period_hours: i32,
    shortfall_satoshis: i64,
) -> Result<MarginCall, sqlx::Error> {
    let grace_period_until = Utc::now() + chrono::Duration::hours(grace_period_hours as i64);

    let margin_call = sqlx::query_as::<_, MarginCall>(
        r#"
        INSERT INTO margin_calls (
            loan_id, grace_period_until, shortfall_satoshis, status
        )
        VALUES ($1, $2, $3, 'active')
        RETURNING *
        "#
    )
    .bind(loan_id)
    .bind(grace_period_until)
    .bind(shortfall_satoshis)
    .fetch_one(pool)
    .await?;

    tracing::warn!(
        "🚨 MARGIN CALL triggered for loan {}: shortfall={} sats, grace until {}",
        loan_id,
        shortfall_satoshis,
        grace_period_until
    );

    Ok(margin_call)
}

/// Get active margin call for a loan
#[allow(dead_code)]
pub async fn get_active_margin_call(
    pool: &PgPool,
    loan_id: Uuid,
) -> Result<Option<MarginCall>, sqlx::Error> {
    sqlx::query_as::<_, MarginCall>(
        "SELECT * FROM margin_calls WHERE loan_id = $1 AND status = 'active' ORDER BY triggered_at DESC LIMIT 1"
    )
    .bind(loan_id)
    .fetch_optional(pool)
    .await
}

/// Acknowledge margin call (borrower or lender acknowledges the call)
#[allow(dead_code)]
pub async fn acknowledge_margin_call(
    pool: &PgPool,
    margin_call_id: Uuid,
) -> Result<MarginCall, sqlx::Error> {
    let margin_call = sqlx::query_as::<_, MarginCall>(
        "UPDATE margin_calls SET acknowledged_at = NOW() WHERE id = $1 RETURNING *"
    )
    .bind(margin_call_id)
    .fetch_one(pool)
    .await?;

    tracing::info!("✅ Margin call {} acknowledged", margin_call_id);
    Ok(margin_call)
}

/// Resolve margin call (collateral added or partial repayment made)
#[allow(dead_code)]
pub async fn resolve_margin_call(
    pool: &PgPool,
    margin_call_id: Uuid,
    resolution_type: &str,
) -> Result<MarginCall, sqlx::Error> {
    let margin_call = sqlx::query_as::<_, MarginCall>(
        r#"
        UPDATE margin_calls
        SET resolved_at = NOW(), resolution_type = $1, status = 'resolved'
        WHERE id = $2
        RETURNING *
        "#
    )
    .bind(resolution_type)
    .bind(margin_call_id)
    .fetch_one(pool)
    .await?;

    tracing::info!(
        "✅ Margin call {} resolved via: {}",
        margin_call_id,
        resolution_type
    );
    Ok(margin_call)
}

/// Escalate margin call to liquidation
#[allow(dead_code)]
pub async fn escalate_to_liquidation(
    pool: &PgPool,
    margin_call_id: Uuid,
) -> Result<MarginCall, sqlx::Error> {
    let margin_call = sqlx::query_as::<_, MarginCall>(
        r#"
        UPDATE margin_calls
        SET status = 'escalated_to_liquidation'
        WHERE id = $1
        RETURNING *
        "#
    )
    .bind(margin_call_id)
    .fetch_one(pool)
    .await?;

    tracing::warn!(
        "🔴 Margin call {} escalated to liquidation",
        margin_call_id
    );
    Ok(margin_call)
}

#[allow(dead_code)]
pub fn is_grace_period_expired(margin_call: &MarginCall) -> bool {
    Utc::now() > margin_call.grace_period_until
}

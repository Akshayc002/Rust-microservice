use sqlx::PgPool;
use crate::db::models::Loan;

/// Fetch all active loans from the database
#[allow(dead_code)]
pub async fn get_active_loans(pool: &PgPool) -> Result<Vec<Loan>, sqlx::Error> {
    sqlx::query_as::<_, Loan>(
        "SELECT id, offer_id, borrower_id, lender_id, status::TEXT, principal_amount::DOUBLE PRECISION, interest_rate_apy::DOUBLE PRECISION, duration_days, collateral_btc_amount::DOUBLE PRECISION, margin_call_ltv_percent::DOUBLE PRECISION, liquidation_ltv_percent::DOUBLE PRECISION, created_at::TIMESTAMPTZ, updated_at::TIMESTAMPTZ FROM loans WHERE status = 'ACTIVE' OR status = 'COLLATERAL_LOCKED' OR status = 'MARGIN_CALL'"
    )
    .fetch_all(pool)
    .await
}

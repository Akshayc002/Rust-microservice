use sqlx::PgPool;
use crate::db::models::Loan;

/// Fetch all active loans from the database
#[allow(dead_code)]
pub async fn get_active_loans(pool: &PgPool) -> Result<Vec<Loan>, sqlx::Error> {
    sqlx::query_as::<_, Loan>(
        "SELECT * FROM loans WHERE status = 'active_loan' OR status = 'collateral_verified' OR status = 'disbursement_acknowledged'"
    )
    .fetch_all(pool)
    .await
}

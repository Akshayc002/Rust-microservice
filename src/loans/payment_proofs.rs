use uuid::Uuid;
use sqlx::PgPool;
use crate::db::models::PaymentProof;
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SubmitPaymentProofRequest {
    pub loan_id: Uuid,
    pub emi_number: i32,
    pub proof_type: String, // "bank_transfer", "blockchain_tx", "signed_message", "invoice"
    pub amount_satoshis: i64,
    pub proof_data: serde_json::Value, // Flexible structure depending on type
}

/// Submit payment proof (by borrower when they claim to have paid, or by lender)
#[allow(dead_code)]
pub async fn submit_payment_proof(
    pool: &PgPool,
    loan_id: Uuid,
    emi_number: i32,
    proof_type: &str,
    amount_satoshis: i64,
    proof_data: serde_json::Value,
    submitted_by: &str, // "borrower" or "lender"
) -> Result<PaymentProof, sqlx::Error> {
    let proof = sqlx::query_as::<_, PaymentProof>(
        r#"
        INSERT INTO payment_proofs (
            loan_id, emi_number, proof_type, amount_satoshis,
            proof_data, submitted_by, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'pending')
        RETURNING *
        "#
    )
    .bind(loan_id)
    .bind(emi_number)
    .bind(proof_type)
    .bind(amount_satoshis)
    .bind(proof_data)
    .bind(submitted_by)
    .fetch_one(pool)
    .await?;

    tracing::info!(
        "📝 Payment proof submitted for loan {} EMI {}: {} sats via {}",
        loan_id,
        emi_number,
        amount_satoshis,
        proof_type
    );

    Ok(proof)
}

/// Acknowledge payment proof (counterparty acknowledges receipt of payment)
#[allow(dead_code)]
pub async fn acknowledge_payment_proof(
    pool: &PgPool,
    proof_id: Uuid,
) -> Result<PaymentProof, sqlx::Error> {
    let proof = sqlx::query_as::<_, PaymentProof>(
        "UPDATE payment_proofs SET acknowledged_at = NOW(), status = 'acknowledged' WHERE id = $1 RETURNING *"
    )
    .bind(proof_id)
    .fetch_one(pool)
    .await?;

    tracing::info!("✅ Payment proof {} acknowledged", proof_id);
    Ok(proof)
}

/// Dispute payment proof (within 72-hour window post-acknowledgment)
#[allow(dead_code)]
pub async fn dispute_payment_proof(
    pool: &PgPool,
    proof_id: Uuid,
) -> Result<PaymentProof, sqlx::Error> {
    let proof = sqlx::query_as::<_, PaymentProof>(
        "UPDATE payment_proofs SET disputed_at = NOW(), status = 'disputed' WHERE id = $1 RETURNING *"
    )
    .bind(proof_id)
    .fetch_one(pool)
    .await?;

    tracing::warn!("🚨 Payment proof {} disputed - escalating to operations", proof_id);
    Ok(proof)
}

/// Mark payment proof as verified (by operations or automated system)
#[allow(dead_code)]
pub async fn verify_payment_proof(
    pool: &PgPool,
    proof_id: Uuid,
) -> Result<PaymentProof, sqlx::Error> {
    let proof = sqlx::query_as::<_, PaymentProof>(
        "UPDATE payment_proofs SET verified_at = NOW(), status = 'verified' WHERE id = $1 RETURNING *"
    )
    .bind(proof_id)
    .fetch_one(pool)
    .await?;

    tracing::info!("✅ Payment proof {} verified", proof_id);
    Ok(proof)
}

/// Get payment proof by ID
#[allow(dead_code)]
pub async fn get_payment_proof(
    pool: &PgPool,
    proof_id: Uuid,
) -> Result<Option<PaymentProof>, sqlx::Error> {
    sqlx::query_as::<_, PaymentProof>(
        "SELECT * FROM payment_proofs WHERE id = $1"
    )
    .bind(proof_id)
    .fetch_optional(pool)
    .await
}

/// Get all payment proofs for a loan
#[allow(dead_code)]
pub async fn get_loan_payment_proofs(
    pool: &PgPool,
    loan_id: Uuid,
) -> Result<Vec<PaymentProof>, sqlx::Error> {
    sqlx::query_as::<_, PaymentProof>(
        "SELECT * FROM payment_proofs WHERE loan_id = $1 ORDER BY emi_number ASC"
    )
    .bind(loan_id)
    .fetch_all(pool)
    .await
}

/// Check if dispute window has passed (72 hours after acknowledgment)
#[allow(dead_code)]
pub fn is_dispute_window_expired(proof: &PaymentProof) -> bool {
    if let Some(ack_time) = proof.acknowledged_at {
        Utc::now() > ack_time + chrono::Duration::hours(72)
    } else {
        false
    }
}

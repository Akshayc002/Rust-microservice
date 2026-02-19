use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============ LOANS ============

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct Loan {
    pub id: Uuid,
    pub borrower_id: Uuid,
    pub lender_id: Uuid,
    pub principal_satoshis: i64,
    pub interest_rate_bps: i32,
    pub tenure_days: i32,
    pub ltv_ratio: i32,
    pub margin_call_threshold: i32,
    pub status: String,
    pub escrow_id: Option<Uuid>,
    pub escrow_address: Option<String>,
    pub agreement_id: Option<Uuid>,
    pub agreement_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum LoanState {
    OfferCreated,
    NegotiationInProgress,
    OfferConfirmed,
    AgreementGenerated,
    AgreementApprovedByBorrower,
    AgreementApprovedByLender,
    AgreementApprovedFinal,
    EscrowCreated,
    CollateralDeposited,
    CollateralVerified,
    DisbursementProofAwaitingLender,
    DisbursementProofAwaitingBorrower,
    DisbursementAcknowledged,
    ActiveLoan,
    MarginCallTriggered,
    LiquidationEligible,
    LiquidationInProgress,
    LiquidationComplete,
    SettledNormally,
    Disputed,
}

impl std::fmt::Display for LoanState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LoanState::OfferCreated => "offer_created",
            LoanState::NegotiationInProgress => "negotiation_in_progress",
            LoanState::OfferConfirmed => "offer_confirmed",
            LoanState::AgreementGenerated => "agreement_generated",
            LoanState::AgreementApprovedByBorrower => "agreement_approved_borrower",
            LoanState::AgreementApprovedByLender => "agreement_approved_lender",
            LoanState::AgreementApprovedFinal => "agreement_approved_final",
            LoanState::EscrowCreated => "escrow_created",
            LoanState::CollateralDeposited => "collateral_deposited",
            LoanState::CollateralVerified => "collateral_verified",
            LoanState::DisbursementProofAwaitingLender => "disbursement_proof_awaiting_lender",
            LoanState::DisbursementProofAwaitingBorrower => "disbursement_proof_awaiting_borrower",
            LoanState::DisbursementAcknowledged => "disbursement_acknowledged",
            LoanState::ActiveLoan => "active_loan",
            LoanState::MarginCallTriggered => "margin_call_triggered",
            LoanState::LiquidationEligible => "liquidation_eligible",
            LoanState::LiquidationInProgress => "liquidation_in_progress",
            LoanState::LiquidationComplete => "liquidation_complete",
            LoanState::SettledNormally => "settled_normally",
            LoanState::Disputed => "disputed",
        };
        write!(f, "{}", s)
    }
}

// ============ ESCROWS ============

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct Escrow {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub address: String,
    pub redeem_script: String,
    pub borrower_pubkey: String,
    pub lender_pubkey: String,
    pub platform_pubkey: String,
    pub platform_key_id: Option<String>, // AWS KMS key ID
    pub created_at: DateTime<Utc>,
}

// ============ AGREEMENTS ============

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct Agreement {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub terms_hash: String,
    pub terms_json: serde_json::Value,
    pub borrower_signed_at: Option<DateTime<Utc>>,
    pub lender_signed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AgreementTerms {
    pub principal_satoshis: i64,
    pub interest_rate_bps: i32,
    pub tenure_days: i32,
    pub ltv_ratio: i32,
    pub margin_call_threshold: i32,
    pub margin_call_grace_period_hours: i32,
    pub liquidation_time_lock_hours: i32,
    pub penalty_rate_bps: i32,
    pub max_extensions: i32,
    pub oracle_sources: Vec<String>, // ["kraken", "binance", "coingecko"]
    pub borrower_description: Option<String>,
}

// ============ PAYMENT PROOFS ============

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct PaymentProof {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub emi_number: i32,
    pub proof_type: String, // "bank_transfer", "blockchain_tx", "signed_message", "invoice"
    pub amount_satoshis: i64,
    pub proof_data: serde_json::Value,
    pub submitted_by: String, // "borrower", "lender"
    pub submitted_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub disputed_at: Option<DateTime<Utc>>,
    pub verified_at: Option<DateTime<Utc>>,
    pub status: String, // "pending", "acknowledged", "disputed", "verified"
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum ProofType {
    BankTransfer,
    BlockchainTx,
    SignedMessage,
    Invoice,
}

// ============ MARGIN CALLS ============

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct MarginCall {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub triggered_at: DateTime<Utc>,
    pub grace_period_until: DateTime<Utc>,
    pub shortfall_satoshis: i64,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_type: Option<String>, // "collateral_added", "partially_repaid", "liquidated"
    pub status: String, // "active", "resolved", "escalated_to_liquidation"
    pub created_at: DateTime<Utc>,
}

// ============ BITCOIN PRICE SNAPSHOTS ============

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct BtcPriceSnapshot {
    pub id: Uuid,
    pub price_usd: f64,
    pub price_inr: f64,
    pub source: String, // "kraken", "binance", "coingecko"
    pub snapshot_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BtcPriceMedian {
    pub median_price_usd: f64,
    pub prices: Vec<BtcPriceSnapshot>,
    pub snapshot_time: DateTime<Utc>,
}

// ============ AUDIT LOGS ============

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct AuditLog {
    pub id: Uuid,
    pub loan_id: Option<Uuid>,
    pub event_type: String,
    pub actor_id: Option<Uuid>,
    pub actor_role: Option<String>, // "borrower", "lender", "platform", "system"
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

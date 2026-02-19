use axum::{
    Json,
    http::StatusCode,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};

use crate::bitcoin::psbt_verify::verify_2of3_psbt;
use crate::bitcoin::signing_registry;
use crate::domain::signing::SignerRole;

/// ===== Request =====
#[derive(Debug, Deserialize)]
pub struct SubmitSignedPsbtRequest {
    pub escrow_id: String,
    pub signer_role: String,
    pub psbt_base64: String,

    pub borrower_pubkey: String,
    pub lender_pubkey: String,
    pub escrow_pubkey: String,
}

/// ===== Response =====
#[derive(Debug, Serialize)]
pub struct SubmitSignedPsbtResponse {
    pub status: String,
    pub escrow_state: String,
    pub signatures_collected: usize,
}

/// ===== Routes =====
pub fn routes() -> Router {
    Router::new()
        .route("/psbt/submit-signed", post(submit_signed_psbt))
}

/// ===== Handler =====
pub async fn submit_signed_psbt(
    Json(req): Json<SubmitSignedPsbtRequest>,
) -> Result<Json<SubmitSignedPsbtResponse>, StatusCode> {

    // ---- Parse signer role ----
    let role = match req.signer_role.as_str() {
        "BORROWER" => SignerRole::Borrower,
        "LENDER" => SignerRole::Lender,
        "ESCROW" => SignerRole::Escrow,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    // ---- Parse public keys ----
    let borrower_pk = req.borrower_pubkey.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let lender_pk = req.lender_pubkey.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let escrow_pk = req.escrow_pubkey.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    let allowed_keys = vec![borrower_pk, lender_pk, escrow_pk];

    // ---- Verify PSBT (Phase 1: structural + signer validation) ----
    verify_2of3_psbt(&req.psbt_base64, &allowed_keys)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // ---- Load or create signing state ----
    let mut state = signing_registry::get_or_create(&req.escrow_id);

    // ---- Apply signature ----
    state.add_signature(role);

    let approved = state.is_approved();

    // ---- Persist updated state ----
    signing_registry::save(&req.escrow_id, state.clone());

    // ---- Build response ----
    let status = if approved {
        "APPROVED"
    } else {
        "PARTIALLY_SIGNED"
    };

    Ok(Json(SubmitSignedPsbtResponse {
        status: status.to_string(),
        escrow_state: format!("{:?}", state.state),
        signatures_collected: state.signed_roles.len(),
    }))
}
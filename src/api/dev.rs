use axum::{Router, routing::post, Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use base64::Engine;
use bitcoin::psbt::Psbt;

use crate::{config, bitcoin::dev_signer, bitcoin::dev_keys::DevRole};

#[derive(Deserialize)]
pub struct DevSignRequest {
    pub psbt_base64: String,
    pub role: String, // borrower | lender | escrow
}

#[derive(Serialize)]
pub struct DevSignResponse {
    pub psbt_base64: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub fn routes() -> Router {
    Router::new().route("/dev/sign-psbt", post(sign_psbt))
}

async fn sign_psbt(
    Json(req): Json<DevSignRequest>
) -> Result<Json<DevSignResponse>, (StatusCode, Json<ErrorResponse>)> {

    if !config::dev_signing_enabled() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse { error: "Dev signing disabled".into() })
        ));
    }

    let role = match req.role.as_str() {
        "borrower" => DevRole::Borrower,
        "lender" => DevRole::Lender,
        "escrow" => DevRole::Escrow,
        _ => return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Invalid role".into() })
        )),
    };

    let bytes = base64::engine::general_purpose::STANDARD.decode(req.psbt_base64)
        .map_err(|_| (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Invalid base64".into() })
        ))?;

    let psbt: Psbt = Psbt::deserialize(&bytes)
        .map_err(|_| (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Invalid PSBT".into() })
        ))?;

    let signed = dev_signer::sign_psbt_dev(psbt, role);

    Ok(Json(DevSignResponse {
        psbt_base64: base64::engine::general_purpose::STANDARD
            .encode(signed.serialize()),
    }))
}

use axum::http::{Request, StatusCode};
use axum::body::Body;
use tower::util::ServiceExt;
use serde_json::json;
use crate::bitcoin::test_fixtures::valid_test_psbt;
use base64::{engine::general_purpose, Engine};

use crate::api::routes;

#[tokio::test]
async fn submit_signed_psbt_rejects_invalid_role() {
    let app = routes();

    let req = Request::builder()
        .method("POST")
        .uri("/psbt/submit-signed")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "escrow_id": "escrow-1",
            "signer_role": "HACKER",
            "psbt_base64": general_purpose::STANDARD.encode(valid_test_psbt().serialize()),
            "borrower_pubkey": "02d0de0aaeaefad02b8bdc8a01a1b8b11c696bd3d66a2c9f9b7b8a9d3e6f5f6f",
            "lender_pubkey": "03a34b9d7f8c92d5b0b9d0a5a2d1e9c8b7a6d5e4f3c2b1a09876543210fedcba",
            "escrow_pubkey": "02b4632b9bfa4cbd9c74e8b4e6b2e8d0c4a7d6e5f8c9b0a1d2e3f4b5c6d7e8f"
        }).to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn submit_signed_psbt_accepts_first_signer() {
    let app = routes();

    let req = Request::builder()
        .method("POST")
        .uri("/psbt/submit-signed")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "escrow_id": "escrow-2",
            "signer_role": "BORROWER",
            "psbt_base64": general_purpose::STANDARD.encode(valid_test_psbt().serialize()),
            "borrower_pubkey": "02d0de0aaeaefad02b8bdc8a01a1b8b11c696bd3d66a2c9f9b7b8a9d3e6f5f6f",
            "lender_pubkey": "03a34b9d7f8c92d5b0b9d0a5a2d1e9c8b7a6d5e4f3c2b1a09876543210fedcba",
            "escrow_pubkey": "02b4632b9bfa4cbd9c74e8b4e6b2e8d0c4a7d6e5f8c9b0a1d2e3f4b5c6d7e8f"
        }).to_string()))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn submit_signed_psbt_approves_on_second_signer() {
    let app = routes();

    let body = |role| json!({
        "escrow_id": "escrow-3",
        "signer_role": role,
        "psbt_base64": general_purpose::STANDARD.encode(valid_test_psbt().serialize()),
        "borrower_pubkey": "02d0de0aaeaefad02b8bdc8a01a1b8b11c696bd3d66a2c9f9b7b8a9d3e6f5f6f",
        "lender_pubkey": "03a34b9d7f8c92d5b0b9d0a5a2d1e9c8b7a6d5e4f3c2b1a09876543210fedcba",
        "escrow_pubkey": "02b4632b9bfa4cbd9c74e8b4e6b2e8d0c4a7d6e5f8c9b0a1d2e3f4b5c6d7e8f"
    });

    let req1 = Request::builder()
        .method("POST")
        .uri("/psbt/submit-signed")
        .header("content-type", "application/json")
        .body(body("BORROWER").to_string())
        .unwrap();

    let req2 = Request::builder()
        .method("POST")
        .uri("/psbt/submit-signed")
        .header("content-type", "application/json")
        .body(body("LENDER").to_string())
        .unwrap();

    let res1 = app.clone().oneshot(req1).await.unwrap();
    let res2 = app.oneshot(req2).await.unwrap();

    assert_eq!(res1.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res2.status(), StatusCode::BAD_REQUEST);
}

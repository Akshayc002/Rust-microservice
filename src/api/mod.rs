use axum::Router;

pub mod escrow;
pub mod transaction;
pub mod health;
pub mod dev;
pub mod psbt_submit;

#[cfg(test)]
mod psbt_submit_tests;

pub fn routes() -> Router {
    Router::new()
        .merge(health::routes())
        .merge(escrow::routes())
        .merge(transaction::routes())
        .merge(psbt_submit::routes())
        .merge(dev::routes())
}

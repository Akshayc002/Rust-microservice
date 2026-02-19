use std::collections::HashMap;
use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::domain::signing::SigningState;

/// Global in-memory signing registry (Phase 1 only)
static REGISTRY: Lazy<Mutex<HashMap<String, SigningState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Get existing signing state or create a new one
pub fn get_or_create(escrow_id: &str) -> SigningState {
    let mut registry = REGISTRY
        .lock()
        .expect("signing registry mutex poisoned");

    registry
        .entry(escrow_id.to_string())
        .or_insert_with(|| SigningState::new(escrow_id.to_string()))
        .clone()
}

/// Save updated signing state
pub fn save(escrow_id: &str, state: SigningState) {
    let mut registry = REGISTRY
        .lock()
        .expect("signing registry mutex poisoned");

    registry.insert(escrow_id.to_string(), state);
}

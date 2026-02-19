use std::collections::HashSet;

use crate::domain::escrow_state::EscrowState;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SignerRole {
    Borrower,
    Lender,
    Escrow,
}

#[derive(Debug, Clone)]
pub struct SigningState {
    #[allow(dead_code)]
    pub escrow_id: String,
    pub signed_roles: HashSet<SignerRole>,
    pub state: EscrowState,
}

impl SigningState {
    pub fn new(escrow_id: String) -> Self {
        Self {
            escrow_id,
            signed_roles: HashSet::new(),
            state: EscrowState::Created,
        }
    }

    pub fn add_signature(&mut self, role: SignerRole) {
        self.signed_roles.insert(role);

        self.state = if self.signed_roles.len() >= 2 {
            EscrowState::Approved
        } else {
            EscrowState::Signing
        };
    }

    pub fn is_approved(&self) -> bool {
        self.state == EscrowState::Approved
    }
}

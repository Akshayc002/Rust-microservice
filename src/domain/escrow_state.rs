#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowState {
    Created,
    Signing,
    Approved,
}
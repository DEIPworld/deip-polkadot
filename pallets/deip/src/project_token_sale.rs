use crate::*;

/// Contains information about tokens of the project
#[derive(Encode, Decode, Default)]
pub struct TokenInfo {
    pub total: u64,
    pub reserved: u64,
}

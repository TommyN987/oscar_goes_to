//! The session ID a browser holds, and the hash storage keeps of it (spec rule 3.2.1).

use std::fmt;

use sha2::{Digest, Sha256};

/// The operating system gave no random bytes, so no session can be issued.
#[derive(Debug, thiserror::Error)]
#[error("the operating system gave no random bytes: {0}")]
pub struct RandomnessUnavailable(#[from] getrandom::Error);

/// A session ID: 256 random bits that only the browser holds.
///
/// Its `Debug` output hides the value, so it cannot reach a log or a span (spec rule 6.5.6).
#[derive(Clone, PartialEq, Eq)]
pub struct SessionToken([u8; 32]);

impl SessionToken {
    /// A new token from the operating system's random source.
    pub fn generate() -> Result<Self, RandomnessUnavailable> {
        let mut bytes = [0; 32];
        getrandom::fill(&mut bytes)?;
        Ok(Self(bytes))
    }

    /// The token a browser sent back.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// The token itself. Give it only to the browser it belongs to; never store or log it.
    pub const fn expose_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// What storage keeps in place of the token.
    pub fn hash(&self) -> SessionTokenHash {
        SessionTokenHash(Sha256::digest(self.0).into())
    }
}

impl fmt::Debug for SessionToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SessionToken(<redacted>)")
    }
}

/// The SHA-256 hash of a session token. A token cannot be recovered from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionTokenHash([u8; 32]);

impl SessionTokenHash {
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_3_2_1_every_token_is_new() {
        let first = SessionToken::generate().unwrap();
        let second = SessionToken::generate().unwrap();

        assert_ne!(first.expose_bytes(), second.expose_bytes());
    }

    #[test]
    fn rule_3_2_1_a_token_is_hashed_with_sha_256() {
        let token = SessionToken::from_bytes([0; 32]);

        let hash_in_hex: String = token
            .hash()
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(
            hash_in_hex,
            "66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925"
        );
    }

    #[test]
    fn rule_6_5_6_debug_output_hides_the_token() {
        let token = SessionToken::from_bytes([0xab; 32]);

        assert_eq!(format!("{token:?}"), "SessionToken(<redacted>)");
    }
}

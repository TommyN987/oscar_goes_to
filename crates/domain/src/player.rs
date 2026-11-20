//! Players: who may become one, and what they may be called.

use std::fmt;

/// Identifies a player. Storage assigns it; telemetry names a player by it alone (spec rule 2.7.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(i64);

impl PlayerId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> i64 {
        self.0
    }
}

/// What a player is called, or will be offered to be called.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerName {
    /// The player has not chosen a name yet. `suggested` is the provider's name for them,
    /// kept only to prefill the name step (spec rules 3.1.3 and 3.5.1).
    NotChosen {
        suggested: Option<String>,
    },
    Chosen(DisplayName),
}

/// A player's account (spec rule 3.5.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub id: PlayerId,
    pub email: String,
    pub name: PlayerName,
}

/// What the identity provider asserts about the person signing in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityClaims {
    /// The provider's stable ID for the account; a player is keyed by it (spec rule 1.6.1).
    pub subject: String,
    pub email: String,
    pub email_verified: bool,
    /// The name the provider knows, offered as the first display name.
    pub name: Option<String>,
}

/// The claims carried an email address the provider has not verified (spec rule 1.6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the email address is not verified")]
pub struct UnverifiedEmail;

/// Claims that may sign a player in: the email is verified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedIdentity {
    subject: String,
    email: String,
    name: Option<String>,
}

impl VerifiedIdentity {
    pub fn from_claims(claims: IdentityClaims) -> Result<Self, UnverifiedEmail> {
        if !claims.email_verified {
            return Err(UnverifiedEmail);
        }
        Ok(Self {
            subject: claims.subject,
            email: claims.email,
            name: claims.name,
        })
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// Why a string cannot be a display name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DisplayNameError {
    #[error("the display name is empty")]
    Empty,
    #[error("the display name is longer than {} characters", DisplayName::MAX_CHARACTERS)]
    TooLong,
    #[error("the display name contains a control character")]
    ControlCharacter,
}

/// The name other players see.
///
/// Invariant: 1 to 40 characters, none of them a control character, with no whitespace at either end
/// (spec rules 1.6.2 and 6.5.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayName(String);

impl DisplayName {
    pub const MAX_CHARACTERS: usize = 40;

    /// Trims `input` and accepts it if it then meets the invariant. Characters are counted, not bytes.
    pub fn parse(input: &str) -> Result<Self, DisplayNameError> {
        let name = input.trim();
        if name.is_empty() {
            return Err(DisplayNameError::Empty);
        }
        if name.chars().count() > Self::MAX_CHARACTERS {
            return Err(DisplayNameError::TooLong);
        }
        if name.chars().any(char::is_control) {
            return Err(DisplayNameError::ControlCharacter);
        }
        Ok(Self(name.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The key that decides whether two names are the same: the name in lowercase (spec rule 1.6.2).
    /// Storage keeps this key under a unique index and never derives it itself.
    pub fn uniqueness_key(&self) -> String {
        self.0.to_lowercase()
    }
}

impl fmt::Display for DisplayName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claims(email_verified: bool) -> IdentityClaims {
        IdentityClaims {
            subject: "google-subject-1".into(),
            email: "player@example.com".into(),
            email_verified,
            name: Some("Pat Player".into()),
        }
    }

    #[test]
    fn rule_1_6_1_an_unverified_email_is_refused() {
        assert_eq!(VerifiedIdentity::from_claims(claims(false)), Err(UnverifiedEmail));
    }

    #[test]
    fn rule_1_6_1_a_verified_identity_is_keyed_by_its_subject() {
        let identity = VerifiedIdentity::from_claims(claims(true)).unwrap();

        assert_eq!(identity.subject(), "google-subject-1");
        assert_eq!(identity.email(), "player@example.com");
        assert_eq!(identity.name(), Some("Pat Player"));
    }

    #[test]
    fn rule_6_5_2_a_display_name_has_1_to_40_characters() {
        let forty = "é".repeat(40);
        let forty_one = "é".repeat(41);

        assert_eq!(DisplayName::parse(""), Err(DisplayNameError::Empty));
        assert_eq!(DisplayName::parse("   "), Err(DisplayNameError::Empty));
        assert_eq!(DisplayName::parse("A").unwrap().as_str(), "A");
        assert_eq!(DisplayName::parse(&forty).unwrap().as_str(), forty);
        assert_eq!(DisplayName::parse(&forty_one), Err(DisplayNameError::TooLong));
    }

    #[test]
    fn rule_6_5_2_a_display_name_has_no_control_character() {
        assert_eq!(
            DisplayName::parse("Pat\u{0007}Player"),
            Err(DisplayNameError::ControlCharacter)
        );
        assert_eq!(
            DisplayName::parse("Pat\nPlayer"),
            Err(DisplayNameError::ControlCharacter)
        );
    }

    #[test]
    fn every_player_refusal_says_why() {
        assert_eq!(UnverifiedEmail.to_string(), "the email address is not verified");
        assert_eq!(DisplayNameError::Empty.to_string(), "the display name is empty");
        assert_eq!(
            DisplayNameError::TooLong.to_string(),
            "the display name is longer than 40 characters"
        );
        assert_eq!(
            DisplayNameError::ControlCharacter.to_string(),
            "the display name contains a control character"
        );
    }

    #[test]
    fn whitespace_around_a_display_name_is_trimmed() {
        assert_eq!(DisplayName::parse("  Pat Player \n").unwrap().as_str(), "Pat Player");
    }

    /// The part of spec rule 1.6.2 that needs no storage; plan item P3.5.2 closes the rule.
    #[test]
    fn names_that_differ_only_in_case_share_a_uniqueness_key() {
        let lower = DisplayName::parse("élodie b").unwrap();
        let mixed = DisplayName::parse("Élodie B").unwrap();
        let other = DisplayName::parse("Elodie B").unwrap();

        assert_eq!(lower.uniqueness_key(), mixed.uniqueness_key());
        assert_ne!(lower.uniqueness_key(), other.uniqueness_key());
        assert_eq!(mixed.as_str(), "Élodie B");
    }
}

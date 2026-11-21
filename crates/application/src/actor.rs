//! Who is asking: a use case takes an actor and decides itself what that actor may do (spec rule 3.3.1).
//! The two that take none are the ones that establish who is asking: sign-in and authenticate.

use domain::player::Player;

/// The actor is not a signed-in player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the actor is not signed in")]
pub struct NotSignedIn;

/// Who a use case acts for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Actor {
    /// A request without a live session.
    Anonymous,
    Player(Player),
    /// A player whose email the configuration names as an admin (spec rule 1.6.3).
    Admin(Player),
    /// The command line, which no HTTP request can be (spec rule 3.3.5).
    System,
}

impl Actor {
    /// The signed-in player this actor is, admin or not. A use case that acts for a player starts here.
    pub const fn require_player(&self) -> Result<&Player, NotSignedIn> {
        match self {
            Self::Player(player) | Self::Admin(player) => Ok(player),
            Self::Anonymous | Self::System => Err(NotSignedIn),
        }
    }

    pub const fn is_admin(&self) -> bool {
        matches!(self, Self::Admin(_))
    }
}

#[cfg(test)]
mod tests {
    use domain::player::{PlayerId, PlayerName};

    use super::*;

    fn player() -> Player {
        Player {
            id: PlayerId::new(7),
            email: "player@example.com".into(),
            name: PlayerName::NotChosen { suggested: None },
        }
    }

    #[test]
    fn a_player_and_an_admin_are_signed_in_and_nobody_else_is() {
        assert_eq!(Actor::Player(player()).require_player(), Ok(&player()));
        assert_eq!(Actor::Admin(player()).require_player(), Ok(&player()));
        assert_eq!(Actor::Anonymous.require_player(), Err(NotSignedIn));
        assert_eq!(Actor::System.require_player(), Err(NotSignedIn));
    }

    #[test]
    fn only_the_admin_actor_is_an_admin() {
        assert!(Actor::Admin(player()).is_admin());
        assert!(!Actor::Player(player()).is_admin());
        assert!(!Actor::Anonymous.is_admin());
        assert!(!Actor::System.is_admin());
    }
}

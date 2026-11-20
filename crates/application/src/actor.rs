//! Who is asking: a use case takes an actor and decides itself what that actor may do (spec rule 3.3.1).
//! The two that take none are the ones that establish who is asking: sign-in and authenticate.

use domain::player::Player;

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
    /// The signed-in player this actor is, admin or not.
    pub const fn player(&self) -> Option<&Player> {
        match self {
            Self::Player(player) | Self::Admin(player) => Some(player),
            Self::Anonymous | Self::System => None,
        }
    }
}

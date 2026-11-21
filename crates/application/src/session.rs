//! The session use cases: sign in, authenticate a request, who am I, sign out.

use std::error::Error;

use domain::player::{Player, VerifiedIdentity};
use domain::session::live_if_used_after;
use tracing::field::Empty;

use crate::actor::{Actor, NotSignedIn};
use crate::admin_emails::AdminEmails;
use crate::clock::Clock;
use crate::session_token::{RandomnessUnavailable, SessionToken};
use crate::storage::{PlayerStore, SessionStore, StorageError};

/// Why a sign-in did not happen.
#[derive(Debug, thiserror::Error)]
pub enum SignInError {
    #[error(transparent)]
    NoToken(#[from] RandomnessUnavailable),
    #[error(transparent)]
    Storage(#[from] StorageError),
}

/// Signs in the person `identity` describes and returns the token of their new session.
///
/// The session the browser `presented`, if any, is ended: every sign-in gets a new ID (spec rule 3.2.3).
/// The player's sessions on other devices stay.
/// The token is generated before anything is written, so a failure to generate it changes nothing.
#[tracing::instrument(skip_all, fields(player.id = Empty))]
pub async fn sign_in<S: PlayerStore + SessionStore>(
    identity: &VerifiedIdentity,
    presented: Option<&SessionToken>,
    storage: &S,
    clock: &dyn Clock,
) -> Result<SessionToken, SignInError> {
    let token = SessionToken::generate().inspect_err(|error| {
        tracing::error!(error = error as &dyn Error, "no session token could be generated");
    })?;
    let now = clock.now();
    let player = storage.add_or_refresh_player(identity, now).await?;
    tracing::Span::current().record("player.id", player.value());

    if let Some(presented) = presented {
        storage.end_session(&presented.hash()).await?;
    }
    storage.start_session(&token.hash(), player, now).await?;
    Ok(token)
}

/// Says who a request acts for, and counts the request as a use of its session.
///
/// A request with no token, an unknown one, or one last used 90 days ago or more is anonymous
/// (spec rule 3.2.4). Admin status is read from `admin_emails` each time, never stored (spec rule 3.3.4).
#[tracing::instrument(skip_all, fields(player.id = Empty))]
pub async fn authenticate<S: SessionStore>(
    presented: Option<&SessionToken>,
    storage: &S,
    clock: &dyn Clock,
    admin_emails: &AdminEmails,
) -> Result<Actor, StorageError> {
    let Some(presented) = presented else {
        return Ok(Actor::Anonymous);
    };
    let now = clock.now();
    let Some(player) = storage
        .use_session(&presented.hash(), now, live_if_used_after(now))
        .await?
    else {
        return Ok(Actor::Anonymous);
    };
    tracing::Span::current().record("player.id", player.id.value());

    Ok(if admin_emails.contains(&player.email) {
        Actor::Admin(player)
    } else {
        Actor::Player(player)
    })
}

/// The signed-in player an actor is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignedIn<'a> {
    pub player: &'a Player,
    pub is_admin: bool,
}

/// Tells a signed-in player who they are.
#[tracing::instrument(skip_all)]
pub fn who_am_i(actor: &Actor) -> Result<SignedIn<'_>, NotSignedIn> {
    Ok(SignedIn {
        player: actor.require_player()?,
        is_admin: actor.is_admin(),
    })
}

/// Why a sign-out did not happen.
#[derive(Debug, thiserror::Error)]
pub enum SignOutError {
    #[error(transparent)]
    NotSignedIn(#[from] NotSignedIn),
    #[error(transparent)]
    Storage(#[from] StorageError),
}

/// Ends every session of the signed-in player, on every device (spec rule 3.2.5).
#[tracing::instrument(skip_all)]
pub async fn sign_out<S: SessionStore>(actor: &Actor, storage: &S) -> Result<(), SignOutError> {
    let player = actor.require_player()?;
    storage.end_sessions_of(player.id).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use domain::player::{Player, PlayerId, PlayerName};

    use super::*;
    use crate::actor::Actor;

    fn player() -> Player {
        Player {
            id: PlayerId::new(7),
            email: "player@example.com".into(),
            name: PlayerName::NotChosen { suggested: None },
        }
    }

    #[test]
    fn who_am_i_answers_a_player_and_says_whether_they_are_an_admin() {
        let as_player = Actor::Player(player());
        let as_admin = Actor::Admin(player());

        assert_eq!(
            who_am_i(&as_player),
            Ok(SignedIn {
                player: &player(),
                is_admin: false
            })
        );
        assert_eq!(
            who_am_i(&as_admin),
            Ok(SignedIn {
                player: &player(),
                is_admin: true
            })
        );
    }

    #[test]
    fn who_am_i_refuses_an_actor_who_is_not_a_player() {
        assert_eq!(who_am_i(&Actor::Anonymous), Err(NotSignedIn));
        assert_eq!(who_am_i(&Actor::System), Err(NotSignedIn));
    }
}

//! The session use cases on real Postgres, with a clock the test sets.

use std::sync::Mutex;

use adapters::postgres::PostgresStorage;
use application::actor::Actor;
use application::admin_emails::AdminEmails;
use application::clock::Clock;
use application::session::{SignOutError, authenticate, sign_in, sign_out};
use application::session_token::SessionToken;
use domain::player::{DisplayName, IdentityClaims, Player, PlayerName, VerifiedIdentity};
use domain::time::UtcInstant;
use sqlx::PgPool;

const DAY_MILLIS: i64 = 24 * 60 * 60 * 1000;

/// A clock that shows the day the test last set.
struct FakeClock(Mutex<UtcInstant>);

impl FakeClock {
    fn on_day(day: i64) -> Self {
        Self(Mutex::new(UtcInstant::from_millis_since_epoch(day * DAY_MILLIS)))
    }

    fn go_to_day(&self, day: i64) {
        *self.0.lock().unwrap() = UtcInstant::from_millis_since_epoch(day * DAY_MILLIS);
    }
}

impl Clock for FakeClock {
    fn now(&self) -> UtcInstant {
        *self.0.lock().unwrap()
    }
}

fn identity(subject: &str, email: &str) -> VerifiedIdentity {
    VerifiedIdentity::from_claims(IdentityClaims {
        subject: subject.into(),
        email: email.into(),
        email_verified: true,
        name: Some("Pat Player".into()),
    })
    .unwrap()
}

fn pat() -> VerifiedIdentity {
    identity("subject-pat", "pat@example.com")
}

/// Signs in on a device that presents no earlier session.
async fn sign_in_on_a_new_device(who: &VerifiedIdentity, storage: &PostgresStorage, clock: &FakeClock) -> SessionToken {
    sign_in(who, None, storage, clock).await.unwrap()
}

/// Who a request carrying `token` acts for, when no admin is configured.
async fn actor_of(token: &SessionToken, storage: &PostgresStorage, clock: &FakeClock) -> Actor {
    authenticate(Some(token), storage, clock, &AdminEmails::none())
        .await
        .unwrap()
}

fn player_of(actor: Actor) -> Player {
    match actor {
        Actor::Player(player) | Actor::Admin(player) => player,
        other => panic!("expected a signed-in player, found {other:?}"),
    }
}

#[sqlx::test]
async fn rule_3_2_4_a_session_is_valid_on_day_89_and_refused_on_day_91(database: PgPool) {
    let storage = PostgresStorage::new(database);
    let clock = FakeClock::on_day(0);
    let used_on_day_89 = sign_in_on_a_new_device(&pat(), &storage, &clock).await;
    let never_used_again = sign_in_on_a_new_device(&pat(), &storage, &clock).await;

    clock.go_to_day(89);
    assert!(matches!(
        actor_of(&used_on_day_89, &storage, &clock).await,
        Actor::Player(_)
    ));

    clock.go_to_day(91);
    assert_eq!(actor_of(&never_used_again, &storage, &clock).await, Actor::Anonymous);
    // The 90 days count from the last use, which for this session was day 89.
    assert!(matches!(
        actor_of(&used_on_day_89, &storage, &clock).await,
        Actor::Player(_)
    ));
}

#[sqlx::test]
async fn rule_3_2_4_a_session_has_ended_at_exactly_90_days(database: PgPool) {
    let storage = PostgresStorage::new(database);
    let clock = FakeClock::on_day(0);
    let token = sign_in_on_a_new_device(&pat(), &storage, &clock).await;

    clock.go_to_day(90);

    assert_eq!(actor_of(&token, &storage, &clock).await, Actor::Anonymous);
}

#[sqlx::test]
async fn a_request_without_a_known_token_is_anonymous(database: PgPool) {
    let storage = PostgresStorage::new(database);
    let clock = FakeClock::on_day(0);
    let never_issued = SessionToken::generate();

    let without_a_token = authenticate(None, &storage, &clock, &AdminEmails::none())
        .await
        .unwrap();

    assert_eq!(without_a_token, Actor::Anonymous);
    assert_eq!(actor_of(&never_issued, &storage, &clock).await, Actor::Anonymous);
}

#[sqlx::test]
async fn rule_3_2_3_a_sign_in_ends_the_presented_session_and_no_other(database: PgPool) {
    let storage = PostgresStorage::new(database);
    let clock = FakeClock::on_day(0);
    let phone = sign_in_on_a_new_device(&pat(), &storage, &clock).await;
    let laptop_before = sign_in_on_a_new_device(&pat(), &storage, &clock).await;

    let laptop_after = sign_in(&pat(), Some(&laptop_before), &storage, &clock).await.unwrap();

    assert_eq!(actor_of(&laptop_before, &storage, &clock).await, Actor::Anonymous);
    assert!(matches!(
        actor_of(&laptop_after, &storage, &clock).await,
        Actor::Player(_)
    ));
    assert!(matches!(actor_of(&phone, &storage, &clock).await, Actor::Player(_)));
}

#[sqlx::test]
async fn rule_3_2_5_sign_out_ends_all_of_the_players_sessions(database: PgPool) {
    let storage = PostgresStorage::new(database);
    let clock = FakeClock::on_day(0);
    let pats_phone = sign_in_on_a_new_device(&pat(), &storage, &clock).await;
    let pats_laptop = sign_in_on_a_new_device(&pat(), &storage, &clock).await;
    let sam = identity("subject-sam", "sam@example.com");
    let sams_phone = sign_in_on_a_new_device(&sam, &storage, &clock).await;

    let pat_on_the_phone = actor_of(&pats_phone, &storage, &clock).await;
    sign_out(&pat_on_the_phone, &storage).await.unwrap();

    assert_eq!(actor_of(&pats_phone, &storage, &clock).await, Actor::Anonymous);
    assert_eq!(actor_of(&pats_laptop, &storage, &clock).await, Actor::Anonymous);
    assert!(matches!(
        actor_of(&sams_phone, &storage, &clock).await,
        Actor::Player(_)
    ));
}

/// The part of spec rule 3.3.1 that P3.2 can show; plan item P8.1 closes the rule.
#[sqlx::test]
async fn rule_3_3_1_sign_out_refuses_an_actor_who_is_not_a_player(database: PgPool) {
    let storage = PostgresStorage::new(database);

    let anonymous = sign_out(&Actor::Anonymous, &storage).await;
    let system = sign_out(&Actor::System, &storage).await;

    assert!(matches!(anonymous, Err(SignOutError::NotSignedIn(_))));
    assert!(matches!(system, Err(SignOutError::NotSignedIn(_))));
}

#[sqlx::test]
async fn a_first_sign_in_adds_a_player_without_a_display_name(database: PgPool) {
    let storage = PostgresStorage::new(database);
    let clock = FakeClock::on_day(0);

    let token = sign_in_on_a_new_device(&pat(), &storage, &clock).await;

    let player = player_of(actor_of(&token, &storage, &clock).await);
    assert_eq!(player.email, "pat@example.com");
    assert_eq!(
        player.name,
        PlayerName::NotChosen {
            suggested: Some("Pat Player".into())
        }
    );
}

/// What spec rule 3.1.4 asks of storage; plan item P3.4 owns the rule.
#[sqlx::test]
async fn a_later_sign_in_keeps_the_player_and_the_display_name_and_refreshes_the_email(database: PgPool) {
    let storage = PostgresStorage::new(database.clone());
    let clock = FakeClock::on_day(0);
    let first = sign_in_on_a_new_device(&pat(), &storage, &clock).await;
    sqlx::query("UPDATE players SET display_name = 'Pat', display_name_key = 'pat', suggested_name = NULL")
        .execute(&database)
        .await
        .unwrap();

    let with_a_new_email = identity("subject-pat", "pat@example.org");
    let second = sign_in_on_a_new_device(&with_a_new_email, &storage, &clock).await;

    let on_the_first_device = player_of(actor_of(&first, &storage, &clock).await);
    let on_the_second_device = player_of(actor_of(&second, &storage, &clock).await);
    assert_eq!(on_the_first_device, on_the_second_device);
    assert_eq!(on_the_second_device.email, "pat@example.org");
    assert_eq!(
        on_the_second_device.name,
        PlayerName::Chosen(DisplayName::parse("Pat").unwrap())
    );
}

/// Each call stands for a request under the configuration of the deploy that serves it (spec rule 3.3.4).
#[sqlx::test]
async fn rule_1_6_3_admin_rights_follow_the_configured_emails(database: PgPool) {
    let storage = PostgresStorage::new(database);
    let clock = FakeClock::on_day(0);
    let token = sign_in_on_a_new_device(&pat(), &storage, &clock).await;
    let pat_is_an_admin = AdminEmails::parse("someone@example.com,Pat@Example.com");
    let pat_is_no_admin = AdminEmails::parse("someone@example.com");

    let under_the_first = authenticate(Some(&token), &storage, &clock, &pat_is_an_admin)
        .await
        .unwrap();
    let under_the_second = authenticate(Some(&token), &storage, &clock, &pat_is_no_admin)
        .await
        .unwrap();

    assert!(matches!(under_the_first, Actor::Admin(_)));
    assert!(matches!(under_the_second, Actor::Player(_)));
}

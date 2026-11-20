//! Standings: scores, the leaderboard and the should-win statistics.
//!
//! Everything here is computed from ballots and results on each read; nothing is stored (spec rule 1.4.2).

use crate::ballot::{Ballot, PickKind};
use crate::player::DisplayName;
use crate::results::{Results, WinnerSet};
use crate::season::{CategoryKey, NomineeKey, Season};

/// How a will-win pick fared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickOutcome {
    Correct,
    Missed,
    /// Its category has no result yet.
    Pending,
}

/// The outcome of a will-win `pick`, given the winners of its category, if they are known.
/// With several winners, matching any of them is correct.
pub fn pick_outcome(pick: &NomineeKey, winners: Option<&WinnerSet>) -> PickOutcome {
    match winners {
        None => PickOutcome::Pending,
        Some(winners) if winners.contains(pick) => PickOutcome::Correct,
        Some(_) => PickOutcome::Missed,
    }
}

/// One point for every category of `season` whose will-win pick is among its winners.
/// Should-win picks never score (spec rule 1.4.1).
pub fn score(season: &Season, ballot: &Ballot, results: &Results) -> u32 {
    let correct_picks = season
        .categories()
        .iter()
        .filter(|category| {
            ballot
                .pick(&category.key, PickKind::WillWin)
                .is_some_and(|pick| pick_outcome(pick, results.winners(&category.key)) == PickOutcome::Correct)
        })
        .count();
    u32::try_from(correct_picks).expect("a season has far fewer categories than u32::MAX")
}

/// A player as the leaderboard needs them. `Id` is whatever the caller uses to tell players apart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contestant<Id> {
    pub id: Id,
    pub display_name: DisplayName,
    pub ballot: Ballot,
}

/// One row of the leaderboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Standing<Id> {
    pub rank: u32,
    pub id: Id,
    pub display_name: DisplayName,
    pub score: u32,
}

/// The leaderboard of `season` (spec rules 1.4.3 and 1.4.4).
///
/// It lists every contestant with at least one will-win pick, and no other.
/// A rank is one more than the number of listed players with a higher score, so ties share a rank
/// and the next rank is skipped: 1, 1, 3.
/// Rows run from the highest score down; within a score, by display name ignoring case.
pub fn leaderboard<Id>(
    season: &Season,
    results: &Results,
    contestants: impl IntoIterator<Item = Contestant<Id>>,
) -> Vec<Standing<Id>> {
    let mut scored: Vec<(u32, String, Contestant<Id>)> = contestants
        .into_iter()
        .filter(|contestant| contestant.ballot.has_a_will_win_pick())
        .map(|contestant| {
            (
                score(season, &contestant.ballot, results),
                contestant.display_name.uniqueness_key(),
                contestant,
            )
        })
        .collect();
    scored.sort_by(|(score, name, _), (other_score, other_name, _)| {
        other_score.cmp(score).then_with(|| name.cmp(other_name))
    });

    let mut standings: Vec<Standing<Id>> = Vec::with_capacity(scored.len());
    for (position, (score, _, contestant)) in scored.into_iter().enumerate() {
        // Rows are sorted by score, so everyone above with a different score has a higher one.
        let rank = match standings.last() {
            Some(above) if above.score == score => above.rank,
            _ => u32::try_from(position + 1).expect("a leaderboard has far fewer rows than u32::MAX"),
        };
        standings.push(Standing {
            rank,
            id: contestant.id,
            display_name: contestant.display_name,
            score,
        });
    }
    standings
}

/// How many players wish each nominee of one category would win.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShouldWinCounts {
    pub category: CategoryKey,
    /// Every nominee of the category with its count, zeros included, from the highest count down;
    /// equal counts keep seed order (spec rule 4.4.5).
    pub counts: Vec<(NomineeKey, u32)>,
}

/// The should-win statistics of `season` over `ballots`, one entry per category in seed order
/// (spec rule 1.5.1). A pick that names no nominee of its category is not counted.
pub fn should_win_statistics<'a>(
    season: &Season,
    ballots: impl IntoIterator<Item = &'a Ballot>,
) -> Vec<ShouldWinCounts> {
    let ballots: Vec<&Ballot> = ballots.into_iter().collect();
    season
        .categories()
        .iter()
        .map(|category| {
            let mut counts: Vec<(NomineeKey, u32)> = category
                .nominees
                .iter()
                .map(|nominee| {
                    let wishes = ballots
                        .iter()
                        .filter(|ballot| ballot.pick(&category.key, PickKind::ShouldWin) == Some(&nominee.key))
                        .count();
                    let wishes = u32::try_from(wishes).expect("a season has far fewer players than u32::MAX");
                    (nominee.key.clone(), wishes)
                })
                .collect();
            // A stable sort: nominees with equal counts stay in seed order.
            counts.sort_by_key(|(_, wishes)| std::cmp::Reverse(*wishes));
            ShouldWinCounts {
                category: category.key.clone(),
                counts,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::fixtures::{LOCK, category_key, nominee_key, season};

    fn ballot(picks: &[(&str, PickKind, &str)]) -> Ballot {
        let mut ballot = Ballot::default();
        for (category, kind, nominee) in picks {
            ballot.write(category_key(category), *kind, Some(nominee_key(nominee)));
        }
        ballot
    }

    fn results(season: &Season, winners: &[(&str, &[&str])]) -> Results {
        let mut results = Results::default();
        for (category, nominees) in winners {
            let category = category_key(category);
            let winners = season
                .winner_set(LOCK, &category, nominees.iter().map(|nominee| nominee_key(nominee)))
                .unwrap();
            results.record(category, winners);
        }
        results
    }

    fn contestant(id: u32, name: &str, ballot: Ballot) -> Contestant<u32> {
        Contestant {
            id,
            display_name: DisplayName::parse(name).unwrap(),
            ballot,
        }
    }

    fn three_categories() -> Season {
        season(&[
            ("picture", &["a", "b"]),
            ("director", &["c", "d"]),
            ("actor", &["e", "f"]),
        ])
    }

    #[test]
    fn rule_1_4_1_a_will_win_pick_scores_one_when_it_is_among_the_winners() {
        let season = three_categories();
        let ballot = ballot(&[
            ("picture", PickKind::WillWin, "a"),
            ("director", PickKind::WillWin, "c"),
            ("actor", PickKind::WillWin, "e"),
        ]);
        // A tie in picture, a miss in director, and no result yet for actor.
        let results = results(&season, &[("picture", &["a", "b"]), ("director", &["d"])]);

        assert_eq!(score(&season, &ballot, &results), 1);
    }

    #[test]
    fn rule_1_4_1_a_should_win_pick_never_scores() {
        let season = three_categories();
        let ballot = ballot(&[("picture", PickKind::ShouldWin, "a")]);
        let results = results(&season, &[("picture", &["a"])]);

        assert_eq!(score(&season, &ballot, &results), 0);
    }

    /// The part of spec rule 1.2.10 that needs no page; plan item P8.3 closes the rule.
    #[test]
    fn a_will_win_pick_is_correct_missed_or_pending() {
        let season = three_categories();
        let results = results(&season, &[("picture", &["a", "b"])]);
        let winners = results.winners(&category_key("picture"));

        assert_eq!(pick_outcome(&nominee_key("b"), winners), PickOutcome::Correct);
        assert_eq!(pick_outcome(&nominee_key("z"), winners), PickOutcome::Missed);
        assert_eq!(pick_outcome(&nominee_key("c"), None), PickOutcome::Pending);
    }

    #[test]
    fn rule_1_4_3_tied_scores_share_a_rank_and_list_by_name_ignoring_case() {
        let season = three_categories();
        let results = results(&season, &[("picture", &["a"]), ("director", &["c"])]);
        let two_right = || {
            ballot(&[
                ("picture", PickKind::WillWin, "a"),
                ("director", PickKind::WillWin, "c"),
            ])
        };
        let one_right = ballot(&[("picture", PickKind::WillWin, "a")]);

        let board = leaderboard(
            &season,
            &results,
            [
                contestant(1, "Zoe", one_right),
                contestant(2, "Bert", two_right()),
                contestant(3, "anna", two_right()),
            ],
        );

        let rows: Vec<(u32, &str, u32)> = board
            .iter()
            .map(|standing| (standing.rank, standing.display_name.as_str(), standing.score))
            .collect();
        // Sorted by character code, "Bert" would come before "anna".
        assert_eq!(rows, [(1, "anna", 2), (1, "Bert", 2), (3, "Zoe", 1)]);
        assert_eq!(board[0].id, 3);
    }

    #[test]
    fn rule_1_4_4_a_player_without_a_will_win_pick_is_not_listed() {
        let season = three_categories();
        let results = Results::default();

        let board = leaderboard(
            &season,
            &results,
            [
                contestant(1, "Only favorites", ballot(&[("picture", PickKind::ShouldWin, "a")])),
                contestant(2, "No picks", Ballot::default()),
                contestant(3, "Predictor", ballot(&[("picture", PickKind::WillWin, "b")])),
            ],
        );

        assert_eq!(board.iter().map(|standing| standing.id).collect::<Vec<_>>(), [3]);
        assert_eq!((board[0].rank, board[0].score), (1, 0));
    }

    #[test]
    fn rule_1_5_1_should_win_picks_are_counted_per_nominee() {
        let season = season(&[("picture", &["a", "b", "c"]), ("director", &["d"])]);
        let ballots = [
            ballot(&[
                ("picture", PickKind::ShouldWin, "b"),
                ("picture", PickKind::WillWin, "a"),
            ]),
            ballot(&[("picture", PickKind::ShouldWin, "b")]),
            ballot(&[("picture", PickKind::ShouldWin, "c")]),
            ballot(&[("picture", PickKind::ShouldWin, "gone")]),
        ];

        let statistics = should_win_statistics(&season, &ballots);

        assert_eq!(
            statistics,
            [
                ShouldWinCounts {
                    category: category_key("picture"),
                    counts: vec![(nominee_key("b"), 2), (nominee_key("c"), 1), (nominee_key("a"), 0)],
                },
                ShouldWinCounts {
                    category: category_key("director"),
                    counts: vec![(nominee_key("d"), 0)],
                },
            ]
        );
    }

    #[test]
    fn rule_4_4_5_nominees_with_equal_counts_keep_seed_order() {
        let season = season(&[("picture", &["c", "b", "a"])]);
        let ballots = [
            ballot(&[("picture", PickKind::ShouldWin, "a")]),
            ballot(&[("picture", PickKind::ShouldWin, "b")]),
        ];

        let statistics = should_win_statistics(&season, &ballots);

        // "b" and "a" tie at one each and stay as seeded; by key, "a" would come first.
        let order: Vec<&str> = statistics[0]
            .counts
            .iter()
            .map(|(nominee, _)| nominee.as_str())
            .collect();
        assert_eq!(order, ["b", "a", "c"]);
    }

    // Property tests (spec rule 6.6.2), over a season of four categories with three nominees each.

    const CATEGORIES: usize = 4;
    const NOMINEES: usize = 3;

    fn generated_season() -> Season {
        let keys: Vec<(String, Vec<String>)> = (0..CATEGORIES)
            .map(|category| {
                (
                    format!("category-{category}"),
                    (0..NOMINEES).map(|nominee| format!("nominee-{nominee}")).collect(),
                )
            })
            .collect();
        let borrowed: Vec<(&str, Vec<&str>)> = keys
            .iter()
            .map(|(category, nominees)| (category.as_str(), nominees.iter().map(String::as_str).collect()))
            .collect();
        let categories: Vec<(&str, &[&str])> = borrowed
            .iter()
            .map(|(category, nominees)| (*category, nominees.as_slice()))
            .collect();
        season(&categories)
    }

    /// Per category, the nominee picked to win and the nominee wished to win, each by position.
    type GeneratedPicks = Vec<(Option<usize>, Option<usize>)>;
    /// Per category, which nominees won, by position. No winner means no result yet.
    type GeneratedWinners = Vec<Vec<bool>>;

    fn picks() -> impl Strategy<Value = GeneratedPicks> {
        let pick = || proptest::option::of(0..NOMINEES);
        proptest::collection::vec((pick(), pick()), CATEGORIES)
    }

    fn winners() -> impl Strategy<Value = GeneratedWinners> {
        proptest::collection::vec(proptest::collection::vec(any::<bool>(), NOMINEES), CATEGORIES)
    }

    fn ballot_of(season: &Season, picks: &GeneratedPicks) -> Ballot {
        let mut ballot = Ballot::default();
        for (category, (will_win, should_win)) in season.categories().iter().zip(picks) {
            let key_at = |position: &Option<usize>| position.map(|position| category.nominees[position].key.clone());
            ballot.write(category.key.clone(), PickKind::WillWin, key_at(will_win));
            ballot.write(category.key.clone(), PickKind::ShouldWin, key_at(should_win));
        }
        ballot
    }

    fn results_of(season: &Season, winners: &GeneratedWinners) -> Results {
        let mut results = Results::default();
        for (category, won) in season.categories().iter().zip(winners) {
            let keys = category
                .nominees
                .iter()
                .zip(won)
                .filter(|(_, won)| **won)
                .map(|(nominee, _)| nominee.key.clone());
            if let Ok(winner_set) = season.winner_set(LOCK, &category.key, keys) {
                results.record(category.key.clone(), winner_set);
            }
        }
        results
    }

    proptest! {
        #[test]
        fn rule_1_4_1_the_score_is_the_number_of_categories_whose_pick_won(picks in picks(), winners in winners()) {
            let season = generated_season();
            let matching = picks
                .iter()
                .zip(&winners)
                .filter(|((will_win, _), won)| will_win.is_some_and(|position| won[position]))
                .count();

            let score = score(&season, &ballot_of(&season, &picks), &results_of(&season, &winners));

            prop_assert_eq!(score as usize, matching);
        }

        #[test]
        fn rule_1_4_1_should_win_picks_do_not_change_the_score(picks in picks(), winners in winners()) {
            let season = generated_season();
            let results = results_of(&season, &winners);
            let without_favorites: GeneratedPicks = picks.iter().map(|(will_win, _)| (*will_win, None)).collect();

            prop_assert_eq!(
                score(&season, &ballot_of(&season, &picks), &results),
                score(&season, &ballot_of(&season, &without_favorites), &results)
            );
        }

        #[test]
        fn rule_1_4_3_a_rank_is_one_more_than_the_number_of_higher_scores(
            ballots in proptest::collection::vec(picks(), 0..12),
            winners in winners(),
        ) {
            let season = generated_season();
            let results = results_of(&season, &winners);
            let contestants: Vec<Contestant<usize>> = ballots
                .iter()
                .enumerate()
                .map(|(id, picks)| Contestant {
                    id,
                    display_name: DisplayName::parse(&format!("Player {id:02}")).unwrap(),
                    ballot: ballot_of(&season, picks),
                })
                .collect();
            let predictors = ballots
                .iter()
                .filter(|picks| picks.iter().any(|(will_win, _)| will_win.is_some()))
                .count();

            let board = leaderboard(&season, &results, contestants.clone());

            // Rule 1.4.4: exactly the players with a will-win pick.
            prop_assert_eq!(board.len(), predictors);
            for standing in &board {
                let higher = board.iter().filter(|other| other.score > standing.score).count();
                prop_assert_eq!(standing.rank as usize, higher + 1);
                prop_assert_eq!(standing.score, score(&season, &contestants[standing.id].ballot, &results));
            }
            // From the highest score down, and by name within a score.
            for pair in board.windows(2) {
                let in_order = pair[0].score > pair[1].score
                    || (pair[0].score == pair[1].score
                        && pair[0].display_name.uniqueness_key() < pair[1].display_name.uniqueness_key());
                prop_assert!(in_order);
            }
        }
    }
}

use rand::{rngs::StdRng, RngExt, SeedableRng};
use std::collections::{BTreeMap, BTreeSet};
use vote::{
    borda, plurality, random_dictator, random_dictator_with_rng, Preference, PreferenceError,
    Results, ScoreGroup,
};

#[test]
fn rejects_empty_profiles_and_ballots() {
    assert_eq!(
        Preference::<u8>::new(vec![]),
        Err(PreferenceError::EmptyProfile)
    );
    for (ballots, ballot) in [(vec![vec![]], 0), (vec![vec![0], vec![]], 1)] {
        assert_eq!(
            Preference::new(ballots),
            Err(PreferenceError::EmptyBallot { ballot })
        );
    }
}

#[test]
fn rejects_extra_duplicate_that_previously_passed_audit() {
    assert_eq!(
        Preference::new(vec![vec![0, 1], vec![0, 1, 1]]),
        Err(PreferenceError::BallotLengthMismatch {
            ballot: 1,
            expected: 2,
            actual: 3
        })
    );
}

#[test]
fn rejects_incomplete_ballots() {
    assert_eq!(
        Preference::new(vec![vec![0, 1, 2], vec![0, 1]]),
        Err(PreferenceError::BallotLengthMismatch {
            ballot: 1,
            expected: 3,
            actual: 2
        })
    );
}

#[test]
fn rejects_duplicates_in_first_or_later_ballots() {
    for (ballots, ballot) in [
        (vec![vec![0, 1, 1]], 0),
        (vec![vec![0, 1, 2], vec![0, 1, 1]], 1),
    ] {
        assert_eq!(
            Preference::new(ballots),
            Err(PreferenceError::DuplicateCandidate { ballot, rank: 2 })
        );
    }
}

#[test]
fn rejects_mismatched_candidate_sets() {
    assert_eq!(
        Preference::new(vec![vec!['a', 'b'], vec!['a', 'c']]),
        Err(PreferenceError::UnknownCandidate { ballot: 1, rank: 1 })
    );
}

#[test]
fn error_can_be_reported_without_panicking() {
    let error = Preference::<u8>::new(vec![]).unwrap_err();
    let boxed: Box<dyn std::error::Error> = Box::new(error);
    assert!(boxed.to_string().contains("at least one voter"));
}

#[test]
fn validates_non_clone_candidates_and_round_trips_ballots() {
    #[derive(Debug, Eq, PartialEq, Hash)]
    struct Candidate(u8);
    let ballots = vec![vec![Candidate(0), Candidate(1)]];
    let pref = Preference::try_from(ballots).unwrap();
    assert_eq!(pref.voter_count(), 1);
    assert_eq!(pref.candidate_count(), 2);
    assert_eq!(pref.candidates(), &[Candidate(0), Candidate(1)]);
    assert_eq!(pref.into_ballots(), vec![vec![Candidate(0), Candidate(1)]]);
}

#[test]
fn votes_with_candidates_without_ord_or_debug() {
    #[derive(Clone, Eq, PartialEq, Hash)]
    enum Candidate {
        A,
        B,
    }
    let pref = Preference::new(vec![vec![Candidate::B, Candidate::A]; 2]).unwrap();
    assert!(plurality(&pref).winners() == [Candidate::B]);
    assert!(borda(&pref).winners() == [Candidate::B]);
    assert!(random_dictator(&pref) == [Candidate::B, Candidate::A]);
}

#[test]
fn retains_existing_plurality_ranking_with_exact_scores() {
    let pref = Preference::new(vec![
        vec![0, 1, 2, 3],
        vec![0, 1, 2, 3],
        vec![0, 1, 2, 3],
        vec![1, 0, 2, 3],
        vec![1, 0, 2, 3],
        vec![2, 1, 0, 3],
    ])
    .unwrap();
    assert_eq!(
        plurality(&pref).groups(),
        &[
            ScoreGroup {
                score: 3,
                candidates: vec![0]
            },
            ScoreGroup {
                score: 2,
                candidates: vec![1]
            },
            ScoreGroup {
                score: 1,
                candidates: vec![2]
            },
            ScoreGroup {
                score: 0,
                candidates: vec![3]
            },
        ]
    );
}

#[test]
fn retains_existing_borda_ranking_with_standard_scores() {
    let pref = Preference::new(vec![
        vec![3, 1, 2, 0],
        vec![0, 1, 2, 3],
        vec![0, 1, 2, 3],
        vec![3, 0, 1, 2],
    ])
    .unwrap();
    assert_eq!(
        borda(&pref).groups(),
        &[
            ScoreGroup {
                score: 8,
                candidates: vec![0]
            },
            ScoreGroup {
                score: 7,
                candidates: vec![1]
            },
            ScoreGroup {
                score: 6,
                candidates: vec![3]
            },
            ScoreGroup {
                score: 3,
                candidates: vec![2]
            },
        ]
    );
}

#[test]
fn identical_tied_profiles_have_identical_results() {
    for _ in 0..100 {
        let pref = Preference::new(vec![vec!['b', 'a'], vec!['a', 'b']]).unwrap();
        for result in [plurality(&pref), borda(&pref)] {
            assert_eq!(
                result.groups(),
                &[ScoreGroup {
                    score: 1,
                    candidates: vec!['b', 'a']
                }]
            );
            assert_eq!(result.winners(), &['b', 'a']);
            assert_eq!(result.ranking_by(Ord::cmp), vec!['a', 'b']);
            assert_eq!(result.ranking_by(|a, b| b.cmp(a)), vec!['b', 'a']);
        }
    }
}

#[test]
fn tie_breaking_does_not_reorder_different_scores_or_change_groups() {
    let pref = Preference::new(vec![vec!['a', 'c', 'b']; 2]).unwrap();
    let result = plurality(&pref);
    assert_eq!(result.ranking_by(|a, b| b.cmp(a)), vec!['a', 'c', 'b']);
    assert_eq!(result.ranking_by(Ord::cmp), vec!['a', 'b', 'c']);
    assert_eq!(result.groups()[1].candidates, vec!['c', 'b']);
    assert_eq!(result.winners(), &['a']);
    assert_eq!(result.clone().into_groups(), result.groups());
}

#[test]
fn single_candidate_has_a_winner_even_with_zero_borda_points() {
    let pref = Preference::new(vec![vec![42]; 3]).unwrap();
    assert_eq!(
        plurality(&pref).groups(),
        &[ScoreGroup {
            score: 3,
            candidates: vec![42]
        }]
    );
    assert_eq!(
        borda(&pref).groups(),
        &[ScoreGroup {
            score: 0,
            candidates: vec![42]
        }]
    );
    assert_eq!(borda(&pref).winners(), &[42]);
    assert_eq!(random_dictator(&pref), vec![42]);
}

#[test]
fn random_dictator_reproduces_sequences_from_a_supplied_seed() {
    let pref = Preference::new(vec![vec![0, 1, 2], vec![1, 2, 0], vec![2, 0, 1]]).unwrap();
    let sample = || {
        let mut rng = StdRng::seed_from_u64(42);
        (0..64)
            .map(|_| random_dictator_with_rng(&pref, &mut rng))
            .collect::<Vec<_>>()
    };
    let selections = sample();
    assert_eq!(selections, sample());
    assert_eq!(selections.iter().collect::<BTreeSet<_>>().len(), 3);
    assert!(selections
        .iter()
        .all(|ballot| pref.ballots().contains(ballot)));
}

#[test]
fn random_dictator_samples_voters_including_repeated_ballots() {
    let ballots = vec![vec![0, 1], vec![0, 1], vec![1, 0]];
    let pref = Preference::new(ballots.clone()).unwrap();
    let mut expected_rng = StdRng::seed_from_u64(17);
    let mut actual_rng = StdRng::seed_from_u64(17);
    // Using a trait object also verifies the public RNG parameter supports ?Sized.
    let actual_rng: &mut dyn rand::Rng = &mut actual_rng;
    for _ in 0..64 {
        let index = expected_rng.random_range(0..ballots.len());
        assert_eq!(random_dictator_with_rng(&pref, actual_rng), ballots[index]);
    }
    assert_eq!(pref.ballots(), ballots);
}

fn score_map(result: &Results<u8>) -> BTreeMap<u8, u128> {
    result
        .groups()
        .iter()
        .flat_map(|group| {
            group
                .candidates
                .iter()
                .map(move |&candidate| (candidate, group.score))
        })
        .collect()
}

#[test]
fn exhaustively_checks_small_elections_and_voter_order_invariance() {
    let permutations = [
        vec![0, 1, 2],
        vec![0, 2, 1],
        vec![1, 0, 2],
        vec![1, 2, 0],
        vec![2, 0, 1],
        vec![2, 1, 0],
    ];
    // All 1,554 complete profiles with three candidates and one to four voters.
    for voters in 1..=4 {
        for mut code in 0..6_usize.pow(voters) {
            let ballots: Vec<_> = (0..voters)
                .map(|_| {
                    let ballot = permutations[code % 6].clone();
                    code /= 6;
                    ballot
                })
                .collect();
            let pref = Preference::new(ballots.clone()).unwrap();
            let mut reversed = ballots.clone();
            reversed.reverse();
            let reversed = Preference::new(reversed).unwrap();
            for (result, reordered, is_borda) in [
                (plurality(&pref), plurality(&reversed), false),
                (borda(&pref), borda(&reversed), true),
            ] {
                let scores = score_map(&result);
                assert_eq!(scores.len(), 3);
                assert_eq!(scores, score_map(&reordered));
                assert_eq!(result.ranking_by(Ord::cmp), reordered.ranking_by(Ord::cmp));
                assert_eq!(
                    scores.values().sum::<u128>(),
                    u128::from(voters) * if is_borda { 3 } else { 1 }
                );
                assert!(result
                    .groups()
                    .windows(2)
                    .all(|pair| pair[0].score > pair[1].score));
                for candidate in 0..3 {
                    // Count how many alternatives each voter ranks below this
                    // candidate, or count first-place votes for plurality.
                    let expected: usize = ballots
                        .iter()
                        .map(|ballot| {
                            if is_borda {
                                ballot
                                    .iter()
                                    .skip_while(|&&item| item != candidate)
                                    .skip(1)
                                    .count()
                            } else {
                                usize::from(ballot[0] == candidate)
                            }
                        })
                        .sum();
                    assert_eq!(scores[&candidate], expected as u128);
                }
            }
        }
    }
}

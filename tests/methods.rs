use std::cmp::Reverse;
use std::collections::BTreeMap;
use vote::{
    condorcet_winner, copeland, instant_runoff, instant_runoff_by, pairwise, plurality, Preference,
    Results, RunoffOutcome, RunoffResults,
};

fn profile(rows: &[(usize, &[u8])]) -> Preference<u8> {
    Preference::new(
        rows.iter()
            .flat_map(|(count, ballot)| vec![ballot.to_vec(); *count])
            .collect(),
    )
    .unwrap()
}

fn scores(results: &Results<u8>) -> BTreeMap<u8, u128> {
    results
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
fn pairwise_counts_have_explicit_orientation_and_preserve_voter_multiplicity() {
    // Candidate 1 is the first matrix row, not candidate 0.
    let pref = profile(&[(2, &[1, 0, 2]), (1, &[0, 2, 1])]);
    let result = pairwise(&pref);
    assert_eq!(result.candidates(), &[1, 0, 2]);
    assert_eq!(
        result.counts(),
        &[vec![0, 2, 2], vec![1, 0, 3], vec![1, 0, 0]]
    );
    assert_eq!(result.preference_count(&0, &1), Some(1));
    assert_eq!(result.preference_count(&1, &0), Some(2));
    assert_eq!(result.preference_count(&1, &1), Some(0));
    assert_eq!(result.preference_count(&9, &1), None);
    assert_eq!(result.preference_count(&1, &9), None);
    assert_eq!(result.condorcet_winner(), Some(&1));
    assert_eq!(result.copeland(), copeland(&pref));
}

#[test]
fn condorcet_and_runoff_can_disagree_with_plurality() {
    let pref = profile(&[(4, &[0, 1, 2]), (3, &[1, 2, 0]), (2, &[2, 1, 0])]);
    assert_eq!(plurality(&pref).winners(), &[0]);
    assert_eq!(condorcet_winner(&pref), Some(1));
    assert_eq!(
        scores(&copeland(&pref)),
        BTreeMap::from([(0, 0), (1, 4), (2, 2)])
    );
    let result = instant_runoff(&pref);
    assert_eq!(result.winner(), Some(&1));
    assert_eq!(result.rounds().len(), 2);
    assert_eq!(result.rounds()[0].eliminated, Some(2));
    assert_eq!(
        scores(&result.rounds()[1].tallies),
        BTreeMap::from([(0, 4), (1, 5)])
    );
}

#[test]
fn a_condorcet_cycle_has_no_strict_winner_and_keeps_copeland_ties() {
    let pref = profile(&[(1, &[0, 1, 2]), (1, &[1, 2, 0]), (1, &[2, 0, 1])]);
    let comparisons = pairwise(&pref);
    assert_eq!(
        comparisons.counts(),
        &[vec![0, 2, 1], vec![1, 0, 2], vec![2, 1, 0]]
    );
    assert_eq!(comparisons.condorcet_winner(), None);
    assert_eq!(condorcet_winner(&pref), None);
    assert_eq!(copeland(&pref).winners(), &[0, 1, 2]);
    assert_eq!(
        scores(&copeland(&pref)),
        BTreeMap::from([(0, 2), (1, 2), (2, 2)])
    );
}

#[test]
fn even_electorates_score_pairwise_ties_without_claiming_condorcet_winners() {
    let pref = profile(&[(1, &[0, 1, 2]), (1, &[2, 1, 0])]);
    assert_eq!(condorcet_winner(&pref), None);
    assert_eq!(
        scores(&copeland(&pref)),
        BTreeMap::from([(0, 2), (1, 2), (2, 2)])
    );
}

#[test]
fn all_methods_handle_a_single_candidate() {
    let pref = profile(&[(3, &[9])]);
    assert_eq!(pairwise(&pref).counts(), &[vec![0]]);
    assert_eq!(condorcet_winner(&pref), Some(9));
    assert_eq!(scores(&copeland(&pref)), BTreeMap::from([(9, 0)]));
    let result = instant_runoff(&pref);
    assert_eq!(result.winner(), Some(&9));
    assert_eq!(result.rounds().len(), 1);
    assert_eq!(result.rounds()[0].eliminated, None);
    assert_eq!(
        scores(&result.rounds()[0].tallies),
        BTreeMap::from([(9, 3)])
    );
}

#[test]
fn a_unique_copeland_winner_need_not_be_a_strict_condorcet_winner() {
    let pref = profile(&[(2, &[0, 1, 2]), (1, &[2, 0, 1]), (1, &[2, 1, 0])]);
    assert_eq!(condorcet_winner(&pref), None);
    assert_eq!(copeland(&pref).winners(), &[0]);
    assert_eq!(
        scores(&copeland(&pref)),
        BTreeMap::from([(0, 3), (1, 1), (2, 2)])
    );
}

#[test]
fn majority_wins_before_considering_a_loser_tie() {
    let pref = profile(&[(3, &[0, 1, 2]), (1, &[1, 2, 0]), (1, &[2, 0, 1])]);
    let result = instant_runoff_by(&pref, |_, _| panic!("no tie-break needed"));
    assert_eq!(result.winner(), Some(&0));
    assert_eq!(result.rounds().len(), 1);
    assert_eq!(result.rounds()[0].eliminated, None);
}

#[test]
fn exactly_half_is_not_a_majority() {
    let pref = profile(&[(2, &[0, 1, 2]), (1, &[1, 0, 2]), (1, &[2, 0, 1])]);
    let result = instant_runoff(&pref);
    assert_eq!(result.winner(), None);
    assert_eq!(result.outcome(), &RunoffOutcome::EliminationTie(vec![1, 2]));
}

#[test]
fn unresolved_elimination_candidates_are_not_reported_as_winners() {
    let pref = profile(&[(3, &[0, 1, 2]), (2, &[1, 2, 0]), (2, &[2, 1, 0])]);
    let result = instant_runoff(&pref);
    assert_eq!(result.outcome(), &RunoffOutcome::EliminationTie(vec![1, 2]));
    assert_eq!(result.winner(), None);
    assert_eq!(result.rounds()[0].tallies.winners(), &[0]); // This round's leader only.
    assert_eq!(result.rounds()[0].eliminated, None);
    assert_eq!(instant_runoff_by(&pref, Ord::cmp).winner(), Some(&1));
    assert_eq!(instant_runoff_by(&pref, |a, b| b.cmp(a)).winner(), Some(&2));
}

#[test]
fn final_two_candidate_tie_requires_an_explicit_policy() {
    let pref = profile(&[(1, &[1, 0]), (1, &[0, 1])]);
    assert_eq!(
        instant_runoff(&pref).outcome(),
        &RunoffOutcome::EliminationTie(vec![1, 0])
    );
    let resolved = instant_runoff_by(&pref, Ord::cmp);
    assert_eq!(resolved.winner(), Some(&0));
    assert_eq!(resolved.rounds()[0].eliminated, Some(1));
    assert_eq!(
        scores(&resolved.rounds()[1].tallies),
        BTreeMap::from([(0, 2)])
    );
}

#[test]
fn zero_vote_ties_are_explicit_and_only_one_candidate_is_removed_per_round() {
    let pref = profile(&[(2, &[0, 1, 2, 3]), (2, &[1, 0, 2, 3])]);
    assert_eq!(
        instant_runoff(&pref).outcome(),
        &RunoffOutcome::EliminationTie(vec![2, 3])
    );
    let result = instant_runoff_by(&pref, Ord::cmp);
    assert_eq!(result.winner(), Some(&0));
    assert_eq!(
        result
            .rounds()
            .iter()
            .map(|r| r.eliminated)
            .collect::<Vec<_>>(),
        vec![Some(3), Some(2), Some(1), None]
    );
    for (index, round) in result.rounds().iter().enumerate() {
        assert_eq!(scores(&round.tallies).len(), 4 - index);
        assert_eq!(scores(&round.tallies).values().sum::<u128>(), 4);
    }
}

#[test]
fn an_incomplete_tie_policy_reports_only_the_remaining_lowest_priority_group() {
    let pref = profile(&[
        (1, &[0, 1, 2, 3]),
        (1, &[1, 2, 3, 0]),
        (1, &[2, 3, 0, 1]),
        (1, &[3, 0, 1, 2]),
    ]);
    let result = instant_runoff_by(&pref, |a, b| u8::from(*a != 0).cmp(&u8::from(*b != 0)));
    assert_eq!(result.winner(), None);
    assert_eq!(
        result.outcome(),
        &RunoffOutcome::EliminationTie(vec![1, 2, 3])
    );
    assert_eq!(result.rounds()[0].eliminated, None);
}

#[test]
fn transfers_skip_preferences_that_were_eliminated_in_earlier_rounds() {
    let pref = profile(&[
        (4, &[0, 1, 2, 3]),
        (3, &[1, 0, 2, 3]),
        (2, &[2, 3, 1, 0]),
        (1, &[3, 1, 2, 0]),
    ]);
    let before = pref.ballots().to_vec();
    let result = instant_runoff(&pref);
    assert_eq!(result.winner(), Some(&1));
    assert_eq!(
        result
            .rounds()
            .iter()
            .map(|r| r.eliminated)
            .collect::<Vec<_>>(),
        vec![Some(3), Some(2), None]
    );
    assert_eq!(
        scores(&result.rounds()[2].tallies),
        BTreeMap::from([(0, 4), (1, 6)])
    );
    assert_eq!(pref.ballots(), before);
}

#[test]
fn reproduces_aec_preference_transfer_example() {
    // AEC's Nick/Michael/Jenny example, scaled from 60,000 voters to 600.
    // https://results.aec.gov.au/12246/voting.htm
    let pref = profile(&[
        (63, &[0, 1, 2]),
        (87, &[0, 2, 1]),
        (230, &[1, 0, 2]),
        (220, &[2, 0, 1]),
    ]);
    let result = instant_runoff(&pref);
    assert_eq!(
        scores(&result.rounds()[0].tallies),
        BTreeMap::from([(0, 150), (1, 230), (2, 220)])
    );
    assert_eq!(result.rounds()[0].eliminated, Some(0));
    assert_eq!(
        scores(&result.rounds()[1].tallies),
        BTreeMap::from([(1, 293), (2, 307)])
    );
    assert_eq!(result.winner(), Some(&2));
}

#[test]
fn candidates_do_not_need_ord_or_debug() {
    #[derive(Clone, PartialEq, Eq, Hash)]
    enum Candidate {
        A,
        B,
    }
    let pref = Preference::new(vec![vec![Candidate::B, Candidate::A]; 3]).unwrap();
    assert!(condorcet_winner(&pref) == Some(Candidate::B));
    assert!(pairwise(&pref).condorcet_winner() == Some(&Candidate::B));
    assert!(copeland(&pref).winners() == [Candidate::B]);
    assert!(instant_runoff(&pref).winner() == Some(&Candidate::B));
}

type Trace = Vec<(BTreeMap<u8, u128>, Option<u8>)>;

// An independent reference physically removes eliminated candidates from ballots.
fn reference_runoff(mut ballots: Vec<Vec<u8>>, break_ties: bool) -> (Option<u8>, Vec<u8>, Trace) {
    let voters = ballots.len();
    let mut trace = Vec::new();
    loop {
        let mut counts: BTreeMap<u8, u128> =
            ballots[0].iter().map(|&candidate| (candidate, 0)).collect();
        for ballot in &ballots {
            *counts.get_mut(&ballot[0]).unwrap() += 1;
        }
        if let Some((&winner, _)) = counts
            .iter()
            .find(|&(_, &count)| count > voters as u128 / 2)
        {
            trace.push((counts, None));
            return (Some(winner), vec![], trace);
        }
        let minimum = *counts.values().min().unwrap();
        let tied: Vec<u8> = counts
            .iter()
            .filter(|&(_, &count)| count == minimum)
            .map(|(&c, _)| c)
            .collect();
        if tied.len() > 1 && !break_ties {
            trace.push((counts, None));
            return (None, tied, trace);
        }
        let loser = *counts
            .keys()
            .min_by_key(|&&candidate| (counts[&candidate], Reverse(candidate)))
            .unwrap();
        trace.push((counts, Some(loser)));
        for ballot in &mut ballots {
            ballot.retain(|&candidate| candidate != loser);
        }
    }
}

fn check_runoff(result: &RunoffResults<u8>, expected: &(Option<u8>, Vec<u8>, Trace)) {
    assert_eq!(result.winner().copied(), expected.0);
    if let RunoffOutcome::EliminationTie(tied) = result.outcome() {
        let mut tied = tied.clone();
        tied.sort();
        assert_eq!(tied, expected.1);
    }
    let trace: Trace = result
        .rounds()
        .iter()
        .map(|round| (scores(&round.tallies), round.eliminated))
        .collect();
    assert_eq!(trace, expected.2);
}

#[test]
fn exhaustively_checks_pairwise_and_runoff_for_small_elections() {
    let permutations = [
        vec![0, 1, 2],
        vec![0, 2, 1],
        vec![1, 0, 2],
        vec![1, 2, 0],
        vec![2, 0, 1],
        vec![2, 1, 0],
    ];
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
            let comparisons = pairwise(&pref);
            let mut expected_scores = BTreeMap::new();
            let mut strict_winner = None;
            for first in 0..3 {
                let mut score = 0;
                let mut wins = 0;
                for second in 0..3 {
                    if first == second {
                        continue;
                    }
                    let count = ballots
                        .iter()
                        .filter(|ballot| {
                            ballot.iter().position(|&c| c == first)
                                < ballot.iter().position(|&c| c == second)
                        })
                        .count();
                    assert_eq!(comparisons.preference_count(&first, &second), Some(count));
                    assert_eq!(
                        comparisons.preference_count(&second, &first),
                        Some(voters as usize - count)
                    );
                    if 2 * count > voters as usize {
                        score += 2;
                        wins += 1;
                    } else if 2 * count == voters as usize {
                        score += 1;
                    }
                }
                expected_scores.insert(first, score);
                if wins == 2 {
                    strict_winner = Some(first);
                }
            }
            assert_eq!(scores(&comparisons.copeland()), expected_scores);
            assert_eq!(expected_scores.values().sum::<u128>(), 6);
            assert_eq!(comparisons.condorcet_winner().copied(), strict_winner);
            if let Some(winner) = strict_winner {
                assert_eq!(comparisons.copeland().winners(), &[winner]);
            }

            let unresolved = reference_runoff(ballots.clone(), false);
            let resolved = reference_runoff(ballots.clone(), true);
            check_runoff(&instant_runoff(&pref), &unresolved);
            check_runoff(&instant_runoff_by(&pref, Ord::cmp), &resolved);

            let mut reversed = ballots;
            reversed.reverse();
            let reversed = Preference::new(reversed).unwrap();
            assert_eq!(scores(&copeland(&reversed)), expected_scores);
            assert_eq!(condorcet_winner(&reversed), strict_winner);
            check_runoff(&instant_runoff(&reversed), &unresolved);
            check_runoff(&instant_runoff_by(&reversed, Ord::cmp), &resolved);
        }
    }
}

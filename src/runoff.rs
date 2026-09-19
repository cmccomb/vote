use crate::{candidate_indices, Preference, Results};
use std::cmp::Ordering;
use std::hash::Hash;

/// The outcome of an instant-runoff count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunoffOutcome<T> {
    /// A candidate has a strict majority of all ballots.
    Winner(T),
    /// The count stopped because one candidate could not be selected for elimination.
    ///
    /// These are the candidates tied for elimination after applying any supplied
    /// comparator. They are not a set of winners; other candidates may still
    /// be active. Display order follows the first ballot.
    EliminationTie(Vec<T>),
}

/// A tally before an elimination, or the terminal tally of a runoff count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunoffRound<T> {
    /// Active candidates and their current first-remaining-preference vote counts.
    /// Previously eliminated candidates are omitted; zero-vote active candidates remain.
    pub tallies: Results<T>,
    /// The single candidate removed after this tally, or `None` in the final round.
    pub eliminated: Option<T>,
}

/// An instant-runoff outcome and the full trace of rounds leading to it.
///
/// Counting stops at a strict majority or an unresolved elimination tie. The
/// trace does not assign an overall ranking to the unelected candidates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunoffResults<T> {
    outcome: RunoffOutcome<T>,
    rounds: Vec<RunoffRound<T>>,
}

impl<T> RunoffResults<T> {
    /// Borrow the elected candidate or unresolved elimination tie.
    pub fn outcome(&self) -> &RunoffOutcome<T> {
        &self.outcome
    }

    /// Borrow the elected candidate, or return `None` if the count stopped at a tie.
    pub fn winner(&self) -> Option<&T> {
        match &self.outcome {
            RunoffOutcome::Winner(candidate) => Some(candidate),
            RunoffOutcome::EliminationTie(_) => None,
        }
    }

    /// Borrow all rounds in counting order, including the terminal tally.
    pub fn rounds(&self) -> &[RunoffRound<T>] {
        &self.rounds
    }
}

/// Run instant-runoff voting, stopping if the lowest vote count is tied.
///
/// Each voter supports their highest-ranked active candidate. A strict majority
/// (more than half of all ballots) elects a winner immediately. Otherwise, the
/// unique lowest candidate is eliminated and their votes transfer. A tie for
/// lowest stops the count with [`RunoffOutcome::EliminationTie`], even if the
/// tied candidates have zero votes. Use [`instant_runoff_by`] to choose a policy.
///
/// Only complete, strict ballots are accepted by [`Preference`], so no ballots
/// are exhausted. This is a generic single-winner count, with no automatic
/// historical-count or jurisdiction-specific tie rules.
pub fn instant_runoff<T: Eq + Hash + Clone>(pref: &Preference<T>) -> RunoffResults<T> {
    instant_runoff_by(pref, |_, _| Ordering::Equal)
}

/// Run instant-runoff voting with an explicit preference order for elimination ties.
///
/// `Ordering::Less` favors the first candidate: among candidates tied for the
/// lowest vote count, the greatest (least preferred) is eliminated. For example,
/// `Ord::cmp` favors alphabetically earlier names and eliminates the latest name
/// in a tied group. The comparator is never used across different vote counts.
///
/// Supply a consistent total order to resolve every tie. If the comparator leaves
/// multiple candidates equally least preferred, the count stops and reports them
/// in [`RunoffOutcome::EliminationTie`]. It never silently uses ballot order or
/// eliminates an entire tied group. A majority winner is elected before considering
/// elimination ties. The profile is borrowed and remains unchanged.
pub fn instant_runoff_by<T, F>(pref: &Preference<T>, mut compare: F) -> RunoffResults<T>
where
    T: Eq + Hash + Clone,
    F: FnMut(&T, &T) -> Ordering,
{
    let candidates = pref.candidates();
    let indices = candidate_indices(pref);
    let ballots: Vec<Vec<usize>> = pref
        .ballots()
        .iter()
        .map(|ballot| ballot.iter().map(|candidate| indices[candidate]).collect())
        .collect();
    let mut positions = vec![0; pref.voter_count()];
    let mut active = vec![true; pref.candidate_count()];
    let mut rounds = Vec::new();

    loop {
        let mut scores = vec![0_u128; pref.candidate_count()];
        for (ballot, position) in ballots.iter().zip(&mut positions) {
            // At least one candidate remains, and every ballot contains them.
            // Cursors only advance, skipping already-eliminated preferences.
            while !active[ballot[*position]] {
                *position += 1;
            }
            scores[ballot[*position]] += 1;
        }
        let remaining: Vec<usize> = (0..candidates.len())
            .filter(|&index| active[index])
            .collect();
        let round_candidates: Vec<T> = remaining
            .iter()
            .map(|&index| candidates[index].clone())
            .collect();
        let round_scores = remaining.iter().map(|&index| scores[index]).collect();
        let tallies = Results::from_scores(&round_candidates, round_scores);

        if let Some(&winner) = remaining
            .iter()
            .find(|&&index| scores[index] > pref.voter_count() as u128 / 2)
        {
            rounds.push(RunoffRound {
                tallies,
                eliminated: None,
            });
            return RunoffResults {
                outcome: RunoffOutcome::Winner(candidates[winner].clone()),
                rounds,
            };
        }

        // A validated nonempty profile always has at least one active candidate.
        let lowest_score = remaining.iter().map(|&index| scores[index]).min().unwrap();
        let mut lowest: Vec<usize> = Vec::new();
        for index in remaining
            .into_iter()
            .filter(|&index| scores[index] == lowest_score)
        {
            if let Some(&other) = lowest.first() {
                match compare(&candidates[index], &candidates[other]) {
                    Ordering::Less => continue,
                    Ordering::Greater => lowest.clear(),
                    Ordering::Equal => {}
                }
            }
            lowest.push(index);
        }
        if lowest.len() > 1 {
            let tied = lowest
                .into_iter()
                .map(|index| candidates[index].clone())
                .collect();
            rounds.push(RunoffRound {
                tallies,
                eliminated: None,
            });
            return RunoffResults {
                outcome: RunoffOutcome::EliminationTie(tied),
                rounds,
            };
        }
        let eliminated = lowest[0];
        active[eliminated] = false;
        rounds.push(RunoffRound {
            tallies,
            eliminated: Some(candidates[eliminated].clone()),
        });
    }
}

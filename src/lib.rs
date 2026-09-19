#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod pairwise;
mod preference;
mod results;
mod runoff;

pub use pairwise::{condorcet_winner, copeland, pairwise, PairwiseResults};
pub use preference::{Preference, PreferenceError};
pub use results::{Results, ScoreGroup};
pub use runoff::{instant_runoff, instant_runoff_by, RunoffOutcome, RunoffResults, RunoffRound};

use rand::{Rng, RngExt};
use std::collections::HashMap;
use std::hash::Hash;

/// Count first-place votes, retaining every candidate, including those with zero votes.
///
/// Equal totals share a [`ScoreGroup`]; no tie is broken automatically.
pub fn plurality<T: Eq + Hash + Clone>(pref: &Preference<T>) -> Results<T> {
    let indices = candidate_indices(pref);
    let mut scores = vec![0; pref.candidate_count()];
    for ballot in pref.ballots() {
        scores[indices[&ballot[0]]] += 1;
    }
    Results::from_scores(pref.candidates(), scores)
}

/// Sum Borda points: with `m` candidates, each ballot awards `m - 1, ..., 0`.
///
/// Equal totals share a [`ScoreGroup`]. Scores use `u128`, which holds the
/// product of the voter and candidate counts on supported 32- and 64-bit targets.
/// The 0.1 API used `m, ..., 1` internally; subtracting one point per voter
/// from every candidate preserves all rankings and ties for complete ballots.
pub fn borda<T: Eq + Hash + Clone>(pref: &Preference<T>) -> Results<T> {
    let indices = candidate_indices(pref);
    let mut scores = vec![0; pref.candidate_count()];
    for ballot in pref.ballots() {
        for (rank, candidate) in ballot.iter().enumerate() {
            scores[indices[candidate]] += (pref.candidate_count() - 1 - rank) as u128;
        }
    }
    Results::from_scores(pref.candidates(), scores)
}

fn candidate_indices<T: Eq + Hash>(pref: &Preference<T>) -> HashMap<&T, usize> {
    pref.candidates()
        .iter()
        .enumerate()
        .map(|(index, candidate)| (candidate, index))
        .collect()
}

/// Select a voter uniformly and return a copy of that voter's ranked ballot.
///
/// Each ballot has equal probability, including repeated identical ballots.
/// Uses the thread-local RNG. For experiments, use [`random_dictator_with_rng`].
/// Unlike the positional methods, this returns a ballot, not aggregate scores.
pub fn random_dictator<T: Clone>(pref: &Preference<T>) -> Vec<T> {
    random_dictator_with_rng(pref, &mut rand::rng())
}

/// Select a voter uniformly using the caller's RNG and copy their ranked ballot.
///
/// Reusing the same seed, RNG implementation, dependency versions, target, and
/// ballot order reproduces a sequence of selections. `rand::rngs::StdRng` does
/// not promise identical streams across versions or platforms. The `unbiased`
/// feature of `rand` is enabled for integer range sampling.
pub fn random_dictator_with_rng<T: Clone, R: Rng + ?Sized>(
    pref: &Preference<T>,
    rng: &mut R,
) -> Vec<T> {
    let index = rng.random_range(0..pref.voter_count());
    pref.ballots()[index].clone()
}

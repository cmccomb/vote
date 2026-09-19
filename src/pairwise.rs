use crate::{candidate_indices, Preference, Results};
use std::cmp::Ordering;
use std::hash::Hash;

/// Head-to-head preference counts, with candidates in first-ballot order.
///
/// Entry `[i][j]` counts voters who rank candidate `i` ahead of candidate `j`.
/// The diagonal is zero; opposite off-diagonal entries sum to the voter count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairwiseResults<T> {
    candidates: Vec<T>,
    counts: Vec<Vec<usize>>,
}

impl<T> PairwiseResults<T> {
    /// Candidates indexing both the rows and columns of [`Self::counts`].
    pub fn candidates(&self) -> &[T] {
        &self.candidates
    }

    /// Borrow the square matrix of head-to-head voter counts.
    pub fn counts(&self) -> &[Vec<usize>] {
        &self.counts
    }

    /// Return the candidate who strictly defeats every other candidate, if any.
    ///
    /// Pairwise ties do not count as victories. Cycles or tied contests may
    /// leave no strict Condorcet winner. A single candidate wins vacuously.
    pub fn condorcet_winner(&self) -> Option<&T> {
        (0..self.candidates.len())
            .find(|&candidate| {
                (0..self.candidates.len()).all(|opponent| {
                    candidate == opponent
                        || self.counts[candidate][opponent] > self.counts[opponent][candidate]
                })
            })
            .map(|index| &self.candidates[index])
    }
}

impl<T: Eq> PairwiseResults<T> {
    /// Count voters preferring `preferred` to `other`.
    ///
    /// Returns `None` if either candidate is unknown and `Some(0)` for a known
    /// candidate compared with itself. Lookup takes linear time in the number
    /// of candidates; use [`Self::counts`] for repeated indexed comparisons.
    pub fn preference_count(&self, preferred: &T, other: &T) -> Option<usize> {
        let row = self
            .candidates
            .iter()
            .position(|candidate| candidate == preferred)?;
        let column = self
            .candidates
            .iter()
            .position(|candidate| candidate == other)?;
        Some(self.counts[row][column])
    }
}

impl<T: Clone> PairwiseResults<T> {
    /// Score Copeland's method from these counts without recounting ballots.
    ///
    /// Uses Copeland(0.5), scaled by two to keep scores integral: two points
    /// per pairwise win, one per pairwise tie, zero per loss. Equal final scores
    /// remain grouped. A strict Condorcet winner, when present, uniquely wins.
    pub fn copeland(&self) -> Results<T> {
        let mut scores = vec![0; self.candidates.len()];
        for first in 0..self.candidates.len() {
            for second in first + 1..self.candidates.len() {
                match self.counts[first][second].cmp(&self.counts[second][first]) {
                    Ordering::Greater => scores[first] += 2,
                    Ordering::Less => scores[second] += 2,
                    Ordering::Equal => {
                        scores[first] += 1;
                        scores[second] += 1;
                    }
                }
            }
        }
        Results::from_scores(&self.candidates, scores)
    }
}

/// Count every pairwise preference in a validated profile.
///
/// Takes expected `O(voters * candidates^2)` time and `O(candidates^2)` auxiliary
/// space. Ballots are complete and strict, so every voter contributes to exactly
/// one side of each contest. Repeated ballots keep their multiplicity.
pub fn pairwise<T: Eq + Hash + Clone>(pref: &Preference<T>) -> PairwiseResults<T> {
    let indices = candidate_indices(pref);
    let mut counts = vec![vec![0; pref.candidate_count()]; pref.candidate_count()];
    for ballot in pref.ballots() {
        for (rank, candidate) in ballot.iter().enumerate() {
            let row = indices[candidate];
            for other in &ballot[rank + 1..] {
                counts[row][indices[other]] += 1;
            }
        }
    }
    PairwiseResults {
        candidates: pref.candidates().to_vec(),
        counts,
    }
}

/// Score Copeland(0.5): two points per pairwise win, one per tie, zero per loss.
///
/// This doubles the usual one-point/half-point scoring convention without
/// changing winners or ties. Use [`pairwise`] to inspect or reuse the counts.
pub fn copeland<T: Eq + Hash + Clone>(pref: &Preference<T>) -> Results<T> {
    pairwise(pref).copeland()
}

/// Copy the strict Condorcet winner, if one candidate beats every other head-to-head.
///
/// Returns `None` when no candidate strictly defeats all opponents. Use
/// [`copeland`] for a scored outcome even when there is no Condorcet winner,
/// or [`pairwise`] to inspect the individual contests.
pub fn condorcet_winner<T: Eq + Hash + Clone>(pref: &Preference<T>) -> Option<T> {
    pairwise(pref).condorcet_winner().cloned()
}

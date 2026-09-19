use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::hash::Hash;

/// A nonempty profile of complete, strict ranked ballots over the same candidates.
///
/// Each ballot lists candidates from most to least preferred. Candidates need
/// `Eq + Hash` for validation; they do not need `Ord` or `Debug`. Voting methods
/// clone candidates into their output. Ballots are private so a valid profile
/// cannot subsequently be changed into an invalid one through this API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preference<T> {
    ballots: Vec<Vec<T>>,
}

/// Why a profile could not be constructed. All ballot and rank indices are zero-based.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PreferenceError {
    /// No voters were supplied.
    EmptyProfile,
    /// A voter supplied no candidates.
    EmptyBallot {
        /// Index of the empty ballot.
        ballot: usize,
    },
    /// A ballot has a different length from the first ballot.
    BallotLengthMismatch {
        /// Index of the malformed ballot.
        ballot: usize,
        /// Number of candidates in the first ballot.
        expected: usize,
        /// Number of entries in this ballot.
        actual: usize,
    },
    /// A candidate occurs more than once in a ballot.
    DuplicateCandidate {
        /// Index of the malformed ballot.
        ballot: usize,
        /// Index of the repeated occurrence.
        rank: usize,
    },
    /// A ballot includes a candidate absent from the first ballot.
    UnknownCandidate {
        /// Index of the malformed ballot.
        ballot: usize,
        /// Index of the unknown candidate.
        rank: usize,
    },
}

impl fmt::Display for PreferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProfile => write!(f, "a preference profile must contain at least one voter"),
            Self::EmptyBallot { ballot } => write!(f, "ballot {ballot} contains no candidates"),
            Self::BallotLengthMismatch {
                ballot,
                expected,
                actual,
            } => write!(
                f,
                "ballot {ballot} contains {actual} entries; expected {expected}"
            ),
            Self::DuplicateCandidate { ballot, rank } => write!(
                f,
                "ballot {ballot} repeats a candidate at rank index {rank}"
            ),
            Self::UnknownCandidate { ballot, rank } => write!(
                f,
                "ballot {ballot} has an unknown candidate at rank index {rank}"
            ),
        }
    }
}

impl Error for PreferenceError {}

impl<T: Eq + Hash> Preference<T> {
    /// Validate and own a profile, using the first ballot as its candidate set.
    ///
    /// Validation takes expected `O(voters * candidates)` time and `O(candidates)`
    /// auxiliary space, comparing each ballot to the first candidate set.
    ///
    /// # Errors
    ///
    /// Rejects an empty profile, empty ballots, unequal lengths, duplicate
    /// candidates, and differing candidate sets. For each ballot, emptiness and
    /// length are checked before individual entries; the first error is returned.
    pub fn new(ballots: Vec<Vec<T>>) -> Result<Self, PreferenceError> {
        let first = ballots.first().ok_or(PreferenceError::EmptyProfile)?;
        let expected = first.len();
        let candidates: HashSet<&T> = first.iter().collect();
        for (ballot_index, ballot) in ballots.iter().enumerate() {
            if ballot.is_empty() {
                return Err(PreferenceError::EmptyBallot {
                    ballot: ballot_index,
                });
            }
            if ballot.len() != expected {
                return Err(PreferenceError::BallotLengthMismatch {
                    ballot: ballot_index,
                    expected,
                    actual: ballot.len(),
                });
            }
            let mut seen = HashSet::with_capacity(expected);
            for (rank, candidate) in ballot.iter().enumerate() {
                if !seen.insert(candidate) {
                    return Err(PreferenceError::DuplicateCandidate {
                        ballot: ballot_index,
                        rank,
                    });
                }
                if !candidates.contains(candidate) {
                    return Err(PreferenceError::UnknownCandidate {
                        ballot: ballot_index,
                        rank,
                    });
                }
            }
        }
        Ok(Self { ballots })
    }
}

impl<T> Preference<T> {
    /// Borrow the validated ballots in their original voter order.
    pub fn ballots(&self) -> &[Vec<T>] {
        &self.ballots
    }

    /// Borrow the candidates in first-ballot order.
    ///
    /// This order is used for display within a tied score group, not to break ties.
    pub fn candidates(&self) -> &[T] {
        &self.ballots[0]
    }

    /// Number of voters (always at least one).
    pub fn voter_count(&self) -> usize {
        self.ballots.len()
    }

    /// Number of candidates (always at least one).
    pub fn candidate_count(&self) -> usize {
        self.ballots[0].len()
    }

    /// Consume the profile and recover the original ballots without cloning.
    pub fn into_ballots(self) -> Vec<Vec<T>> {
        self.ballots
    }
}

impl<T: Eq + Hash> TryFrom<Vec<Vec<T>>> for Preference<T> {
    type Error = PreferenceError;

    fn try_from(ballots: Vec<Vec<T>>) -> Result<Self, Self::Error> {
        Self::new(ballots)
    }
}

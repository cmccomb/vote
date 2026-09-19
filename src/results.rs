use std::cmp::Ordering;

/// All candidates with a particular score. Their order does not imply a ranking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreGroup<T> {
    /// Shared aggregate score. Larger scores are better.
    pub score: u128,
    /// Candidates sharing this score, displayed in first-ballot order.
    pub candidates: Vec<T>,
}

/// Positional voting results, grouped by score from highest to lowest.
///
/// Every candidate occurs exactly once, including candidates with zero points.
/// Equal scores remain tied. Use [`Self::ranking_by`] only when a downstream
/// application requires a ranking with a caller-chosen tie-break rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Results<T> {
    groups: Vec<ScoreGroup<T>>,
}

impl<T> Results<T> {
    /// Borrow score groups from highest to lowest score.
    pub fn groups(&self) -> &[ScoreGroup<T>] {
        &self.groups
    }

    /// Borrow every candidate tied for the highest score.
    ///
    /// A unique winner exists exactly when this slice has length one.
    pub fn winners(&self) -> &[T] {
        &self.groups[0].candidates
    }

    /// Consume the result and recover its score groups without cloning.
    pub fn into_groups(self) -> Vec<ScoreGroup<T>> {
        self.groups
    }
}

impl<T: Clone> Results<T> {
    pub(crate) fn from_scores(candidates: &[T], scores: Vec<u128>) -> Self {
        let mut indices: Vec<usize> = (0..candidates.len()).collect();
        // Stable sorting keeps first-ballot order inside each tie, independent
        // of HashMap iteration order. That order is never treated as a rank.
        indices.sort_by(|&a, &b| scores[b].cmp(&scores[a]));
        let mut groups: Vec<ScoreGroup<T>> = Vec::new();
        for index in indices {
            if let Some(group) = groups.last_mut() {
                if group.score == scores[index] {
                    group.candidates.push(candidates[index].clone());
                    continue;
                }
            }
            groups.push(ScoreGroup {
                score: scores[index],
                candidates: vec![candidates[index].clone()],
            });
        }
        Self { groups }
    }

    /// Copy candidates into descending score order, breaking ties with `compare`.
    ///
    /// The comparator only orders candidates within the same score group;
    /// `Ordering::Less` puts the first candidate earlier. Supply a total order
    /// for a complete tie-break policy. Candidates comparing equal retain their
    /// first-ballot order. The original result keeps its tied groups unchanged.
    pub fn ranking_by<F>(&self, mut compare: F) -> Vec<T>
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.groups
            .iter()
            .flat_map(|group| {
                let mut candidates = group.candidates.clone();
                candidates.sort_by(&mut compare);
                candidates
            })
            .collect()
    }
}

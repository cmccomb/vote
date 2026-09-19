[![build](https://github.com/cmccomb/vote/actions/workflows/tests.yml/badge.svg?branch=master)](https://github.com/cmccomb/vote/actions/workflows/tests.yml)
[![Crates.io](https://img.shields.io/crates/v/vote.svg)](https://crates.io/crates/vote)
[![docs.rs](https://docs.rs/vote/badge.svg)](https://docs.rs/vote)

# About

`vote` aggregates ranked preferences with plurality, Borda, and random dictatorship.
It validates complete ballots, reports scores and ties explicitly, and accepts a
caller-supplied random number generator for repeatable voting experiments.

# Install

Requires **Rust 1.85 or newer**. Add `vote` to your project's `Cargo.toml`:

```toml
[dependencies]
vote = "0.2"
```

# Usage

```rust
use vote::{borda, plurality, Preference};

fn main() -> Result<(), vote::PreferenceError> {
    let preferences = Preference::new(vec![
        vec!["bridge", "canopy", "courtyard"],
        vec!["canopy", "bridge", "courtyard"],
        vec!["bridge", "courtyard", "canopy"],
    ])?;

    // Reuse the same validated profile with multiple voting methods.
    let first_choices = plurality(&preferences);
    let ranked_scores = borda(&preferences);
    assert_eq!(first_choices.winners(), &["bridge"]);
    assert_eq!(ranked_scores.groups()[0].score, 5);

    for group in ranked_scores.groups() {
        println!("{} points: {:?}", group.score, group.candidates);
    }
    Ok(())
}
```

Output:

```text
5 points: ["bridge"]
3 points: ["canopy"]
1 points: ["courtyard"]
```

# Voting Methods

| Method | Rule | Output |
|--------|------|--------|
| `plurality` | One point for each voter's first choice | Score groups, highest first |
| `borda` | With `m` candidates, award `m - 1, ..., 0` points | Score groups, highest first |
| `random_dictator` | Select a voter uniformly | A copy of that voter's ranked ballot |
| `random_dictator_with_rng` | Same rule, using the caller's RNG | A copy of that voter's ranked ballot |

Each ballot represents one voter. Repeated identical ballots retain their
multiplicity. Scores use `u128`, and candidates with zero points remain in the
results. Instant-runoff voting and Condorcet methods are not yet implemented.

## Ballot requirements

Every ballot lists the same candidates exactly once, from most to least
preferred. Profiles and ballots must be nonempty. Partial rankings, tied ranks
within ballots, and separate voter weights are not supported. Candidate types
need `Eq + Hash` for validation and `Clone` for voting; `Ord` is optional.

`Preference::new` (or `Preference::try_from`) returns a `PreferenceError` for
invalid input, with zero-based ballot and rank indices where applicable:

```rust
use vote::{Preference, PreferenceError};

let malformed = Preference::new(vec![vec![0, 1], vec![0, 1, 1]]);
assert_eq!(
    malformed,
    Err(PreferenceError::BallotLengthMismatch {
        ballot: 1,
        expected: 2,
        actual: 3,
    }),
);
```

Validated ballots are accessible through `ballots()` and cannot be mutated in
place. Validation compares each ballot with the first candidate set, taking
expected time proportional to the number of ballot entries.

## Ties

`Results::groups()` preserves equal scores in a shared `ScoreGroup`.
`winners()` returns every candidate tied for the highest score. Candidates
within each group are displayed in first-ballot order; that order does not break
the tie. Reordering voters can change this display order, but not scores or tie
membership.

If an application needs a single ranking, supply its tie-break rule explicitly:

```rust
use vote::{plurality, Preference};

fn main() -> Result<(), vote::PreferenceError> {
    let preferences = Preference::new(vec![vec!["B", "A"], vec!["A", "B"]])?;
    let result = plurality(&preferences);

    assert_eq!(result.winners(), &["B", "A"]); // Still tied.
    let alphabetical = result.ranking_by(Ord::cmp);
    assert_eq!(alphabetical, vec!["A", "B"]);
    Ok(())
}
```

The comparator only orders candidates with equal scores. Use a total order to
resolve all ties. The original score groups remain unchanged.

## Seeded experiments

Add `rand = "0.10.2"` to your project's dependencies to run this example:

```rust
use rand::{rngs::StdRng, SeedableRng};
use vote::{random_dictator_with_rng, Preference};

fn main() -> Result<(), vote::PreferenceError> {
    let preferences = Preference::new(vec![vec!["A", "B"], vec!["B", "A"]])?;
    let mut first_run = StdRng::seed_from_u64(42);
    let mut repeated_run = StdRng::seed_from_u64(42);

    for _ in 0..10 {
        assert_eq!(
            random_dictator_with_rng(&preferences, &mut first_run),
            random_dictator_with_rng(&preferences, &mut repeated_run),
        );
    }
    Ok(())
}
```

To reproduce an experiment, preserve the seed, RNG implementation, dependency
versions, target, and ballot order. `StdRng` does not guarantee the same stream
across versions or platforms. `vote` enables `rand`'s unbiased integer sampling.

# Migrating from 0.1

Version 0.2 makes intentional API changes:

| Earlier API | 0.2 API |
|-------------|---------|
| `Preference(ballots)` followed by `audit()` | `Preference::new(ballots)?`, which validates once |
| Public `preferences.0` | `preferences.ballots()` or `preferences.into_ballots()` |
| `plurality(preferences)` or `borda(preferences)` | Borrow with `plurality(&preferences)` or `borda(&preferences)` |
| Flat `results.0` ranking with implicit tie order | `results.groups()`, `results.winners()`, or an explicit `results.ranking_by(...)` |
| `random_dictator(preferences).0` | `random_dictator(&preferences)`, returning `Vec<T>` directly |

Borda now reports standard `m - 1, ..., 0` scores. The earlier implementation
used `m, ..., 1` internally; rankings and ties are unchanged for valid complete
ballots. Invalid inputs produce construction errors instead of audit/indexing
panics.

# Development

```sh
cargo test --all-features
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo run --example simple
cargo run --example seeded
```

The README is also the crate-level documentation, so its Rust examples run as
documentation tests. CI checks stable Rust on Linux, macOS, and Windows, plus
Rust 1.85 on Linux. Tests include the malformed-ballot regressions and all 1,554
complete profiles with three candidates and one to four voters.

See the [published API documentation](https://docs.rs/vote),
[examples](https://github.com/cmccomb/vote/tree/master/examples), and
[changelog](https://github.com/cmccomb/vote/blob/master/CHANGELOG.md).
For API documentation matching this checkout, run `cargo doc --open`.

# License

Licensed under either the [MIT license](https://github.com/cmccomb/vote/blob/master/LICENSE.md)
or the [Apache License, Version 2.0](https://github.com/cmccomb/vote/blob/master/LICENSE-APACHE),
at your option.

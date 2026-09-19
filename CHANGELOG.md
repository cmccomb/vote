# Changelog

## 0.3.0 - 2026-09-19

- Add Copeland(0.5) with integer scores scaled by two and explicit tied groups.
- Expose pairwise comparison counts and strict Condorcet winner detection,
  including reusable matrix methods to avoid recounting ballots.
- Add instant-runoff voting with per-round tallies, single-candidate
  eliminations, and explicit unresolved elimination ties. A caller-supplied
  comparator can resolve ties without changing candidates' unequal vote counts.
- Preserve the existing 0.2 API and its complete, strict ballot requirements.
- Add a methods-comparison example, README guidance, a published counting
  fixture, and exhaustive pairwise/runoff checks over 1,554 small profiles.

## 0.2.0 - 2026-09-19

- Validate profiles at construction with `Preference::new` and `TryFrom`,
  returning typed errors for empty inputs, unequal lengths, duplicates, and
  mismatched candidate sets. Profile fields are now private.
- Replace pairwise ballot auditing with validation against one candidate set,
  taking expected time proportional to the number of ballot entries.
- Borrow profiles in voting functions so callers can compare methods without
  cloning the ballots or repeating validation.
- Return scores and explicit tied groups from plurality and Borda. Include
  zero-score candidates and require a caller-supplied comparator to flatten
  ties into a single ranking.
- Report standard Borda scores (`m - 1` through `0`) using `u128`. Rankings
  and ties for valid profiles are unchanged from the earlier scoring convention.
- Add `random_dictator_with_rng`. Random dictatorship now returns a copied
  ranked ballot directly as `Vec<T>`.
- Update to `rand` 0.10.2 with unbiased integer sampling, Rust 2021 edition,
  and a declared minimum Rust version of 1.85.
- Replace Travis CI with GitHub Actions for tests, formatting, Clippy,
  documentation, minimum-version compatibility, and package verification.
- Standardize README badges, overview, usage, methods, and development guidance
  with the other Rust repositories. Include migration and seeded examples.
- Add regression coverage and exhaustive score/invariance checks for all 1,554
  complete profiles with three candidates and one to four voters.
- Include the Apache license text already named by the package metadata.

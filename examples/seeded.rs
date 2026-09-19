use rand::{rngs::StdRng, SeedableRng};
use vote::{random_dictator_with_rng, Preference};

fn main() -> Result<(), vote::PreferenceError> {
    let preferences = Preference::new(vec![
        vec!["bridge", "canopy", "courtyard"],
        vec!["canopy", "courtyard", "bridge"],
        vec!["courtyard", "bridge", "canopy"],
    ])?;
    let mut first_run = StdRng::seed_from_u64(42);
    let mut repeated_run = StdRng::seed_from_u64(42);

    for trial in 1..=10 {
        let selected = random_dictator_with_rng(&preferences, &mut first_run);
        assert_eq!(
            selected,
            random_dictator_with_rng(&preferences, &mut repeated_run)
        );
        println!("Trial {trial}: {selected:?}");
    }
    Ok(())
}

use vote::{borda, copeland, instant_runoff, instant_runoff_by, pairwise, plurality, Preference};

fn main() -> Result<(), vote::PreferenceError> {
    let mut ballots = vec![vec!["A", "B", "C"]; 4];
    ballots.extend(vec![vec!["B", "C", "A"]; 3]);
    ballots.extend(vec![vec!["C", "B", "A"]; 2]);
    let preferences = Preference::new(ballots)?;

    println!("Plurality winners: {:?}", plurality(&preferences).winners());
    println!("Borda winners: {:?}", borda(&preferences).winners());
    println!("Copeland winners: {:?}", copeland(&preferences).winners());
    let comparisons = pairwise(&preferences);
    println!("Pairwise row/column order: {:?}", comparisons.candidates());
    for row in comparisons.counts() {
        println!("{row:?}");
    }
    println!(
        "Strict Condorcet winner: {:?}",
        comparisons.condorcet_winner()
    );

    let runoff = instant_runoff(&preferences);
    for (index, round) in runoff.rounds().iter().enumerate() {
        println!(
            "Round {}: {:?}; eliminate {:?}",
            index + 1,
            round.tallies.groups(),
            round.eliminated
        );
    }
    println!("Runoff outcome: {:?}", runoff.outcome());
    assert_eq!(runoff.winner(), Some(&"B"));

    let tied = Preference::new(vec![vec!["B", "A"], vec!["A", "B"]])?;
    println!("Unresolved runoff: {:?}", instant_runoff(&tied).outcome());
    let resolved = instant_runoff_by(&tied, Ord::cmp);
    println!(
        "Favoring alphabetically earlier names: {:?}",
        resolved.outcome()
    );
    assert_eq!(resolved.winner(), Some(&"A"));
    Ok(())
}

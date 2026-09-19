use vote::{borda, plurality, Preference};

fn main() -> Result<(), vote::PreferenceError> {
    let preferences = Preference::new(vec![
        vec!["bridge", "canopy", "courtyard"],
        vec!["canopy", "bridge", "courtyard"],
        vec!["bridge", "courtyard", "canopy"],
    ])?;

    println!("Plurality winners: {:?}", plurality(&preferences).winners());
    for group in borda(&preferences).groups() {
        println!("{} points: {:?}", group.score, group.candidates);
    }

    let tied = Preference::new(vec![vec!["B", "A"], vec!["A", "B"]])?;
    let result = plurality(&tied);
    println!("Tied winners: {:?}", result.winners());
    println!("Alphabetical tie-break: {:?}", result.ranking_by(Ord::cmp));
    Ok(())
}

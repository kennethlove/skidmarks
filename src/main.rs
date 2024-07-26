use skidmarks::cli::parse;
use skidmarks::db::Database;

fn main() {
    let mut db = Database::new("streaks.ron").expect("Could not load database");
    let output = parse(&mut db);
    dbg!(output);
    // Serialize and save
    // Load
    // Alert to any streaks left for the day
}

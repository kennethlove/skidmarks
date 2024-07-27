use skidmarks::settings::Settings;
use skidmarks::cli::parse;
use skidmarks::db::Database;


fn main() {
    let settings = Settings::new().unwrap();
    let db_url = settings.database.url;
    let mut db = Database::new(db_url.as_str()).expect("Could not load database");
    parse(&mut db);

    // Alert to any streaks left for the day
}

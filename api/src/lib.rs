use shared::db::{get_database_url, Database};
use std::sync::{Arc, Mutex};

pub mod endpoints;

#[derive(Clone, Debug)]
pub struct AppState {
    database: Arc<Mutex<Database>>,
}

impl AppState {
    pub fn new() -> Self {
        let database_url = get_database_url();
        let database = Database::new(&database_url);
        Self {
            database: Arc::new(Mutex::new(database.unwrap())),
        }
    }
}

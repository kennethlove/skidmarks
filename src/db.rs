use crate::streaks::Streak;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

const DB_FILENAME: &str = "streaks.ron";

pub struct Database {
    pub filename: String,
    pub streaks: Mutex<Vec<Streak>>,
}

impl Database {
    fn create_if_missing(filename: &str) -> Result<(), std::io::Error> {
        let data = "[]".as_bytes();
        let metadata = match std::fs::metadata(filename) {
            Ok(meta) => meta,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(filename)?;
                file.write_all(data)?;
                return Ok(());
            }
            Err(err) => return Err(err),
        };

        if metadata.len() == 0 {
            let mut file = File::open(filename)?;
            file.write_all(data)?;
        }
        Ok(())
    }

    fn load_database(filename: &str) -> Result<Vec<Streak>, std::io::Error> {
        let ronned = std::fs::read_to_string(filename)?;
        let ronned: Vec<Streak> = ron::de::from_str(&ronned).expect("Couldn't load database.");
        Ok(ronned)
    }

    fn save_database(&self, filename: &str) {
        let streaks: Vec<Streak> = self.streaks.lock().unwrap().clone();
        let ronned = ron::ser::to_string(&streaks).unwrap();
        std::fs::write(filename, ronned).unwrap();
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        self.save_database(self.filename.as_str());
        Ok(())
    }

    pub fn new(filename: &str) -> Result<Self, std::io::Error> {
        Self::create_if_missing(filename)?;
        let existing_db = Self::load_database(filename)?;
        let new_db = Self {
            streaks: Mutex::new(existing_db.clone()),
            filename: filename.to_string(),
        };
        Ok(new_db)
    }
}

impl Default for Database {
    fn default() -> Self {
        Self {
            streaks: Mutex::new(Vec::new()),
            filename: DB_FILENAME.to_string(),
        }
    }
}

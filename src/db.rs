use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::streak::Streak;

lazy_static::lazy_static! {
    static ref FILE_LOCK: Mutex<()> = Mutex::new(());
}

#[derive(Debug)]
pub struct Database {
    pub filename: String,
    // pub streaks: Arc<Mutex<Vec<Streak>>>,
    pub streaks: Arc<Mutex<HashMap<Uuid, Streak>>>,
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            filename: self.filename.clone(),
            streaks: self.streaks.clone(),
        }
    }
}

impl PartialEq for Database {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
            && *self.streaks.lock().unwrap() == *other.streaks.lock().unwrap()
    }
}

impl Database {
    fn create_if_missing(filename: &str) -> Result<(), std::io::Error> {
        // let data = "[]".as_bytes();
        let data = "{}".as_bytes();
        let metadata = match std::fs::metadata(filename) {
            Ok(meta) => meta,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(filename)?;

                let _lock = FILE_LOCK.lock().unwrap();
                file.write_all(data)?;
                return Ok(());
            }
            Err(err) => return Err(err),
        };

        if metadata.len() == 0 {
            let mut file = File::open(filename)?;
            let _lock = FILE_LOCK.lock().unwrap();
            file.write_all(data)?;
        }
        Ok(())
    }

    pub fn num_tasks(&self) -> usize {
        self.streaks.lock().unwrap().len()
    }

    fn load_database(filename: &str) -> Result<HashMap<Uuid, Streak>, std::io::Error> {
        Self::create_if_missing(filename)?;
        let ronned = std::fs::read_to_string(filename)?;
        let ronned: HashMap<Uuid, Streak> =
            ron::de::from_str(&ronned).unwrap_or_else(|_| HashMap::<Uuid, Streak>::new());
        Ok(ronned)
    }

    fn save_database(&self, filename: &str) {
        let streaks: HashMap<Uuid, Streak> = self.streaks.lock().unwrap().clone();
        let ronned = ron::ser::to_string(&streaks).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(filename)
            .unwrap();
        let _lock = FILE_LOCK.lock().unwrap();
        file.write_all(ronned.as_bytes()).unwrap();
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        self.save_database(self.filename.as_str());
        Ok(())
    }

    pub fn add(&mut self, streak: Streak) -> Result<(), std::io::Error> {
        let mut streaks = self.streaks.lock().unwrap();
        let id = streak.id.clone();
        streaks.insert(id, streak);
        Ok(())
    }

    pub fn update(&mut self, id: Uuid, streak: Streak) -> Result<(), std::io::Error> {
        let mut streaks = self.streaks.lock().unwrap();
        let mut old_streak: &mut Streak = streaks.get_mut(&id).unwrap();
        let _ = old_streak.update(streak);
        Ok(())
    }

    pub fn delete(&mut self, id: Uuid) -> Result<(), std::io::Error> {
        let mut streaks = self.streaks.lock().unwrap();
        streaks.remove(&id);

        Ok(())
    }

    pub fn new(filename: &str) -> Result<Self, std::io::Error> {
        Self::create_if_missing(filename)?;
        let existing_db = Self::load_database(filename)?;
        let new_db = Self {
            streaks: Arc::new(Mutex::new(existing_db.clone())),
            filename: filename.to_string(),
        };
        Ok(new_db)
    }

    pub fn get_all(&mut self) -> Option<HashMap<Uuid, Streak>> {
        let streaks = self.streaks.lock();
        match streaks {
            Ok(streaks) => {
                if streaks.is_empty() {
                    Some(HashMap::<Uuid, Streak>::new())
                } else {
                    Some(streaks.clone())
                }
            }
            _ => None,
        }
    }

    pub fn get_one(&mut self, id: Uuid) -> Option<Streak> {
        let streaks = self.streaks.lock().unwrap();
        let streak = streaks.get(&id);
        match streak {
            Some(streak) => Some(streak.clone()),
            None => None,
        }
    }
}

impl Default for Database {
    fn default() -> Self {
        Self {
            streaks: Arc::new(Mutex::new(HashMap::<Uuid, Streak>::new())),
            filename: "skidmarks.ron".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;

    use super::*;

    #[test]
    fn test_create_if_missing() {
        let temp = assert_fs::TempDir::new().unwrap();
        let db_file = temp.child("test_create_if_missing.ron");

        let result = Database::create_if_missing(db_file.to_str().unwrap());
        assert!(result.is_ok());

        let result = Database::create_if_missing(db_file.to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_database() {
        let temp = assert_fs::TempDir::new().unwrap();
        let db_file = temp.child("test_load_database.ron");
        let _ = Database::new(db_file.to_str().unwrap()).unwrap();

        db_file
            .write_str(r#"[(task:"brush teeth",frequency:Daily,last_checkin:Some("2024-07-26"),total_checkins:1)]"#)
            .unwrap();

        let result = Database::load_database(db_file.to_str().unwrap());
        assert!(result.is_ok());
        assert!(result.unwrap().len() == 1);

        temp.close().unwrap();
    }

    #[test]
    fn test_load_database_empty() {
        let temp = assert_fs::TempDir::new().unwrap();
        let db_file = temp.child("test_load_database_empty.ron");
        let _ = Database::new(db_file.to_str().unwrap()).unwrap();

        let result = Database::load_database(db_file.to_str().unwrap());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());

        temp.close().unwrap();
    }

    #[test]
    fn test_save_database() {
        let temp = assert_fs::TempDir::new().unwrap();
        let db_file = temp.child("test_save_database.ron");
        let file_path = db_file.to_str().unwrap();

        let mut db = Database::new(file_path).unwrap();
        let streak = Streak::new_daily("brush teeth".to_string());
        db.add(streak).unwrap();
        db.save().unwrap();

        let expected_content =
            r#"[(task:"brush teeth",frequency:Daily,last_checkin:None,total_checkins:0)]"#;

        let result = std::fs::read_to_string(file_path);
        assert_eq!(result.unwrap(), expected_content);

        temp.close().unwrap();
    }
}

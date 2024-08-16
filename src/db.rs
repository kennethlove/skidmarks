use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

use crate::sorting::{SortByDirection, SortByField};
use crate::streak::{sort_streaks, Streak};
use uuid::Uuid;

lazy_static::lazy_static! {
    static ref FILE_LOCK: Mutex<()> = Mutex::new(());
}

#[derive(Debug)]
pub struct Database {
    pub filename: String,
    pub streaks: Vec<Streak>,
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
        self.filename == other.filename && *self.streaks == *other.streaks
    }
}

impl Database {
    fn create_if_missing(filename: &str) -> Result<(), std::io::Error> {
        // let data = "[]".as_bytes();
        let data = "[]".as_bytes();
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
        self.streaks.len()
    }

    fn load_database(filename: &str) -> Result<Vec<Streak>, std::io::Error> {
        Self::create_if_missing(filename)?;
        let contents = std::fs::read_to_string(filename)?;
        let decoded: Vec<Streak> =
            ron::de::from_str(&contents).unwrap_or_else(|_| Vec::<Streak>::new());
        Ok(decoded)
    }

    fn save_database(&self, filename: &str) {
        let streaks: Vec<Streak> = self.streaks.clone();
        let encoded = ron::ser::to_string(&streaks).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(filename)
            .unwrap();
        let _lock = FILE_LOCK.lock().unwrap();
        file.write_all(encoded.as_bytes()).unwrap();
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        self.save_database(self.filename.as_str());
        Ok(())
    }

    pub fn create_from_string(filename: &str, data: &str) -> Result<Self, std::io::Error> {
        let mut db = Self::new(filename)?;
        let streaks: Vec<Streak> = ron::de::from_str(data).unwrap();
        for streak in streaks {
            db.streaks.push(streak);
        }
        Ok(db)
    }

    pub fn add(&mut self, streak: Streak) -> Result<(), std::io::Error> {
        let mut streaks = self.streaks.clone();
        streaks.push(streak);
        self.streaks = streaks;
        Ok(())
    }

    pub fn update(&mut self, id: Uuid, streak: Streak) -> Result<(), std::io::Error> {
        self.delete(id)?;
        let mut streaks = self.streaks.clone();
        streaks.push(streak);
        self.streaks = streaks;
        Ok(())
    }

    pub fn delete(&mut self, id: Uuid) -> Result<(), std::io::Error> {
        let streaks = self.streaks.clone();
        let filtered_streaks = streaks
            .iter()
            .filter(|s| s.id != id)
            .cloned()
            .collect::<Vec<Streak>>();
        self.streaks = filtered_streaks;
        Ok(())
    }

    pub fn new(filename: &str) -> Result<Self, std::io::Error> {
        Self::create_if_missing(filename)?;
        let existing_db = Self::load_database(filename)?;
        let new_db = Self {
            streaks: existing_db.clone(),
            filename: filename.to_string(),
        };
        Ok(new_db)
    }

    pub fn get_all(&mut self) -> Vec<Streak> {
        match self.streaks.len() {
            0 => Vec::<Streak>::new(),
            _ => self.streaks.clone(),
        }
    }

    pub fn get_sorted(
        &self,
        sort_field: SortByField,
        sort_direction: SortByDirection,
    ) -> Vec<Streak> {
        let streaks = self.streaks.clone();
        sort_streaks(streaks, sort_field, sort_direction)
    }

    pub fn get_one(&mut self, id: Uuid) -> Option<Streak> {
        let streak = self.streaks.iter().find(|s| s.id == id);
        match streak {
            Some(streak) => Some(streak.clone()),
            None => None,
        }
    }

    pub fn get_by_index(&mut self, index: usize) -> Option<Streak> {
        let streak = self.streaks.iter().nth(index);
        match streak {
            Some(streak) => Some(streak.clone()),
            None => None,
        }
    }

    pub fn get_by_id(&mut self, ident: &str) -> Option<Streak> {
        let streak = self
            .streaks
            .iter()
            .find(|s| s.id.to_string()[0..5].to_string() == ident);
        match streak {
            Some(streak) => Some(streak.clone()),
            None => None,
        }
    }

    pub fn search(&mut self, query: &str) -> Vec<Streak> {
        let streaks = self.streaks.clone();
        streaks
            .iter()
            .filter(|s| s.task.contains(query))
            .cloned()
            .collect()
    }

    pub fn checkin(&mut self, id: Uuid) -> Result<(), std::io::Error> {
        let mut streaks = self.streaks.clone();
        let streak = streaks.iter_mut().find(|s| s.id == id);
        match streak {
            Some(streak) => {
                streak.checkin();
                self.streaks = streaks;
                Ok(())
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Streak not found",
            )),
        }
    }
}

impl Default for Database {
    fn default() -> Self {
        Self {
            streaks: Vec::<Streak>::new(),
            filename: "skidmarks.ron".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;

    use super::*;

    const DATABASE_PRELOAD: &str = r#"[(id:"00e8a16c-0edd-4e90-8c3f-2ee7aa6a2210",task:"Poop",frequency:Daily,last_checkin:Some("2024-08-06"),total_checkins:2),(id:"77cbbb3f-2690-45a9-9a30-94a53556d93e",task:"Take a walk",frequency:Daily,last_checkin:Some("2024-08-07"),total_checkins:1),(id:"af1f4cc5-87b1-40b1-9fa3-2e0344d35d3b",task:"Eat brekkie",frequency:Daily,last_checkin:Some("2024-08-05"),total_checkins:3)]"#;
    #[test]
    fn create_if_missing() {
        let temp = assert_fs::TempDir::new().unwrap();
        let db_file = temp.child("test_create_if_missing.ron");

        let result = Database::create_if_missing(db_file.to_str().unwrap());
        assert!(result.is_ok());

        let result = Database::create_if_missing(db_file.to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn create_from_string() {
        let temp = assert_fs::TempDir::new().unwrap();
        let db_file = temp.child("test_create_from_string.ron");

        let result = Database::create_from_string(db_file.to_str().unwrap(), DATABASE_PRELOAD);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().num_tasks(), 3);

        temp.close().unwrap();
    }

    #[test]
    fn load_database() {
        let temp = assert_fs::TempDir::new().unwrap();
        let db_file = temp.child("test_load_database.ron");

        let result = Database::create_from_string(db_file.to_str().unwrap(), DATABASE_PRELOAD);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().num_tasks(), 3);

        temp.close().unwrap();
    }

    #[test]
    fn load_database_empty() {
        let temp = assert_fs::TempDir::new().unwrap();
        let db_file = temp.child("test_load_database_empty.ron");
        let _ = Database::new(db_file.to_str().unwrap()).unwrap();

        let result = Database::load_database(db_file.to_str().unwrap());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());

        temp.close().unwrap();
    }

    #[test]
    fn save_database() {
        let temp = assert_fs::TempDir::new().unwrap();
        let db_file = temp.child("test_save_database.ron");
        let file_path = db_file.to_str().unwrap();

        let mut db = Database::new(file_path).unwrap();
        let streak = Streak::new_daily("brush teeth".to_string());
        db.add(streak.clone()).unwrap();
        db.save().unwrap();

        let expected_content = format!(
            r#"[(id:"{}",task:"{}",frequency:Daily,last_checkin:{:?},current_streak:{},longest_streak:{},total_checkins:{})]"#,
            streak.id,
            streak.task,
            streak.last_checkin,
            streak.current_streak,
            streak.longest_streak,
            streak.total_checkins
        );

        let result = std::fs::read_to_string(file_path);
        assert_eq!(result.unwrap(), expected_content);

        temp.close().unwrap();
    }

    #[test]
    fn add_streak() {
        let temp = assert_fs::TempDir::new().unwrap();
        let db_file = temp.child("test_add_streak.ron");
        let file_path = db_file.to_str().unwrap();

        let mut db = Database::new(file_path).unwrap();
        let streak = Streak::new_daily("brush teeth".to_string());
        db.add(streak.clone()).unwrap();

        let result = db.get_all();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], streak);

        temp.close().unwrap();
    }

    #[test]
    fn update_streak() {
        let temp = assert_fs::TempDir::new().unwrap();
        let db_file = temp.child("test_update_streak.ron");
        let file_path = db_file.to_str().unwrap();

        let mut db = Database::new(file_path).unwrap();
        let mut streak = Streak::new_daily("brush teeth".to_string());
        db.add(streak.clone()).unwrap();

        streak.task = "floss".to_string();
        db.update(streak.id, streak.clone()).unwrap();

        let result = db.get_all();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].task, "floss");

        temp.close().unwrap();
    }

    #[test]
    fn delete_streak() {
        let temp = assert_fs::TempDir::new().unwrap();
        let db_file = temp.child("test_delete_streak.ron");
        let file_path = db_file.to_str().unwrap();

        let mut db = Database::new(file_path).unwrap();
        let streak = Streak::new_daily("brush teeth".to_string());
        db.add(streak.clone()).unwrap();
        assert!(!db.get_all().is_empty());

        db.delete(streak.id).unwrap();

        let result = db.get_all();
        assert!(result.is_empty());

        temp.close().unwrap();
    }

    #[test]
    fn get_all_streaks() {
        let temp = assert_fs::TempDir::new().unwrap();
        let db_file = temp.child("test_get_all_streaks.ron");
        let file_path = db_file.to_str().unwrap();

        let mut db = Database::new(file_path).unwrap();
        let streak1 = Streak::new_daily("brush teeth".to_string());
        let streak2 = Streak::new_daily("floss".to_string());
        db.add(streak1.clone()).unwrap();
        db.add(streak2.clone()).unwrap();

        let result = db.get_all();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], streak1);
        assert_eq!(result[1], streak2);

        temp.close().unwrap();
    }

    #[test]
    fn get_one_streak() {
        let temp = assert_fs::TempDir::new().unwrap();
        let db_file = temp.child("test_get_one_streak.ron");
        let file_path = db_file.to_str().unwrap();

        let mut db = Database::new(file_path).unwrap();
        let streak1 = Streak::new_daily("brush teeth".to_string());
        let streak2 = Streak::new_daily("floss".to_string());
        db.add(streak1.clone()).unwrap();
        db.add(streak2.clone()).unwrap();

        let result = db.get_one(streak1.id).unwrap();
        assert_eq!(result, streak1);

        temp.close().unwrap();
    }

    #[test]
    fn get_streak_by_index() {
        let temp = assert_fs::TempDir::new().unwrap();
        let db_file = temp.child("test_get_streak_by_index.ron");
        let file_path = db_file.to_str().unwrap();

        let mut db = Database::create_from_string(file_path, DATABASE_PRELOAD).unwrap();
        let result = db.get_by_index(1).unwrap();
        let expected = db.streaks.iter().nth(1).unwrap().clone();
        assert_eq!(expected, result);

        temp.close().unwrap();
    }

    #[test]
    fn sort_by_task() {
        let temp = assert_fs::TempDir::new().unwrap();
        let db_file = temp.child("test_sort_by_task.ron");
        let file_path = db_file.to_str().unwrap();

        let db = Database::create_from_string(file_path, DATABASE_PRELOAD).unwrap();
        let result = db.get_sorted(SortByField::Task, SortByDirection::Ascending);
        assert_ne!(db.streaks.clone()[..], result[..]);

        temp.close().unwrap();
    }
}

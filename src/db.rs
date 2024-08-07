use crate::gui::Filter;
use crate::streak::Streak;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

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

    pub fn create_from_string(filename: &str, data: &str) -> Result<Self, std::io::Error> {
        let db = Self::new(filename)?;
        let streaks: HashMap<Uuid, Streak> = ron::de::from_str(data).unwrap();
        let mut db_streaks = db.streaks.lock().unwrap();
        for (id, streak) in streaks {
            db_streaks.insert(id, streak);
        }
        Ok(db.clone())
    }

    pub fn add(&mut self, streak: Streak) -> Result<(), std::io::Error> {
        let mut streaks = self.streaks.lock().unwrap();
        streaks.insert(streak.id.clone(), streak);
        Ok(())
    }

    pub fn update(&mut self, id: Uuid, streak: Streak) -> Result<(), std::io::Error> {
        let mut streaks = self.streaks.lock().unwrap();
        let old_streak: &mut Streak = streaks.get_mut(&id).unwrap();
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

    pub fn get_by_index(&mut self, index: usize) -> Option<Streak> {
        let streaks = self.streaks.lock().unwrap();
        let streak = streaks.values().nth(index);
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SavedState {
    pub input_value: String,
    pub filter: Filter,
    pub tasks: Vec<crate::gui::Task>,
}

#[derive(Clone, Debug)]
pub enum LoadError {
    File,
    Format,
}

#[derive(Clone, Debug)]
pub enum SaveError {
    File,
    Write,
    Format,
}

#[cfg(not(target_arch = "wasm32"))]
impl SavedState {
    fn path() -> std::path::PathBuf {
        let mut path = if let Some(project_dirs) =
            directories_next::ProjectDirs::from("com", "thekennethlove", "Skidmarks")
        {
            project_dirs.data_dir().into()
        } else {
            std::env::current_dir().unwrap_or_default()
        };

        path.push("skidmarks.ron");

        path
    }

    pub async fn load() -> Result<SavedState, LoadError> {
        use async_std::prelude::*;

        let mut contents = String::new();

        let mut file = async_std::fs::File::open(Self::path())
            .await
            .map_err(|_| LoadError::File)?;

        file.read_to_string(&mut contents)
            .await
            .map_err(|_| LoadError::File)?;

        ron::de::from_str(&contents).map_err(|_| LoadError::Format)
    }

    pub async fn save(self) -> Result<(), SaveError> {
        use async_std::prelude::*;

        let ron = ron::ser::to_string(&self).map_err(|_| SaveError::Format)?;

        let path = Self::path();

        if let Some(dir) = path.parent() {
            async_std::fs::create_dir_all(dir)
                .await
                .map_err(|_| SaveError::File)?;
        }

        {
            let mut file = async_std::fs::File::create(path)
                .await
                .map_err(|_| SaveError::File)?;

            file.write_all(ron.as_bytes())
                .await
                .map_err(|_| SaveError::Write)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;

    use super::*;

    const DATABASE_PRELOAD: &str = r#" {"00e8a16c-0edd-4e90-8c3f-2ee7aa6a2210":(id:"00e8a16c-0edd-4e90-8c3f-2ee7aa6a2210",task:"Poop",frequency:Daily,last_checkin:Some("2024-08-06"),total_checkins:1),"77cbbb3f-2690-45a9-9a30-94a53556d93e":(id:"77cbbb3f-2690-45a9-9a30-94a53556d93e",task:"Take a walk",frequency:Daily,last_checkin:Some("2024-08-06"),total_checkins:1),"af1f4cc5-87b1-40b1-9fa3-2e0344d35d3b":(id:"af1f4cc5-87b1-40b1-9fa3-2e0344d35d3b",task:"Eat brekkie",frequency:Daily,last_checkin:Some("2024-08-06"),total_checkins:1)}"#;
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
            r#"{{"{}":(id:"{}",task:"{}",frequency:Daily,last_checkin:{:?},total_checkins:{})}}"#,
            streak.id, streak.id, streak.task, streak.last_checkin, streak.total_checkins
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

        let result = db.get_all().unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&streak.id).unwrap(), &streak);

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

        let result = db.get_all().unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&streak.id).unwrap().task, "floss");

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

        db.delete(streak.id).unwrap();

        let result = db.get_all().unwrap();
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

        let result = db.get_all().unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(&streak1.id).unwrap(), &streak1);
        assert_eq!(result.get(&streak2.id).unwrap(), &streak2);

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
        let expected = db.streaks.lock().unwrap().values().nth(1).unwrap().clone();
        assert_eq!(expected, result);

        temp.close().unwrap();
    }
}

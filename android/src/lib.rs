use std::path::Path;
use std::str::FromStr;
use app_dirs2::*;
use dioxus::prelude::*;
use rusqlite::Connection;
use rusqlite::types::{FromSql, FromSqlResult, ValueRef};
use serde::{Serialize, Deserialize};
use streak::filtering::FilterByStatus;
use streak::sorting::{SortByDirection, SortByField};
use streak::{filter_by_status, sort_streaks, Streak, Frequency};

pub mod components;

const APP_INFO: AppInfo = AppInfo { name: "Skidmarks", author: "thekennethlove" };

pub fn get_database_path() -> String {
    let path = Path::new(&get_app_root(AppDataType::UserData, &APP_INFO).expect("no UserData directory")).join("db.sqlite3");
    path.to_string_lossy().to_string()
}

pub fn create_db() -> Result<(), rusqlite::Error> {
    let conn = Connection::open(get_database_path())?;
    conn.execute(r#"CREATE TABLE IF NOT EXISTS streak (
    id TEXT PRIMARY KEY,
    task TEXT NOT NULL,
    frequency TEXT NOT NULL,
    last_checkin DATE,
    current_streak INTEGER NOT NULL,
    longest_streak INTEGER NOT NULL,
    total_checkins INTEGER NOT NULL,
    )"#, ())?;
    Ok(())
}


#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppState {
    streaks: Vec<Streak>,
    sort_by_field: SortByField,
    sort_by_direction: SortByDirection,
    filter_by: FilterByStatus,
    dark_mode: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            streaks: vec![],
            sort_by_field: SortByField::LastCheckIn,
            sort_by_direction: SortByDirection::Ascending,
            filter_by: FilterByStatus::All,
            dark_mode: false,
        }
    }
}

impl AppState {
    fn new() -> Result<Self, rusqlite::Error> {
        dioxus_logger::tracing::info!("initializing app state");
        let conn = Connection::open(get_database_path())?;
        let mut stmt = conn.prepare("SELECT * from streak")?;
        let streak_iter = stmt.query_map([], |row| {
            let mut streak: Streak = Streak::default();

            match row.get(2) {
                Ok(Frequency::Daily) => {
                    streak.frequency = Frequency::Daily;
                },
                Ok(Frequency::Weekly) => {
                    streak.frequency = Frequency::Weekly;
                },
                _ =>  {}
            }

            streak.last_checkin = row.get(3)?;
            streak.current_streak = row.get(4)?;
            streak.longest_streak = row.get(5)?;
            streak.total_checkins = row.get(6)?;
            streak.id = row.get(1)?;

            Ok(streak)
        });

        let mut state = Self::default();
        state.streaks = streak_iter?.collect::<Result<Vec<_>, _>>()?;
        Ok(state)

    }

    fn get_sorted_streaks(&self) -> Vec<Streak> {
        let streaks = self.streaks.clone();
        let streaks = sort_streaks(streaks, &self.sort_by_field, &self.sort_by_direction);
        let streaks = filter_by_status(streaks, &self.filter_by);
        streaks
    }

    fn check_in_streak(&mut self, streak: &Streak) -> Streak {
        let mut streak = streak.clone();
        streak.checkin();
        streak
    }

    fn update_streak(&mut self, streak: &Streak) -> Vec<Streak> {
        let mut streaks = self.streaks.clone();
        let Some(index) = streaks.iter().position(|s| s.id == streak.id) else {
            return streaks;
        };

        streaks[index] = streak.clone();
        streaks
    }

    fn remove_streak(&self, streak: &Streak) {
        let mut storage: Signal<AppState> = use_context();
        let mut streak_signal: Signal<Vec<Streak>> = use_context();

        let mut streaks = storage.read().streaks.clone();
        streaks.retain(|s| s.id != streak.id);

        let mut state = storage.read().clone();
        state.streaks = streaks.clone();
        storage.set(state);
        streak_signal.set(streaks.clone());
    }
}

use serde::{Deserialize, Serialize};
use std::fmt::Display;

use chrono::{Local, NaiveDate};
use clap::ValueEnum;

#[derive(Clone, Debug, PartialEq, ValueEnum, Serialize, Deserialize)]
pub enum Frequency {
    Daily,
    Weekly,
}

impl Display for Frequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Frequency::Daily => write!(f, "daily"),
            Frequency::Weekly => write!(f, "weekly"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Streak {
    pub task: String,
    pub frequency: Frequency,
    pub last_checkin: NaiveDate,
}

impl Streak {
    pub fn new_daily(name: String) -> Self {
        let date = Local::now();
        let mut streaks: Vec<Streak> = load_streaks();
        let streak = Self {
            task: name,
            frequency: Frequency::Daily,
            last_checkin: date.date_naive(),
        };
        streaks.push(streak.clone());
        save_streaks(streaks);
        streak
    }

    pub fn new_weekly(name: String) -> Self {
        let date = Local::now();
        Self {
            task: name,
            frequency: Frequency::Weekly,
            last_checkin: date.date_naive(),
        }
    }

    pub fn checkin(&mut self) {
        let date = Local::now().date_naive();
        self.last_checkin = date;
    }

    pub fn was_missed(self) -> bool {
        let today = Local::now().date_naive();
        let duration = today - self.last_checkin;
        match self.frequency {
            Frequency::Daily => duration.num_days() > 1,
            Frequency::Weekly => duration.num_days() > 7,
        }
    }
}

use std::fs::{File, OpenOptions};
use std::io::Write;

fn create_database_if_new() -> std::io::Result<()> {
    let filename = "streaks.ron";
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
        },
        Err(err) => return Err(err),
    };

    if metadata.len() == 0 {
        let mut file = File::open(filename)?;
        file.write_all(data)?;
    }
    Ok(())
}

fn save_streaks(streaks: Vec<Streak>) {
    let ronned = ron::ser::to_string(&streaks).unwrap();
    let _ = create_database_if_new();
    std::fs::write("streaks.ron", ronned).unwrap();
}

fn load_streaks() -> Vec<Streak> {
    let _ = create_database_if_new();
    let ronned = std::fs::read_to_string("streaks.ron").unwrap();
    let ronned: Vec<Streak> = ron::de::from_str(&ronned).expect("Failed to load streaks");
    dbg!(&ronned);
    ronned
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;

    #[test]
    fn new_daily_streak() {
        let streak = Streak::new_daily("Test Streak".to_string());
        let today = Local::now();
        assert_eq!(streak.task, "Test Streak");
        assert_eq!(streak.frequency, Frequency::Daily);
        assert_eq!(streak.last_checkin, today.date_naive());
    }

    #[test]
    fn new_weekly_streak() {
        let streak = Streak::new_weekly("Test Streak".to_string());
        assert_eq!(streak.frequency, Frequency::Weekly);
        assert_eq!(streak.last_checkin, Local::now().date_naive());
    }

    #[test]
    fn update_checkin() {
        let old_date = NaiveDate::from_ymd_opt(2020, 4, 20).unwrap();
        let mut streak = Streak::new_daily("Test Streak".to_string());
        streak.last_checkin = old_date;
        assert_eq!(streak.last_checkin, old_date);

        streak.checkin();
        assert!(streak.last_checkin != old_date);
    }

    #[test]
    fn daily_streak_broken() {
        let old_date = NaiveDate::from_ymd_opt(2020, 4, 20).unwrap();
        let mut streak = Streak::new_daily("Test Streak".to_string());
        streak.last_checkin = old_date;
        assert_eq!(streak.last_checkin, old_date);
        assert!(streak.was_missed())
    }

    #[test]
    fn weekly_streak_broken() {
        let old_date = NaiveDate::from_ymd_opt(2020, 4, 20).unwrap();
        let mut streak = Streak::new_weekly("Test Streak".to_string());
        streak.last_checkin = old_date;
        assert_eq!(streak.last_checkin, old_date);
        assert!(streak.was_missed())
    }

    #[test]
    fn weekly_streak_unbroken() {
        let today = Local::now();
        let yesterday = today - TimeDelta::days(7);
        let mut streak = Streak::new_weekly("Test Streak".to_string());
        streak.last_checkin = yesterday.date_naive();
        assert!(!streak.was_missed())
    }
}

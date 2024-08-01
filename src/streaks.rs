#[allow(unused_imports)]
use chrono::{Local, NaiveDate};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

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

impl Frequency {
    pub fn from_str(s: &str) -> Self {
        match s {
            "daily" => Frequency::Daily,
            "weekly" => Frequency::Weekly,
            _ => panic!("Invalid frequency"),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Frequency::Daily => "daily".to_string(),
            Frequency::Weekly => "weekly".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Waiting,
    Done,
    Missed,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Waiting => write!(f, "waiting"),
            Status::Done => write!(f, "done"),
            Status::Missed => write!(f, "missed"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Streak {
    pub task: String,
    pub frequency: Frequency,
    pub last_checkin: Option<NaiveDate>,
    pub total_checkins: u32,
}

impl Streak {
    pub fn new_daily(name: String) -> Self {
        Self {
            task: name,
            frequency: Frequency::Daily,
            last_checkin: None,
            total_checkins: 0,
        }
    }

    pub fn new_weekly(name: String) -> Self {
        Self {
            task: name,
            frequency: Frequency::Weekly,
            last_checkin: None,
            total_checkins: 0,
        }
    }

    pub fn checkin(&mut self) {
        let date = Local::now().date_naive();
        self.last_checkin = Some(date);
        self.total_checkins += 1;
    }

    pub fn was_missed(&self) -> bool {
        let today = Local::now().date_naive();
        let duration = match self.last_checkin {
            Some(date) => today - date,
            None => return true,
        };
        match &self.frequency {
            Frequency::Daily => duration.num_days() > 1,
            Frequency::Weekly => duration.num_days() > 7,
        }
    }

    pub fn done_in_period(&self) -> bool {
        let today = Local::now().date_naive();
        let duration = match self.last_checkin {
            Some(date) => today - date,
            None => return false,
        };
        match &self.frequency {
            Frequency::Daily => duration.num_days() == 0,
            Frequency::Weekly => duration.num_days() < 6,
        }
    }

    pub fn status(&self) -> Status {
        if self.was_missed() {
            Status::Missed
        } else if self.done_in_period() {
            Status::Done
        } else {
            Status::Waiting
        }
    }

    pub fn emoji_status(&self) -> String {
        match self.status() {
            Status::Done => "✅".to_string(),
            Status::Missed => "❌".to_string(),
            Status::Waiting => "⏳".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeDelta};

    #[test]
    fn status_waiting() {
        let mut streak = Streak::new_daily("Test Streak".to_string());
        streak.last_checkin = Some(Local::now().date_naive() - TimeDelta::days(1));
        assert_eq!(streak.status(), Status::Waiting);
    }

    #[test]
    fn status_waiting_weekly() {
        let mut streak = Streak::new_weekly("Test Streak".to_string());
        streak.last_checkin = Some(Local::now().date_naive() - TimeDelta::days(7));
        assert_eq!(streak.status(), Status::Waiting);
    }

    #[test]
    fn status_done() {
        let mut streak = Streak::new_weekly("Test Streak".to_string());
        streak.last_checkin = Some(Local::now().date_naive() - TimeDelta::days(3));
        assert_eq!(streak.status(), Status::Done);
    }

    #[test]
    fn new_daily_streak() {
        let streak = Streak::new_daily("Test Streak".to_string());
        assert_eq!(streak.task, "Test Streak");
        assert_eq!(streak.frequency, Frequency::Daily);
        assert_eq!(streak.last_checkin, None);
        assert_eq!(streak.total_checkins, 0);
    }

    #[test]
    fn new_weekly_streak() {
        let streak = Streak::new_weekly("Test Streak".to_string());
        assert_eq!(streak.frequency, Frequency::Weekly);
        assert_eq!(streak.last_checkin, None);
    }

    #[test]
    fn update_checkin() {
        let old_date = NaiveDate::from_ymd_opt(2020, 4, 20).unwrap();
        let mut streak = Streak::new_daily("Test Streak".to_string());
        streak.last_checkin = Some(old_date);
        streak.total_checkins = 1;

        streak.checkin();
        assert!(streak.last_checkin.unwrap() != old_date);
        assert_eq!(streak.total_checkins, 2);
    }

    #[test]
    fn daily_streak_broken() {
        let old_date = NaiveDate::from_ymd_opt(2020, 4, 20).unwrap();
        let mut streak = Streak::new_daily("Test Streak".to_string());
        streak.last_checkin = Some(old_date);
        assert_eq!(streak.last_checkin.unwrap(), old_date);
        assert!(streak.was_missed())
    }

    #[test]
    fn weekly_streak_broken() {
        let old_date = NaiveDate::from_ymd_opt(2020, 4, 20).unwrap();
        let mut streak = Streak::new_weekly("Test Streak".to_string());
        streak.last_checkin = Some(old_date);
        assert_eq!(streak.last_checkin.unwrap(), old_date);
        assert!(streak.was_missed())
    }

    #[test]
    fn weekly_streak_unbroken() {
        let today = Local::now();
        let yesterday = today - TimeDelta::days(7);
        let mut streak = Streak::new_weekly("Test Streak".to_string());
        streak.last_checkin = Some(yesterday.date_naive());
        assert!(!streak.was_missed())
    }
}

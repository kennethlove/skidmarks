use chrono::{NaiveDate, Local, TimeDelta};

#[derive(Debug, PartialEq)]
enum Frequency {
    Daily,
    Weekly,
}

#[derive(Debug)]
struct Streak {
    task: String,
    frequency: Frequency,
    last_checkin: NaiveDate,
}

impl Streak {
    fn new_daily(name: String) -> Self {
        let date = Local::now();
        Self {
            task: name,
            frequency: Frequency::Daily,
            last_checkin: date.date_naive()
        }
    }

    fn new_weekly(name: String) -> Self {
        let date = Local::now();
        Self {
            task: name,
            frequency: Frequency::Weekly,
            last_checkin: date.date_naive()
        }
    }

    fn checkin(&mut self) {
        let date = Local::now().date_naive();
        self.last_checkin = date;
    }

    fn was_missed(self) -> bool {
        let today = Local::now().date_naive();
        let duration = today - self.last_checkin;
        match self.frequency {
            Frequency::Daily => {
                duration.num_days() > 1
            },
            Frequency::Weekly => {
                duration.num_days() > 7
            }
        }
    }
}

fn main() {
    // Get input from user
    // Convert to task
    // Serialize and save
    // Load
    // Alert to any streaks left for the day
}

#[cfg(test)]
mod tests {
    use super::*;

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

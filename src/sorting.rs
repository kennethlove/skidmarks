use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum SortByField {
    Task,
    Frequency,
    Status,
    LastCheckIn,
    CurrentStreak,
    LongestStreak,
    TotalCheckins,
}

impl Display for SortByField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortByField::Task => write!(f, "task"),
            SortByField::Frequency => write!(f, "frequency"),
            SortByField::Status => write!(f, "status"),
            SortByField::LastCheckIn => write!(f, "last_checkin"),
            SortByField::CurrentStreak => write!(f, "current_streak"),
            SortByField::LongestStreak => write!(f, "longest_streak"),
            SortByField::TotalCheckins => write!(f, "total_checkins"),
        }
    }
}

impl SortByField {
    pub fn to_string(&self) -> String {
        match self {
            SortByField::Task => "task".to_string(),
            SortByField::Frequency => "frequency".to_string(),
            SortByField::Status => "status".to_string(),
            SortByField::LastCheckIn => "last_checkin".to_string(),
            SortByField::CurrentStreak => "current_streak".to_string(),
            SortByField::LongestStreak => "longest_streak".to_string(),
            SortByField::TotalCheckins => "total_checkins".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "task" => SortByField::Task,
            "frequency" => SortByField::Frequency,
            "status" => SortByField::Status,
            "last_checkin" => SortByField::LastCheckIn,
            "current_streak" => SortByField::CurrentStreak,
            "longest_streak" => SortByField::LongestStreak,
            "total_checkins" => SortByField::TotalCheckins,
            _ => SortByField::Task,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SortByDirection {
    Ascending,
    Descending,
}

impl SortByDirection {
    pub fn emoji(&self) -> &str {
        match self {
            SortByDirection::Ascending => "⬆",
            SortByDirection::Descending => "⬇",
        }
    }
}

pub fn get_sort_order(sort_by: &str) -> (SortByField, SortByDirection) {
    let sign = match sort_by.chars().rev().next() {
        Some('+') => SortByDirection::Ascending,
        Some('-') => SortByDirection::Descending,
        _ => SortByDirection::Ascending,
    };

    let ln = sort_by.len() - 1;
    let field = match sort_by[..ln].to_lowercase().as_str() {
        "task" => SortByField::Task,
        "streak" => SortByField::Task,
        "name" => SortByField::Task,
        "frequency" => SortByField::Frequency,
        "freq" => SortByField::Frequency,
        "status" => SortByField::Status,
        "last_checkin" => SortByField::LastCheckIn,
        "last-checkin" => SortByField::LastCheckIn,
        "last" => SortByField::LastCheckIn,
        "current_streak" => SortByField::CurrentStreak,
        "current-streak" => SortByField::CurrentStreak,
        "current" => SortByField::CurrentStreak,
        "longest_streak" => SortByField::LongestStreak,
        "longest-streak" => SortByField::LongestStreak,
        "longest" => SortByField::LongestStreak,
        "total_checkins" => SortByField::TotalCheckins,
        "total-checkins" => SortByField::TotalCheckins,
        "total" => SortByField::TotalCheckins,
        _ => SortByField::Task,
    };

    (field, sign)
}

#[cfg(test)]
mod tests {
    use super::{get_sort_order, SortByDirection, SortByField};
    #[test]
    fn test_single_sort_order() {
        let sort = "task+";
        let (field, direction) = get_sort_order(sort);
        assert_eq!(field, SortByField::Task);
        assert_eq!(direction, SortByDirection::Ascending);
    }
}

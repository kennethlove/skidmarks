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

#[derive(Clone, Debug, PartialEq)]
pub enum SortByDirection {
    Ascending,
    Descending,
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

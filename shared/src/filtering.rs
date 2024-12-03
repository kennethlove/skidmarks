use crate::streak::{Status, Streak};

#[derive(Clone, Debug, PartialEq)]
pub enum FilterByStatus {
    All,
    Done,
    Missed,
    Waiting,
}

impl FilterByStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "All" => FilterByStatus::All,
            "Done" => FilterByStatus::Done,
            "Missed" => FilterByStatus::Missed,
            "Waiting" => FilterByStatus::Waiting,
            _ => FilterByStatus::All,
        }
    }
}

pub fn filter_by_status(streaks: Vec<Streak>, status: FilterByStatus) -> Vec<Streak> {
    match status {
        FilterByStatus::All => streaks,
        FilterByStatus::Done => streaks
            .into_iter()
            .filter(|streak| streak.status() == Status::Done)
            .collect(),
        FilterByStatus::Missed => streaks
            .into_iter()
            .filter(|streak| streak.status() == Status::Missed)
            .collect(),
        FilterByStatus::Waiting => streaks
            .into_iter()
            .filter(|streak| streak.status() == Status::Waiting)
            .collect(),
    }
}

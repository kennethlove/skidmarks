use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum FilterByStatus {
    All,
    Done,
    Missed,
    Waiting,
}

impl FilterByStatus {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "all" => FilterByStatus::All,
            "done" => FilterByStatus::Done,
            "missed" => FilterByStatus::Missed,
            "waiting" => FilterByStatus::Waiting,
            _ => FilterByStatus::All,
        }
    }
}

impl Display for FilterByStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterByStatus::All => write!(f, "all"),
            FilterByStatus::Done => write!(f, "done"),
            FilterByStatus::Missed => write!(f, "missed"),
            FilterByStatus::Waiting => write!(f, "waiting")
        }
    }
}

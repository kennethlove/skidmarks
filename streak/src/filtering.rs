#[derive(Clone, Debug, PartialEq)]
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

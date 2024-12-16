use serde::{Deserialize, Serialize};
use streak::filtering::FilterByStatus;
use streak::sorting::{SortByDirection, SortByField};
use streak::Streak;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppState {
    streaks: Vec<Streak>,
    #[default]
    sort_by_field: SortByField,
    #[default]
    sort_by_direction: SortByDirection,
    #[default]
    filter_by: FilterByStatus,
    #[default]
    dark_mode: bool,
}

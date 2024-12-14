use serde::{Deserialize, Serialize};
use streak::filtering::FilterByStatus;
use streak::sorting::{SortByDirection, SortByField};
use streak::Streak;
use uuid::Uuid;

pub trait State {
    fn checkin_streak(&self, streak_id: Uuid);
    fn get_streaks(&self);
    fn remove_streak(&self, streak_id: Uuid);
}
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct DefaultAppState {
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

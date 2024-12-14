pub mod components;

use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use streak::filtering::FilterByStatus;
use streak::sorting::{SortByDirection, SortByField};
use streak::{filter_by_status, sort_streaks, Streak};
use uuid::Uuid;

/// A persistent storage hook that can be used to store data across application reloads.
#[allow(clippy::needless_return)]
pub fn use_persistent<T: Serialize + DeserializeOwned + Default + 'static>(
    // A unique key for the storage entry
    key: impl ToString,
    // A function that returns the initial value if the storage entry is empty
    init: impl FnOnce() -> T,
) -> UsePersistent<T> {
    // Use the use_signal hook to create a mutable state for the storage entry
    let state = use_signal(move || {
        // This closure will run when the hook is created
        let key = key.to_string();
        let value = LocalStorage::get(key.as_str()).ok().unwrap_or_else(init);
        StorageEntry { key, value }
    });

    // Wrap the state in a new struct with a custom API
    UsePersistent { inner: state }
}

struct StorageEntry<T> {
    key: String,
    value: T,
}

/// Storage that persists across application reloads
pub struct UsePersistent<T: 'static> {
    inner: Signal<StorageEntry<T>>,
}

impl<T> Clone for UsePersistent<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for UsePersistent<T> {}

impl<T: Serialize + DeserializeOwned + Clone + 'static> UsePersistent<T> {
    /// Returns a reference to the value
    pub fn get(&self) -> T {
        self.inner.read().value.clone()
    }

    /// Sets the value
    pub fn set(&mut self, value: T) {
        let mut inner = self.inner.write();
        // Write the new value to local storage
        LocalStorage::set(inner.key.as_str(), &value).expect("unable to write to local storage");
        inner.value = value;
    }
}

fn get_saved_state(storage: UsePersistent<AppState>) -> AppState {
    //Signal<AppState> {
    let mut state = AppState::default();
    if !storage.get().streaks.is_empty() {
        state = storage.get();
    }
    state
    // Signal::new(state)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppState {
    streaks: Vec<Streak>,
    sort_by_field: SortByField,
    sort_by_direction: SortByDirection,
    filter_by: FilterByStatus,
    dark_mode: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            streaks: vec![],
            sort_by_field: SortByField::LastCheckIn,
            sort_by_direction: SortByDirection::Ascending,
            filter_by: FilterByStatus::All,
            dark_mode: false,
        }
    }
}

impl AppState {
    fn toggle_dark_mode(&mut self) {
        self.dark_mode = !self.dark_mode;
    }

    fn get_sorted_streaks(&self) -> Vec<Streak> {
        let streaks = self.streaks.clone();
        let streaks = sort_streaks(streaks, &self.sort_by_field, &self.sort_by_direction);
        let streaks = filter_by_status(streaks, &self.filter_by);
        streaks
    }

    fn check_in_streak(&mut self, streak: &Streak) -> Streak {
        let mut streak = streak.clone();
        streak.checkin();
        streak
    }

    fn update_streak(&mut self, streak: &Streak) -> Vec<Streak> {
        let mut streaks = self.streaks.clone();
        let Some(index) = streaks.iter().position(|s| s.id == streak.id) else {
            return streaks;
        };

        streaks[index] = streak.clone();
        streaks
    }

    fn remove_streak(&self, streak: &Streak) {
        let mut storage = use_persistent("skidmarks", || AppState::default());
        let mut streak_signal: Signal<Vec<Streak>> = use_context();

        let mut streaks = storage.get().streaks.clone();
        streaks.retain(|s| s.id != streak.id);

        let mut state = storage.get();
        state.streaks = streaks.clone();
        storage.set(state);
        streak_signal.set(streaks.clone());
    }
}

use crate::components::FilterButton;
use crate::AppState;
use dioxus::prelude::*;
use streak::filtering::FilterByStatus;

#[component]
pub fn StreakFilters() -> Element {
    // let mut state = use_context::<Signal<AppState>>();
    // let filter_by = state.read().filter_by.clone();

    rsx! {
        div {
            class: "lg:pl-4 self-center",
            fieldset {
                class: "inline-flex rounded-lg border border-pink-100 bg-purple-50/75 p-1 dark:bg-purple-200/75 dark:border-purple-100 select-none",
                legend {
                    class: "sr-only",
                    "Frequency"
                }
                FilterButton { button_text: "All".to_string(), button_status: FilterByStatus::All }
                FilterButton { button_text: "Waiting".to_string(), button_status: FilterByStatus::Waiting }
                FilterButton { button_text: "Missed".to_string(), button_status: FilterByStatus::Missed }
                FilterButton { button_text: "Done".to_string(), button_status: FilterByStatus::Done }
            }
        }
    }
}

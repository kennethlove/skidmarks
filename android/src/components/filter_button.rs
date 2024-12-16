use crate::{AppState};
use dioxus::prelude::*;
use std::clone::Clone;
use streak::filtering::FilterByStatus;
use streak::Streak;

#[derive(Debug, Clone, Props, PartialEq)]
pub struct FilterButtonProps {
    button_text: String,
    button_status: FilterByStatus,
}

#[component]
pub fn FilterButton(props: FilterButtonProps) -> Element {
    let mut storage: Signal<AppState> = use_context();
    let mut streak_signal: Signal<Vec<Streak>> = use_context();

    rsx! {
        div {
            label {
                class: "transition cursor-pointer inline-block rounded-md px-4 py-2 text-sm text-purple-500 dark:text-purple-900 hover:text-purple-700 focus:relative has-[:checked]:border-purple-500 has-[:checked]:bg-purple-500 has-[:checked]:text-pink-100 dark:has-[:checked]:border-purple-900 dark:has-[:checked]:bg-purple-900",
                input {
                    class: "sr-only",
                    name: "filter",
                    r#type: "radio",
                    checked: storage.read().filter_by == props.button_status.clone(),
                    onclick: move |e| {
                        let mut state = storage.read().clone();
                        state.filter_by = props.button_status.clone();
                        storage.set(state);
                        streak_signal.set(storage.read().get_sorted_streaks());
                    },
                }
                p {
                    class: "text-sm font-medium",
                    "{props.button_text}",
                }
            }
        }

    }
}

use crate::{use_persistent, AppState};
use dioxus::prelude::*;
use streak::Streak;

#[component]
pub fn CheckInButton(streak: Streak) -> Element {
    // let mut storage = use_persistent("skidmarks", || AppState::default());
    // let mut state = use_context::<Signal<AppState>>();

    rsx! {
        button {
            class: "cursor-pointer select-none hover:text-green-500 focus:text-green-500 focus:outline-none",
            onclick: move |e| {
                // state.write().check_in_streak(&streak);
                // storage.set(state.read().clone());
            },
            span {
                class: "material-symbols-rounded",
                title: "Check-in",
                "check_circle"
            }
        }
    }
}

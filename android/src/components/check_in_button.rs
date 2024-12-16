use crate::{AppState};
use dioxus::prelude::*;
use streak::Streak;

#[component]
pub fn CheckInButton(streak: Streak) -> Element {
    let mut storage: Signal<AppState> = use_context();
    let mut streak_signal: Signal<Vec<Streak>> = use_context();

    rsx! {
        button {
            class: "cursor-pointer select-none hover:text-green-500 focus:text-green-500 focus:outline-none",
            onclick: move |e| {
                let mut state = storage.read().clone();
                let streak = state.check_in_streak(&streak);
                state.streaks = state.update_streak(&streak);

                streak_signal.set(state.streaks.clone());

                storage.set(state);
            },
            span {
                class: "material-symbols-rounded",
                title: "Check-in",
                "check_circle"
            }
        }
    }
}

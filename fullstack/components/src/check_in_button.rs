use crate::state::AppStateExt;
use dioxus::prelude::*;
use streak::Streak;

#[component]
pub fn CheckInButton(streak: Streak) -> Element {
    let state = use_context::<Signal<dyn AppStateExt>>();
    rsx! {
        button {
            class: "cursor-pointer select-none hover:text-green-500 focus:text-green-500 focus:outline-none",
            onclick: move |e| {state.read().checkin_streak(streak.id)},
            span {
                class: "material-symbols-rounded",
                title: "Check-in",
                "check_circle"
            }
        }
    }
}

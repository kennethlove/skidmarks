use crate::state::AppStateExt;
use dioxus::prelude::*;
use streak::Streak;

#[component]
pub fn DeleteButton(streak: Streak) -> Element {
    let state = use_context::<Signal<dyn AppStateExt>>();
    let mut modal_signal: Signal<Option<Streak>> = use_context();
    rsx! {
        button {
            class: "cursor-pointer select-none hover:text-red-500 focus:text-red-500 focus:outline-none",
            onclick: move |_| {
                modal_signal.set(Some(streak.clone()));
            },
            span {
                class: "material-symbols-rounded",
                title: "Delete",
                "delete"
            }
        }
    }
}

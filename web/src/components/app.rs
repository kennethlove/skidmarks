use crate::components::{DeleteModal, StreakFilters, StreakForm, StreakTable};
use crate::{get_saved_state, use_persistent, AppState};
use dioxus::prelude::*;
use streak::Streak;

#[component]
pub fn App() -> Element {
    let mut storage = use_persistent("skidmarks", || AppState::default());
    let streak_signal = use_context_provider(|| Signal::new(storage.get().streaks.clone()));

    let dark_mode: Signal<bool> = Signal::new(storage.get().dark_mode);

    let modal_state: Signal<Option<Streak>> = Signal::new(None);
    let modal_state = use_context_provider(|| modal_state);

    let mut classes =
        "bg-gradient-to-br from-pink-300 to-indigo-300 dark:from-emerald-950 dark:to-purple-900 min-h-full min-h-screen transition duration-150 selection:text-pink-200 selection:bg-purple-500 dark:selection:bg-emerald-200 dark:selection:text-purple-800"
            .to_string();

    if *dark_mode.read() {
        classes.push_str(" dark")
    }

    rsx! {
        document::Link { href: "https://fonts.googleapis.com", rel: "preconnect" }
        document::Link { href: "https://fonts.gstatic.com", rel: "preconnect", crossorigin: "true" }
        document::Stylesheet { href: asset!("/assets/main.css") }
        div {
            class: classes,
            div {
                class: "container mx-auto sm:w-7/8 lg:w-10/12 xs:w-full pt-5",
                div {
                    class: "flex flex-row flex-nowrap justify-between sm:justify-center relative",
                    h1 {
                        class: "text-3xl font-bold text-purple-900 dark:text-teal-500",
                        "Skidmarks"
                    }
                    button {
                        class: "cursor-pointer sm:absolute sm:right-0 sm:top-2",
                        onclick: move |_| {
                            let mut state = storage.get();
                            state.toggle_dark_mode();
                            storage.set(state);
                        },
                        span {
                            class: "material-symbols-rounded dark:text-teal-500 text-purple-500 hover:text-pink-50 dark:hover:text-teal-200",
                            title: "Switch to dark/light mode",
                            "contrast"
                        }
                    }
                }
                div {
                    class: "flex flex-col lg:flex-row flex-wrap lg:flex-nowrap gap-3 py-4 lg:justify-center",
                    StreakForm {}
                    StreakFilters {}
                }

                StreakTable {}

                footer {
                    class: "mt-4",
                    p {
                        class: "text-center text-sm text-pink-50",
                        "Made with 💜 by "
                        a {
                            class: "underline",
                            href: "https://thekennethlove.com",
                            target: "_new",
                            "klove"
                        }
                    }
                }
            }
            DeleteModal {}
        }
    }
}

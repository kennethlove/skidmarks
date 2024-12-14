use crate::{DeleteModal, StreakFilters, StreakForm, StreakTable};
use dioxus::prelude::*;
use shared::state::State;
use streak::Streak;

#[component]
pub fn App(state: impl State) -> Element {
    let state = Signal::new(state::default());
    let mut state: Signal<state> = use_context_provider(|| state);
    let filter = state.read().filter_by.clone();

    let modal_state: Signal<Option<Streak>> = Signal::new(None);
    let modal_state = use_context_provider(|| modal_state);

    state.read().get_streaks();

    let mut classes =
        "bg-gradient-to-br from-pink-300 to-indigo-300 dark:from-emerald-950 dark:to-purple-900 min-h-full min-h-screen transition duration-150 selection:text-pink-200 selection:bg-purple-500 dark:selection:bg-emerald-200 dark:selection:text-purple-800"
            .to_string();
    if state.read().dark_mode {
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
                            let dark = state.read().dark_mode.clone();
                            match dark {
                                true => {
                                    state.write().dark_mode = false;
                                    eval("document.querySelector('html').classList.remove('dark')");
                                },
                                false => {
                                    state.write().dark_mode = true;
                                    eval("document.querySelector('html').classList.add('dark')");
                                }
                            };
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

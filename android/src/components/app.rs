use dioxus::prelude::*;
use streak::Streak;
// use crate::AppState;
// use crate::components::{DeleteModal, StreakFilters, StreakForm, StreakTable};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
pub fn App() -> Element {
    // let state = AppState::new();
    // let storage: Signal<AppState> = use_context_provider(|| Signal::new(state.unwrap()));
    // let streak_signal: Signal<Vec<Streak>> = use_context_provider(|| Signal::new(storage.read().clone().streaks.clone()));
    //
    // let modal_state: Signal<Option<Streak>> = Signal::new(None);
    // let modal_state = use_context_provider(|| modal_state);

    rsx! {
        document::Link { href: "https://fonts.googleapis.com", rel: "preconnect" }
        document::Link { href: "https://fonts.gstatic.com", rel: "preconnect", crossorigin: "true" }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        div {
            class:"bg-gradient-to-br from-pink-300 to-indigo-300 dark:from-emerald-950 dark:to-purple-900 min-h-full min-h-screen transition duration-150 selection:text-pink-200 selection:bg-purple-500 dark:selection:bg-emerald-200 dark:selection:text-purple-800",
            div {
                class: "container mx-auto sm:w-7/8 lg:w-10/12 xs:w-full pt-5",
                h1 {
                    class: "text-3xl font-bold text-purple-800 dark:text-teal-500 text-center",
                    "Skidmarks"
                }
                div {
                    class: "flex flex-col lg:flex-row flex-wrap lg:flex-nowrap gap-3 py-4 lg:justify-center",
                    // StreakForm {}
                    // StreakFilters {}
                }

                // StreakTable {}

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
            // DeleteModal {}
        }
    }
}

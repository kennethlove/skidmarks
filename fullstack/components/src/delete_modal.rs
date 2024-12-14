use crate::state::AppStateExt;
use dioxus::prelude::*;
use streak::Streak;

#[component]
pub fn DeleteModal() -> Element {
    let state = use_context::<Signal<dyn AppStateExt>>();
    let mut streak: Streak = Streak::default();
    let mut modal_signal: Signal<Option<Streak>> = use_context();

    if modal_signal.read().clone().is_some() {
        streak = modal_signal.read().clone().unwrap();
    }

    rsx! {
        dialog {
            open: modal_signal.read().clone().is_some(),
            class: "relative z-10",
            role: "confirm",
            div { class: "fixed inset-0 bg-purple-500/25 dark:bg-purple-800/25 transition-opacity backdrop-blur-sm" }
            div {
                class: "fixed inset-0 z-10 w-screen h-screen overflow-y-hidden",
                div {
                    class: "flex items-center gap-4 min-h-full justify-center",
                    div {
                        class: "relative transform overflow-hidden p-2 rounded-xl",
                        div {
                            class: "mx-auto bg-pink-50 border border-purple-500 rounded-xl p-2 dark:bg-purple-300",
                            div {
                                class: "flex-1",
                                h1 {
                                    class: "block select-none text-2xl mb-4 font-medium text-purple-900 border-b-2 border-purple-500 dark:border-purple-500",
                                    "Delete this streak?"
                                }
                                blockquote {
                                    class: "p-2 italic bg-pink-200 dark:bg-purple-100 rounded-md",
                                    "{streak.task}"
                                }
                            }
                            div {
                                class: "flex flex-col justify-center mt-2 divide-y divide-purple-500 dark:divide-purple-500",
                                button {
                                    r#type: "button",
                                    class: "rounded-t-md cursor-pointer select-none py-2 hover:font-bold focus:font-bold hover:bg-purple-200 focus:bg-purple-200 focus:outline-none dark:hover:text-purple-200 dark:hover:bg-purple-500 dark:focus:bg-purple-900",
                                    onclick: move |e| {
                                        state.read().remove_streak(streak.id);
                                        modal_signal.set(None);
                                    },
                                    span {
                                        "Yes"
                                    }
                                }
                                button {
                                    r#type: "button",
                                    class: "rounded-b-md cursor-pointer select-none py-2 hover:font-bold focus:font-bold hover:bg-purple-200 focus:bg-purple-200 focus:outline-none dark:hover:text-purple-200 dark:hover:bg-purple-500 dark:focus:bg-purple-900",
                                    onclick: move |e| { modal_signal.set(None) },
                                    span {
                                        "No"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

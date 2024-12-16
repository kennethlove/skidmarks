use crate::{AppState};
use dioxus::prelude::*;
use std::string::ToString;
use streak::{Frequency, Streak};

#[component]
pub fn StreakForm() -> Element {
    let mut storage: Signal<AppState> = use_context();
    let mut streak_signal: Signal<Vec<Streak>> = use_context();

    let mut task_signal = use_signal(|| String::default());
    let mut frequency_signal = use_signal(|| Frequency::Daily);

    rsx! {
        div {
            class: "lg:w-2/3 w-full",
            form {
                class: "flex flex-row lg:flex-nowrap gap-3 lg:content-between lg:justify-stretch xs:flex-col",
                onsubmit: move |e| {
                    let mut new_streak: Streak = Streak::default();
                    let task = task_signal.read().clone();
                    if task.is_empty() {
                        return;
                    }

                    match frequency_signal.read().clone() {
                        Frequency::Daily => new_streak = Streak::new_daily(task.clone()),
                        Frequency::Weekly => new_streak = Streak::new_weekly(task.clone())
                    }

                    let mut state = storage.read().clone();
                    state.streaks.push(new_streak.clone());
                    streak_signal.write().push(new_streak.clone());
                    storage.set(state);

                    task_signal.set(String::from(""));
                    frequency_signal.set(Frequency::Daily);
                },
                label {
                    class: "grow relative block overflow-hidden rounded-md border border-pink-100 dark:border-purple-100 focus-within:border-purple-500 focus-within:ring-1 focus-within:ring-purple-500 dark:bg-purple-200/75 bg-purple-50/75 focus-within:bg-pink-50 px-3 pt-1.5 transition dark:focus-within:border-purple-100 dark:focus-within:ring-purple-100",
                    input {
                        class: "peer h-8 border-none bg-transparent focus:border-transparent focus:outline-none focus:ring-0 p-0 w-full",
                        r#type: "text",
                        name: "task",
                        oninput: move |e| {
                            task_signal.set(e.value().clone());
                        },
                        value: task_signal.read().clone(),
                        placeholder: "Task"
                    }
                }
                fieldset {
                    class: "inline-flex rounded-lg border border-pink-100 bg-purple-50/75 p-1 dark:border-purple-100 dark:bg-purple-200/75 select-none",
                    legend {
                        class: "sr-only",
                        "Frequency"
                    }
                    div {
                        label {
                            class: "transition cursor-pointer inline-block rounded-md px-4 py-2 text-sm text-purple-900 hover:text-purple-700 focus:relative hover:border-purple-500 has-[:checked]:border-purple-500 has-[:checked]:bg-purple-500 has-[:checked]:text-pink-100 focus:bg-purple-500 focus:text-gray-700 focus:border-purple-500 focus:outline-none focus:ring dark:border-purple-300 bg-transparent dark:has-[:checked]:border-purple-900 dark:has-[:checked]:bg-purple-900 bg-transparent",
                            input {
                                class: "sr-only",
                                name: "frequency",
                                r#type: "radio",
                                checked: frequency_signal.read().clone() == Frequency::Daily,
                                onclick: move |event| {
                                    frequency_signal.set(Frequency::Daily)
                                }
                            }
                            p {
                                class: "text-sm font-medium",
                                "Daily"
                            }
                        }
                    }
                    div {
                        label {
                            class: "transition cursor-pointer inline-block rounded-md px-4 py-2 text-sm text-purple-500 dark:text-purple-900 hover:text-purple-700 focus:relative hover:border-purple-500 has-[:checked]:border-purple-500 has-[:checked]:bg-purple-500 has-[:checked]:text-pink-100 focus:bg-purple-500 focus:text-purple-700 focus:border-purple-500 focus:outline-none focus:ring dark:border-purple-300 bg-transparent dark:has-[:checked]:border-purple-900 dark:has-[:checked]:bg-purple-900",
                            input {
                                class: "sr-only",
                                name: "frequency",
                                r#type: "radio",
                                checked: frequency_signal.read().clone() == Frequency::Weekly,
                                onclick: move |event| {
                                    frequency_signal.set(Frequency::Weekly)
                                }
                            }
                            p {
                                class: "text-sm font-medium",
                                "Weekly"
                            }
                        }
                    }
                }

                button {
                    class: "transition cursor-pointer inline-block rounded-lg px-4 leading-tight text-sm font-medium text-purple-500 dark:text-purple-900 bg-purple-50/75 border border-pink-100 hover:border-purple-500 hover:bg-purple-500 hover:text-pink-100 focus:bg-purple-500 focus:text-pink-100 focus:border-purple-500 focus:outline-none focus:ring dark:bg-purple-200/75 dark:hover:bg-purple-900",
                    r#type: "submit",
                    "Add"
                }
            }
        }
    }
}

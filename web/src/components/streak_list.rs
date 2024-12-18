#![allow(non_snake_case)]

use crate::components::{CheckInButton, DeleteButton};
use crate::{use_persistent, AppState};
use dioxus::prelude::*;
use streak::{Frequency, Streak};

pub fn StreakList() -> Element {
    let storage = use_persistent("skidmarks", || AppState::default());
    let streak_signal: Signal<Vec<Streak>> = use_context();
    let _direction = storage.get().sort_by_direction.clone();
    let streaks = streak_signal.read().clone();

    rsx! {
        ol {
            class: "grow flex flex-nowrap flex-col md:flex-row md:flex-wrap gap-3 h-full w-full",
            for streak in &streaks {
                StreakListItem { streak: streak.clone() }
            }
            if streaks.is_empty() {
                p {
                    "None"
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Props)]
pub struct StreakListItemProps {
    streak: Streak,
}

pub fn StreakListItem(props: StreakListItemProps) -> Element {
    rsx! {
        li {
            article {
                class: "min-w-52 bg-purple-100 dark:bg-purple-200 dark:bg-gray-300 rounded p-2 transition shadow-sm hover:shadow-lg drop-shadow-sm hover:drop-shadow-lg dark:selection:text-teal-100 dark:selection:bg-purple-800",
                div {
                    class: "flex flex-wrap flex-wrap-reverse gap-2 border-b-1 border-purple-500",
                    p {
                        class: "pt-1",
                        "{props.streak.emoji_status()}"
                    }
                    h1 {
                        class: "text-2xl",
                        "{props.streak.task}"
                    }
                }
                p {
                    class: "text-center text-sm uppercase",
                    match props.streak.frequency {
                        Frequency::Daily => {
                            rsx! { "daily" }
                        }
                        Frequency::Weekly => {
                            rsx! { "weekly" }
                        }
                    }
                }

                dl {
                    class: "grid grid-cols-2 gap-2 py-2",
                    dt {
                        class: "text-sm uppercase text-right",
                        "Last Check-in"
                    }
                    dd {
                        class: "text-sm",
                        {
                            let date_string = props.streak.last_checkin.clone();
                            if let Some(date_string) = date_string {
                                let date_string = date_string.format("%Y-%m-%d");
                                rsx!{ "{date_string}" }
                            } else {
                                rsx!{ "Never" }
                            }
                        }
                    }
                    dt {
                        class: "text-sm uppercase text-right",
                        "Total Check-ins"
                    }
                    dd {
                        class: "text-sm",
                        "{props.streak.total_checkins}"
                    }
                    dt {
                        class: "text-sm uppercase text-right",
                        "Current Streak"
                    }
                    dd {
                        class: "text-sm",
                        "{props.streak.current_streak}"
                    }
                    dt {
                        class: "text-sm uppercase text-right",
                        "Longest Streak"
                    }
                    dd {
                        class: "text-sm",
                        "{props.streak.longest_streak}"
                    }
                }
                div {
                    class: "pt-3 grid grid-cols-2 divide-x divide-purple-500",
                    CheckInButton { streak: props.streak.clone() }
                    DeleteButton { streak: props.streak.clone() }
                }
            }
        }
    }
}

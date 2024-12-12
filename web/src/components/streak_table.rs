use crate::components::{CheckInButton, DeleteButton};
use crate::{get_saved_state, use_persistent, AppState, UsePersistent};
use dioxus::prelude::*;
use streak::sorting::{SortByDirection, SortByField};
use streak::{filter_by_status, sort_streaks, Status, Streak};

#[component]
pub fn TableHeader(sort_by_field: SortByField, text: String) -> Element {
    let mut storage = use_persistent("skidmarks", || AppState::default());
    let mut streak_signal: Signal<Vec<Streak>> = use_context();
    let mut direction = storage.get().sort_by_direction.clone();

    rsx! {
        th {
            class: "relative cursor-pointer select-none text-purple-900 dark:text-teal-200",
            scope: "col",
            onclick: move |_| {
                direction = match storage.get().sort_by_direction {
                    SortByDirection::Ascending => SortByDirection::Descending,
                    SortByDirection::Descending => SortByDirection::Ascending,
                };

                let mut state = storage.get();
                state.sort_by_field = sort_by_field.clone();
                state.sort_by_direction = direction.clone();
                storage.set(state);

                streak_signal.set(storage.get().get_sorted_streaks());
            },
            span {
                class: "align-middle pr-5",
                title: "Click to sort by this column",
                "{text}"
            }
            span {
                class: "absolute inset-y-0 right-0 top-0.5",
                match storage.get().sort_by_direction {
                    SortByDirection::Ascending => rsx! {
                        span {
                            class: "material-symbols-rounded",
                            title: "Click to sort descending",
                            "arrow_upward_alt"
                        }
                    },
                    SortByDirection::Descending => rsx! {
                        span {
                            class: "material-symbols-rounded",
                            title: "Click to sort ascending",
                            "arrow_downward_alt"
                        }
                    },
                }
            }
        }
    }
}

#[component]
pub fn StreakTable() -> Element {
    rsx! {
        div {
            class: "overflow-hidden border-1 border-pink-100 rounded-lg dark:border-teal-700",
            table {
                class: "table table-auto min-w-full bg-pink-200 dark:bg-transparent border-spacing-0.5 border border-separate",
                colgroup {
                    col { span: 3, class: "bg-pink-50 dark:bg-teal-800/25" }
                    col { span: 2, class: "bg-pink-100 dark:bg-teal-900/25" }
                    col { span: 2, class: "bg-pink-50 dark:bg-teal-950/25" }
                    col { span: 1, class: "bg-pink-100 dark:bg-teal-900/25" }
                }
                thead {
                    class: "select-none text-purple-900 dark:text-teal-800",
                    tr {
                        td { colspan: 3 }
                        th { class: "text-purple-900 dark:text-teal-200", colspan: 2, scope: "col", "Check-ins" }
                        th { class: "text-purple-900 dark:text-teal-200", colspan: 2, scope: "col", "Streaks" }
                        td { colspan: "*" }
                    }
                    tr {
                        TableHeader { sort_by_field: SortByField::Task, text: "Task" }
                        TableHeader { sort_by_field: SortByField::Status, text: "Status" }
                        TableHeader { sort_by_field: SortByField::Frequency, text: "Frequency" }
                        TableHeader { sort_by_field: SortByField::LastCheckIn, text: "Last" }
                        TableHeader { sort_by_field: SortByField::TotalCheckins, text: "Total" }
                        TableHeader { sort_by_field: SortByField::CurrentStreak, text: "Current" }
                        TableHeader { sort_by_field: SortByField::LongestStreak, text: "Longest" }
                        th { class: "cursor-pointer select-none text-purple-900 dark:text-teal-100", "Tools" } // dark:even:bg-purple-800 even:bg-purple-100 odd:bg-pink-50 dark:odd:bg-purple-600
                    }
                }
                StreakTableBody {}
            }
        }
    }
}

#[component]
pub fn StreakTableBody() -> Element {
    let mut storage = use_persistent("skidmarks", || AppState::default());
    let streak_signal: Signal<Vec<Streak>> = use_context();
    let streaks = storage.get().get_sorted_streaks();

    rsx! {
        tbody {
            for streak in streak_signal.read().clone() {
                StreakTableRow { streak: streak.clone() }
            }
        }
    }
}

#[component]
pub fn StreakTableRow(mut streak: Streak) -> Element {
    let mut streak_signal = Signal::new(streak.clone());

    let last_checkin = match streak_signal.read().last_checkin {
        Some(checkin) => checkin.to_string(),
        None => "Never".to_string(),
    };

    let streak = streak_signal.read().clone();
    let status = match streak.status() {
        Status::Missed => {
            rsx! {
                span {
                    class: "material-symbols-rounded",
                    title: "Missed, time to restart",
                    "cached"
                }
            }
        }
        Status::Done => {
            rsx! {
                span {
                    class: "material-symbols-rounded",
                    title: "Done",
                    "verified"
                }
            }
        }
        Status::Waiting => {
            rsx! {
                span {
                    class: "material-symbols-rounded",
                    title: "Waiting",
                    "pending"
                }
            }
        }
    };

    rsx! {
        tr {
            class: "text-center text-purple-900 dark:text-teal-100",
            td {
                class: "text-left pl-2",
                "{streak.task}"
            }
            td { class: "select-none", {status} }
            td { "{streak.frequency}" }
            td { {last_checkin} }
            td { "{streak.current_streak}" }
            td { "{streak.longest_streak}" }
            td { "{streak.total_checkins}" }
            td {
                class: "flex flex-row flex-nowrap",
                CheckInButton { streak: streak.clone() }
                // DeleteButton { streak: streak.clone() }
            }
        }
    }
}

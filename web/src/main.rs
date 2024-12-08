use std::cell::Ref;
use dioxus::dioxus_core::internal::generational_box::GenerationalRef;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use serde::{Deserialize, Serialize};

use streak::{Frequency, Streak, Status, sort_streaks};
use uuid::Uuid;
use streak::sorting::{SortByDirection, SortByField};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppState {
    streaks: Vec<Streak>,
    sort_by_field: SortByField,
    sort_by_direction: SortByDirection,
}

impl AppState {
    fn new() -> Self {
        let streak_request = use_resource(move || async move { streak_server().await });
        match streak_request() {
            Some(Ok(streak_response)) => Self {
                streaks: streak_response,
                sort_by_field: SortByField::LastCheckIn,
                sort_by_direction: SortByDirection::Ascending
            },
            _ => Self {
                streaks: vec![],
                sort_by_field: SortByField::LastCheckIn,
                sort_by_direction: SortByDirection::Ascending
            },
        }
    }

    fn sort(self) -> Vec<Streak> {
        let mut state = use_context::<Signal<AppState>>();

        let sorted_streaks = sort_streaks(
            self.streaks,
            self.sort_by_field.clone(),
            self.sort_by_direction.clone()
        );

        sorted_streaks
    }

    fn remove_streak(&self, streak_id: Uuid) {
        let _ = use_resource(move || async move { delete_streak(streak_id).await });
        let mut state = use_context::<Signal<AppState>>();
        let streaks = state.read().streaks.clone();
            use_effect(move || {
                let mut streaks = streaks.clone();
                streaks.retain(|s| s.id != streak_id);
                state.write().streaks = streaks
            });
            dioxus_logger::tracing::info!("{:?}", state.read());
    }

    fn checkin_streak(&self, streak_id: Uuid) {
        let updated_streak = use_resource(move || async move { check_in_streak(streak_id).await });
        let mut state = use_context::<Signal<AppState>>();
        let streaks = state.read().streaks.clone();
        use_effect(move || {
            let mut streaks = streaks.clone();
            let index = streaks.iter().position(|s| s.id == streak_id);
            if let Some(streak) = streaks.get_mut(index.unwrap()) {
                streak.checkin();

                if let Some(index) = index {
                    streaks[index] = streak.clone();
                }
            }
            state.write().streaks = streaks
        });
        dioxus_logger::tracing::info!("{:?}", state.read());
    }
}

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

#[component]
fn App() -> Element {
    // Build cool things ✌️
    let streak_request = use_resource(move || async move { streak_server().await });
    let signal = Signal::new(AppState::new());
    let mut state: Signal<AppState> = use_context_provider(|| signal);

    use_effect(move || match streak_request() {
        Some(Ok(response)) => state.write().streaks.extend(response),
        Some(Err(err)) => (),
        None => (),
    });

    rsx! {
        document::Link { href: "https://fonts.googleapis.com", rel: "preconnect" }
        document::Link { href: "https://fonts.gstatic.com", rel: "preconnect", crossorigin: "true"}
        document::Stylesheet { href: "https://fonts.googleapis.com/css2?family=Material+Symbols+Rounded:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200&icon_names=arrow_downward_alt,arrow_upward_alt,check_circle,delete,pending,verfied" }
        document::Stylesheet { href: asset!("/assets/main.css") }
        div {
            class: "containter mx-auto sm:w-full lg:w-3/4",
            h1 {
                class: "text-3xl font-bold",
                "Skidmarks"
            }
            StreakForm {}
            StreakTable {}
        }

    }
}

#[component]
fn StreakForm() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    let mut task_signal = use_signal(|| "".to_string());
    let mut frequency_signal = use_signal(|| Frequency::Daily);

    rsx! {
        form {
            onsubmit: move |e| {
                let mut new_streak: Streak = Streak::default();
                let task = task_signal.read().clone();

                match frequency_signal.read().clone() {
                    Frequency::Daily => new_streak = Streak::new_daily(task.clone()),
                    Frequency::Weekly => new_streak = Streak::new_weekly(task.clone())
                }
                state.write().streaks.push(new_streak.clone());

                let save_streak = use_resource(move || {
                    let url = "http://minty:3000/streak";
                    let new_streak = new_streak.clone();

                    async move {
                        let url = url.clone();
                        let client = reqwest::Client::new();
                        let response = client
                            .post(url.clone())
                            .json(&new_streak)
                            .send()
                            .await
                            .unwrap();
                        response.json::<Streak>().await.unwrap()
                    }
                });

                task_signal.set("".to_string());
                frequency_signal.set(Frequency::Daily);
            },
            label {
                "Task",
                input {
                    oninput: move |e| {
                        task_signal.set(e.value().clone());
                    },
                    value: task_signal.read().clone(),
                    placeholder: "task"
                }
            }
            label {
                "Frequency",
                select {
                    onchange: move |event| {
                        frequency_signal.set(Frequency::from_str(event.value().as_str()));
                    },
                    option {
                        selected: frequency_signal.read().clone() == Frequency::Daily,
                        value: "daily",
                        "Daily"
                    }
                    option {
                        selected: frequency_signal.read().clone() == Frequency::Weekly,
                        value: "weekly",
                        "Weekly"
                    }
                }
            }
            button {
                type: "submit",
                "Add"
            }
        }
    }
}

#[component]
fn TableHeader(sort_by_field: SortByField, text: String) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut direction = SortByDirection::Ascending;

    rsx! {
        th {
            class: "min-w-fit text-pretty",
            onclick: move |_| {
                if state.read().sort_by_field == sort_by_field {}
                    direction = match state.read().sort_by_direction {
                        SortByDirection::Ascending => SortByDirection::Descending,
                        SortByDirection::Descending => SortByDirection::Ascending,
                    };
                }
                state.write().sort_by_field = sort_by_field.clone();
                state.write().sort_by_direction = direction.clone();
            },
            "{text}",
            if state.read().sort_by_field == sort_by_field {
                match state.read().sort_by_direction {
                    SortByDirection::Ascending => rsx! {
                        span {
                            class: "material-symbols-rounded",
                            "arrow_upward_alt"
                        }
                    },
                    SortByDirection::Descending => rsx! {
                        span {
                            class: "material-symbols-rounded",
                            "arrow_downward_alt"
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn StreakTable() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    rsx! {
        table {
            class: "w-full",
            thead {
                tr {
                    td { colspan: 3 }
                    th { colspan: 2, "Check-ins" }
                    th { colspan: 2, "Streaks" }
                }
                TableHeader { sort_by_field: SortByField::Task, text: "Task" }
                TableHeader { sort_by_field: SortByField::Status, text: "Status" }
                TableHeader { sort_by_field: SortByField::Frequency, text: "Frequency" }
                TableHeader { sort_by_field: SortByField::LastCheckIn, text: "Last" }
                TableHeader { sort_by_field: SortByField::TotalCheckins, text: "Total" }
                TableHeader { sort_by_field: SortByField::CurrentStreak, text: "Current" }
                TableHeader { sort_by_field: SortByField::LongestStreak, text: "Longest" }
                th { "Tools" }
            }
            tbody {
                for streak in state.read().clone().sort().iter() {
                    StreakTableRow { streak: streak.clone() }
                }
            }
        }
    }
}

#[component]
fn StreakTableRow(mut streak: Streak) -> Element {
    let mut state = use_context::<Signal<AppState>>();
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
                    "dangerous"
                }
            }
        },
        Status::Done => {
            rsx! {
                span {
                    class: "material-symbols-rounded",
                    "verified"
                }
            }
        },
        Status::Waiting => {
            rsx! {
                span {
                    class: "material-symbols-rounded",
                    "pending"
                }
            }
        }
    };

    rsx! {
        tr {
            td { "{streak.task}" }
            td { {status} }
            td { "{streak.frequency}" }
            td { {last_checkin} }
            td { "{streak.current_streak}" }
            td { "{streak.longest_streak}" }
            td { "{streak.total_checkins}" }
            td {
                CheckInButton { streak: streak.clone() }
                DeleteButton { streak: streak.clone() }
            }
        }
    }
}

#[component]
fn CheckInButton(streak: Streak) -> Element {
    let state = use_context::<Signal<AppState>>();
    rsx! {
        button {
            onclick: move |e| {state.read().checkin_streak(streak.id)},
            span {
                class: "material-symbols-rounded",
                "check_circle"
            }
        }
    }
}

#[component]
fn DeleteButton(streak: Streak) -> Element {
    let state = use_context::<Signal<AppState>>();
    rsx! {
        button {
            onclick: move |e| {state.read().remove_streak(streak.id)},
            span {
                class: "material-symbols-rounded",
                "delete"
            }
        }
    }
}

#[server]
async fn streak_server() -> Result<Vec<Streak>, ServerFnError> {
    let response = reqwest::get("http://minty:3000/streak").await;
    let streaks = response?.json::<Vec<Streak>>().await.unwrap();
    dioxus_logger::tracing::info!("streak_server done");
    Ok(streaks)
}

#[server]
async fn delete_streak(streak_id: Uuid) -> Result<Uuid, ServerFnError> {
    let client = reqwest::Client::new();
    let response = client.delete(format!("http://minty:3000/streak/{}", streak_id)).send().await;
    Ok(streak_id)
}

#[server]
async fn check_in_streak(streak_id: Uuid) -> Result<Streak, ServerFnError> {
    let client = reqwest::Client::new();
    let response = client.put(format!("http://minty:3000/streak/{}/check-in", streak_id)).send().await?;
    let streak = response.json::<Streak>().await?;
    Ok(streak)
}

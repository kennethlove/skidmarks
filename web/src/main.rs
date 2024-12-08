use std::cell::Ref;
use dioxus::dioxus_core::internal::generational_box::GenerationalRef;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use serde::{Deserialize, Serialize};

use streak::{Frequency, Streak};
use uuid::Uuid;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    StreakTable {},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppState {
    streaks: Vec<Streak>,
}

impl AppState {
    fn new() -> Self {
        let streak_request = use_resource(move || async move { streak_server().await });
        match streak_request() {
            Some(Ok(streak_response)) => Self {
                streaks: streak_response,
            },
            _ => Self {
                streaks: Vec::new(),
            },
        }
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
        document::Stylesheet { href: "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20,300,0,0" }
        document::Stylesheet { href: asset!("/assets/main.css") }

        h1 {
            "Streaks"
        }

        StreakForm {}

        Router::<Route> {}
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            Link {
                to: Route::StreakTable {},
                "Home"
            }
        }

        Outlet::<Route> {}
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
fn StreakTable() -> Element {
    let state = use_context::<Signal<AppState>>();

    rsx! {
        table {
            thead {
                th { "Task" }
                th { "Status" }
                th { "Frequency" }
                th { "Last Check-in" }
                th { "Current Streak" }
                th { "Longest Streak" }
                th { "Total Check-ins" }
                th { "Tools" }
            }
            tbody {
                for streak in state.read().streaks.iter() {
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

    rsx! {
        tr {
            td { "{streak.task}" }
            td { "{streak.emoji_status()}" }
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
                class: "material-symbols-outlined",
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
                class: "material-symbols-outlined",
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

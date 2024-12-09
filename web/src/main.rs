use dioxus::dioxus_core::internal::generational_box::GenerationalRef;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use serde::{Deserialize, Serialize};
use std::cell::Ref;

use streak::filtering::FilterByStatus;
use streak::sorting::{SortByDirection, SortByField};
use streak::{filter_by_status, sort_streaks, Frequency, Status, Streak};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppState {
    streaks: Vec<Streak>,
    sort_by_field: SortByField,
    sort_by_direction: SortByDirection,
    filter_by: FilterByStatus,
}

impl AppState {
    fn new() -> Self {
        let streak_request = use_resource(move || {
            let filter = FilterByStatus::All;
            async move { streak_server(filter).await }
        });
        match streak_request() {
            Some(Ok(streak_response)) => Self {
                streaks: streak_response,
                sort_by_field: SortByField::LastCheckIn,
                sort_by_direction: SortByDirection::Ascending,
                filter_by: FilterByStatus::All,
            },
            _ => Self {
                streaks: vec![],
                sort_by_field: SortByField::LastCheckIn,
                sort_by_direction: SortByDirection::Ascending,
                filter_by: FilterByStatus::All,
            },
        }
    }

    fn sort(self) -> Vec<Streak> {
        let sorted_streaks = sort_streaks(
            self.streaks,
            self.sort_by_field.clone(),
            self.sort_by_direction.clone(),
        );

        sorted_streaks
    }

    fn filter(self) -> Vec<Streak> {
        let filtered_streaks = filter_by_status(self.streaks, self.filter_by);

        filtered_streaks
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
    }
}

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

#[component]
fn App() -> Element {
    // Build cool things ✌️
    let signal = Signal::new(AppState::new());
    let mut state: Signal<AppState> = use_context_provider(|| signal);
    let filter = state.read().filter_by.clone();

    let streak_request = use_resource(move || {
        let filter = filter.clone();
        async move { streak_server(filter).await }
    });

    use_effect(move || match streak_request() {
        Some(Ok(response)) => state.write().streaks.extend(response),
        Some(Err(err)) => (),
        None => (),
    });

    rsx! {
        document::Link { href: "https://fonts.googleapis.com", rel: "preconnect" }
        document::Link { href: "https://fonts.gstatic.com", rel: "preconnect", crossorigin: "true" }
        document::Stylesheet { href: "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20,400,0,0&icon_names=arrow_downward_alt,arrow_upward_alt,cached,check_circle,delete,pending,verified", rel: "stylesheet" }
        document::Stylesheet { href: asset!("/assets/main.css") }
        div {
            class: "container mx-auto sm:w-full lg:w-10/12",
            h1 {
                class: "text-3xl font-bold pt-4",
                "Skidmarks"
            }
            div {
                class: "flex flex-row flex-nowrap gap-3 py-4",
                StreakForm {}
                StreakFilters {}
            }
            StreakTable {}
        }

    }
}

#[component]
fn StreakFilters() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let filter_by = state.read().filter_by.clone();

    rsx! {
        div {
            class: "border-l border-gray-300 pl-3",
            fieldset {
                class: "flex flex-nowrap gap-0 select-none",
                legend {
                    class: "sr-only",
                    "Frequency"
                }
                div {
                    label {
                        class: "flex cursor-pointer items-center justify-center rounded-l-md border border-gray-100 bg-white px-3 py-3 text-gray-900 hover:border-gray-200 has-[:checked]:border-blue-500 has-[:checked]:bg-blue-500 has-[:checked]:text-white",
                        input {
                            class: "sr-only",
                            name: "filter",
                            r#type: "radio",
                            checked: filter_by == FilterByStatus::All,
                            onclick: move |e| { state.write().filter_by = FilterByStatus::All },
                        }
                        p {
                            class: "text-sm font-medium",
                            "All",
                        }
                    }
                }
                div {
                    label {
                        class: "flex cursor-pointer items-center justify-center border border-gray-100 bg-white px-3 py-3 text-gray-900 hover:border-gray-200 has-[:checked]:border-blue-500 has-[:checked]:bg-blue-500 has-[:checked]:text-white",
                        input {
                            class: "sr-only",
                            name: "filter",
                            r#type: "radio",
                            checked: filter_by == FilterByStatus::Waiting,
                            onclick: move |e| { state.write().filter_by = FilterByStatus::Waiting },
                        }
                        p {
                            class: "text-sm font-medium",
                            "Waiting"
                        }
                    }
                }
                div {
                    label {
                        class: "flex cursor-pointer items-center justify-center border border-gray-100 bg-white px-3 py-3 text-gray-900 hover:border-gray-200 has-[:checked]:border-blue-500 has-[:checked]:bg-blue-500 has-[:checked]:text-white",
                        input {
                            class: "sr-only",
                            name: "filter",
                            r#type: "radio",
                            checked: filter_by == FilterByStatus::Missed,
                            onclick: move |e| { state.write().filter_by = FilterByStatus::Missed },
                        }
                        p {
                            class: "text-sm font-medium",
                            "Missed"
                        }
                    }
                }
                div {
                    label {
                        class: "flex cursor-pointer items-center justify-center rounded-r-md border border-gray-100 bg-white px-3 py-3 text-gray-900 hover:border-gray-200 has-[:checked]:border-blue-500 has-[:checked]:bg-blue-500 has-[:checked]:text-white",
                        input {
                            class: "sr-only",
                            name: "filter",
                            r#type: "radio",
                            checked: filter_by == FilterByStatus::Done,
                            onclick: move |e| { state.write().filter_by = FilterByStatus::Done },
                        }
                        p {
                            class: "text-sm font-medium",
                            "Done"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StreakForm() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    let mut task_signal = use_signal(|| "".to_string());
    let mut frequency_signal = use_signal(|| Frequency::Daily);

    rsx! {
        div {
            class: "w-2/3",
            form {
                class: "flex flex-row flex-nowrap gap-3 content-between justify-stretch",
                onsubmit: move |e| {
                    let mut new_streak: Streak = Streak::default();
                    let task = task_signal.read().clone();

                    dioxus_logger::tracing::info!("task started: {}", &frequency_signal.read());

                    match frequency_signal.read().clone() {
                        Frequency::Daily => new_streak = Streak::new_daily(task.clone()),
                        Frequency::Weekly => new_streak = Streak::new_weekly(task.clone())
                    }
                    state.write().streaks.push(new_streak.clone());

                    let save_streak = use_resource(move || {
                        let url = "http://127.0.0.1:3000/streak";
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
                    class: "grow relative block rounded-md border border-gray-200 shadow-sm focus-within:border-blue-600 focus-within:ring-1 focus-within:ring-blue-600",
                    input {
                        class: "peer border-none bg-transparent placeholder-transparent focus:border-transparent focus:outline-none focus:ring-0 px-3 py-2",
                        oninput: move |e| {
                            task_signal.set(e.value().clone());
                        },
                        value: task_signal.read().clone(),
                        placeholder: "Task"
                    }
                    span {
                        class: "pointer-events-none absolute start-2.5 top-0 -translate-y-1/2 bg-white p-0.5 text-xs text-gray-700 transition-all peer-placeholder-shown:top-1/2 peer-placeholder-shown:text-sm peer-focus:top-0 peer-focus:text-xs",
                        "Task",
                    }
                }
                fieldset {
                    class: "flex flex-wrap gap-3 select-none",
                    legend {
                        class: "sr-only",
                        "Frequency"
                    }
                    div {
                        label {
                            class: "flex cursor-pointer items-center justify-center rounded-md border border-gray-100 bg-white px-3 py-3 text-gray-900 hover:border-gray-200 has-[:checked]:border-blue-500 has-[:checked]:bg-blue-500 has-[:checked]:text-white",
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
                            class: "flex cursor-pointer items-center justify-center rounded-md border border-gray-100 bg-white px-3 py-3 text-gray-900 hover:border-gray-200 has-[:checked]:border-blue-500 has-[:checked]:bg-blue-500 has-[:checked]:text-white",
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
                    class: "inline-block rounded border border-indigo-600 bg-indigo-600 px-6 text-sm font-medium text-white hover:bg-transparent hover:text-indigo-600 focus:outline-none focus:ring active:text-indigo-500 cursor-pointer",
                    type: "submit",
                    "Add"
                }
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
            class: "min-w-fit text-pretty cursor-pointer select-none border-x border-gray-200",
            onclick: move |_| {
                if state.read().sort_by_field == sort_by_field {
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
                            class: "material-symbols-rounded align-middle",
                            "arrow_upward_alt"
                        }
                    },
                    SortByDirection::Descending => rsx! {
                        span {
                            class: "material-symbols-rounded align-middle",
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
        div {
            class: "overflow-hidden border-1 border-gray-200 rounded-lg",
            table {
                class: "min-w-full divide-y-2 divide-gray-200 bg-white text-sm",
                thead {
                    class: "bg-gray-300 select-none",
                    tr {
                        td { class: "border-x border-gray-200", colspan: 3 }
                        th { class: "border-x border-b border-gray-200", colspan: 2, "Check-ins" }
                        th { class: "border-x border-b border-gray-200", colspan: 2, "Streaks" }
                        td { class: "border-x border-gray-200", colspan: "*" }
                    }
                    TableHeader { sort_by_field: SortByField::Task, text: "Task" }
                    TableHeader { sort_by_field: SortByField::Status, text: "Status" }
                    TableHeader { sort_by_field: SortByField::Frequency, text: "Frequency" }
                    TableHeader { sort_by_field: SortByField::LastCheckIn, text: "Last" }
                    TableHeader { sort_by_field: SortByField::TotalCheckins, text: "Total" }
                    TableHeader { sort_by_field: SortByField::CurrentStreak, text: "Current" }
                    TableHeader { sort_by_field: SortByField::LongestStreak, text: "Longest" }
                    th { class: "border-x border-gray-200", "Tools" }
                }
                StreakTableBody {}
            }
        }
    }
}

#[component]
fn StreakTableBody() -> Element {
    let state = use_context::<Signal<AppState>>();
    let filter_field = state.read().filter_by.clone();
    let sort_field = state.read().sort_by_field.clone();
    let sort_direction = state.read().sort_by_direction.clone();
    let mut streaks = state.read().clone().streaks;
    streaks = sort_streaks(streaks.clone(), sort_field, sort_direction);
    streaks = filter_by_status(streaks.clone(), filter_field);
    rsx! {
        tbody {
            for streak in streaks {
                StreakTableRow { streak: streak.clone() }
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
                    class: "material-symbols-outlined",
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
            class: "text-center odd:bg-gray-50",
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
            class: "cursor-pointer select-none",
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
            class: "cursor-pointer select-none",
            onclick: move |e| {state.read().remove_streak(streak.id)},
            span {
                class: "material-symbols-rounded",
                "delete"
            }
        }
    }
}

#[server]
async fn streak_server(filter: FilterByStatus) -> Result<Vec<Streak>, ServerFnError> {
    let response = reqwest::get(format!("http://127.0.0.1:3000/streak?status={}", filter)).await;
    let streaks = response?.json::<Vec<Streak>>().await.unwrap();
    dioxus_logger::tracing::info!("streak_server done");
    Ok(streaks)
}

#[server]
async fn delete_streak(streak_id: Uuid) -> Result<Uuid, ServerFnError> {
    let client = reqwest::Client::new();
    let response = client
        .delete(format!("http://127.0.0.1:3000/streak/{}", streak_id))
        .send()
        .await;
    Ok(streak_id)
}

#[server]
async fn check_in_streak(streak_id: Uuid) -> Result<Streak, ServerFnError> {
    let client = reqwest::Client::new();
    let response = client
        .put(format!(
            "http://127.0.0.1:3000/streak/{}/check-in",
            streak_id
        ))
        .send()
        .await?;
    let streak = response.json::<Streak>().await?;
    Ok(streak)
}

use dioxus::dioxus_core::internal::generational_box::GenerationalRef;
use dioxus::document::eval;
use dioxus::prelude::server_fn::codec::Json;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use serde::{Deserialize, Serialize};
use std::cell::Ref;
use std::convert::From;
use std::future::Future;
use std::iter::Extend;
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
    dark_mode: bool,
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
                dark_mode: false,
            },
            _ => Self {
                streaks: vec![],
                sort_by_field: SortByField::LastCheckIn,
                sort_by_direction: SortByDirection::Ascending,
                filter_by: FilterByStatus::All,
                dark_mode: false,
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
    let state = Signal::new(AppState::new());
    let mut state: Signal<AppState> = use_context_provider(|| state);
    let filter = state.read().filter_by.clone();

    let modal_state: Signal<Option<Streak>> = Signal::new(None);
    let modal_state = use_context_provider(|| modal_state);

    let streak_request = use_resource(move || {
        let filter = filter.clone();
        async move { streak_server(filter).await }
    });

    use_effect(move || match streak_request() {
        Some(Ok(response)) => state.write().streaks.extend(response),
        Some(Err(err)) => (),
        None => (),
    });

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
                        class: "text-3xl font-bold text-purple-900 dark:text-teal-700",
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
                            class: "material-symbols-rounded dark:text-teal-700 text-purple-500 hover:text-pink-50 dark:hover:text-teal-600",
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

#[derive(Debug, Clone, Props, PartialEq)]
struct FilterButtonProps {
    button_text: String,
    button_status: FilterByStatus,
}

#[component]
fn FilterButton(props: FilterButtonProps) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let filter_by = state.read().filter_by.clone();

    rsx! {
        div {
            label {
                class: "transition cursor-pointer inline-block rounded-md px-4 py-2 text-sm text-purple-500 dark:text-purple-900 hover:text-purple-700 focus:relative has-[:checked]:border-purple-500 has-[:checked]:bg-purple-500 has-[:checked]:text-pink-100 dark:has-[:checked]:border-purple-900 dark:has-[:checked]:bg-purple-900",
                input {
                    class: "sr-only",
                    name: "filter",
                    r#type: "radio",
                    checked: filter_by == props.button_status.clone(),
                    onclick: move |e| { state.write().filter_by = props.button_status.clone(); },
                }
                p {
                    class: "text-sm font-medium",
                    "{props.button_text}",
                }
            }
        }

    }
}

#[component]
fn StreakFilters() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let filter_by = state.read().filter_by.clone();

    rsx! {
        div {
            class: "lg:pl-4 self-center",
            fieldset {
                class: "inline-flex rounded-lg border border-pink-100 bg-purple-50/75 p-1 dark:bg-purple-200/75 dark:border-purple-100 select-none",
                legend {
                    class: "sr-only",
                    "Frequency"
                }
                FilterButton { button_text: "All".to_string(), button_status: FilterByStatus::All }
                FilterButton { button_text: "Waiting".to_string(), button_status: FilterByStatus::Waiting }
                FilterButton { button_text: "Missed".to_string(), button_status: FilterByStatus::Missed }
                FilterButton { button_text: "Done".to_string(), button_status: FilterByStatus::Done }
            }
        }
    }
}

#[component]
fn StreakForm() -> Element {
    let mut state = use_context::<Signal<AppState>>();

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

                    match frequency_signal.read().clone() {
                        Frequency::Daily => new_streak = Streak::new_daily(task.clone()),
                        Frequency::Weekly => new_streak = Streak::new_weekly(task.clone())
                    }

                    let save_streak = create_streak(new_streak.clone());
                    async move {
                        if let Ok(streak) = save_streak.await {
                            state.write().streaks.push(streak);
                            task_signal.set(String::from(""));
                            frequency_signal.set(Frequency::Daily);
                        }
                    }
                },
                label {
                    class: "grow relative block overflow-hidden rounded-md border border-pink-100 dark:border-purple-100 focus-within:border-purple-500 focus-within:ring-1 focus-within:ring-purple-500 dark:bg-purple-200/75 bg-purple-50/75 focus-within:bg-pink-50 px-3 pt-1.5 transition dark:focus-within:border-purple-100 dark:focus-within:ring-purple-100",
                    input {
                        class: "peer h-8 border-none bg-transparent focus:border-transparent focus:outline-none focus:ring-0 p-0 w-full",
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
            class: "relative cursor-pointer select-none text-purple-900 dark:text-teal-200",
            scope: "col",
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
            span {
                class: "align-middle pr-5",
                title: "Click to sort by this column",
                "{text}"
            }
            span {
                class: "absolute inset-y-0 right-0 top-0.5",
                if state.read().sort_by_field == sort_by_field {
                    match state.read().sort_by_direction {
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
}

#[component]
fn StreakTable() -> Element {
    let mut state = use_context::<Signal<AppState>>();

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
            class: "cursor-pointer select-none hover:text-green-500 focus:text-green-500 focus:outline-none",
            onclick: move |e| {state.read().checkin_streak(streak.id)},
            span {
                class: "material-symbols-rounded",
                title: "Check-in",
                "check_circle"
            }
        }
    }
}

#[component]
fn DeleteButton(streak: Streak) -> Element {
    let state = use_context::<Signal<AppState>>();
    let mut modal_signal: Signal<Option<Streak>> = use_context();
    rsx! {
        button {
            class: "cursor-pointer select-none hover:text-red-500 focus:text-red-500 focus:outline-none",
            onclick: move |_| {
                modal_signal.set(Some(streak.clone()));
            },
            span {
                class: "material-symbols-rounded",
                title: "Delete",
                "delete"
            }
        }
    }
}

#[component]
fn DeleteModal() -> Element {
    let state = use_context::<Signal<AppState>>();
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

#[server]
async fn streak_server(filter: FilterByStatus) -> Result<Vec<Streak>, ServerFnError> {
    let response = reqwest::get(format!("http://127.0.0.1:3000/streak?status={}", filter)).await;
    let streaks = response?.json::<Vec<Streak>>().await.unwrap();
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

#[server]
async fn create_streak(streak: Streak) -> Result<Streak, ServerFnError> {
    if streak.task.is_empty() {
        return Err(ServerFnError::MissingArg("task".to_string()));
    }

    let client = reqwest::Client::new();
    let response = client
        .post("http://127.0.0.1:3000/streak")
        .json(&streak)
        .send()
        .await?;
    let streak = response.json::<Streak>().await?;
    Ok(streak)
}

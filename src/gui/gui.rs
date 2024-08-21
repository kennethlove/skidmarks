use crate::cli::get_database_url;
use crate::color::GuiStyles;
use crate::filtering::FilterByStatus;
use crate::sorting::{SortByDirection, SortByField};
use crate::streak::Status;
use crate::{db::Database, streak::Frequency, streak::Streak};
use dioxus::desktop::{use_global_shortcut, Config, WindowBuilder};
use dioxus::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

pub fn main() {
    LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(WindowBuilder::new().with_resizable(true)))
        .launch(app)
}

fn app() -> Element {
    let mut streaks = use_signal(Streaks::new);
    let gui_styles = GuiStyles::new();

    let show_popup = use_signal(|| None);
    _ = use_global_shortcut("CmdOrCtrl+Q", move || {
        std::process::exit(0);
    });
    _ = use_global_shortcut("CmdOrCtrl+R", move || {
        streaks.write().refresh();
    });

    rsx! {
        head::Link {
            rel: "stylesheet",
            href: "https://cdn.jsdelivr.net/npm/bulma@1.0.2/css/bulma.min.css"
        }
        // head::Link { rel: "stylesheet", href: asset!("./assets/streaks.css") }

        style { r#type: "text/css",
            {format!(r#"
            body {{
                background-color: {0};
                color: {1};
            }}
            "#,
                gui_styles.background,
                gui_styles.foreground
            )}
        }

        div {
            header {
                class: "is-fixed",
                style: "background-color: {gui_styles.header_bg}",
                h1 {
                    style: "color: {gui_styles.header_fg}",
                    class: "is-size-1 has-text-centered has-text-weight-bold",
                    "Skidmarks"
                }
            }
            div { class: "section p-2 mt-2", {streak_search(streaks)} }
            main { class: "section p-2 mt-1 container", {streak_table(streaks, show_popup)} }
            div { class: "section p-2 mt-1", {streak_form(streaks)} }
            p { class: "has-text-centered is-size-7 pb-3", "Copyright © 2024 klove" }
            {popup(show_popup, streaks)}
        }
    }
}

fn streak_search(mut streaks: Signal<Streaks>) -> Element {
    rsx! {
        form { class: "form columns is-1 is-0-mobile",
            div { class: "column is-half",
                input {
                    class: "input",
                    r#type: "search",
                    placeholder: "Search",
                    oninput: move |event| {
                        let search = event.data().value();
                        streaks.write().search(search);
                    }
                }
            }
            div { class: "column",
                div { class: "select mr-2",
                    select {
                        class: "select",
                        name: "status",
                        oninput: move |event| {
                            let filter = FilterByStatus::from_str(&event.data().value());
                            streaks.write().filter_by(filter);
                            streaks.write().load_streaks();
                        },
                        option { "All" }
                        option { "Done" }
                        option { "Waiting" }
                        option { "Missed" }
                    }
                }
                button {
                    class: "button",
                    onclick: move |_| {
                        streaks.write().load_streaks();
                    },
                    "Reset"
                }
            }
        }
    }
}

fn streak_table(mut streaks: Signal<Streaks>, mut show_popup: Signal<Option<Uuid>>) -> Element {
    rsx! {
        table { class: "table is-striped is-hoverable is-narrow is-fullwidth",
            thead {
                tr {
                    th {
                        onclick: move |_| {
                            streaks.write().sort_by(SortByField::Task);
                        },
                        {
                            streaks.read().field_and_emoji(SortByField::Task)
                        }
                    }
                    th {
                        onclick: move |_| {
                            streaks.write().sort_by(SortByField::Frequency);
                        },
                        {
                            streaks.read().field_and_emoji(SortByField::Frequency)
                        }
                    }
                    th {
                        onclick: move |_| {
                            streaks.write().sort_by(SortByField::Status);
                        },
                        {
                            streaks.read().field_and_emoji(SortByField::Status)
                        }
                    }
                    th {
                        onclick: move |_| {
                            streaks.write().sort_by(SortByField::LastCheckIn);
                        },
                        {
                            streaks.read().field_and_emoji(SortByField::LastCheckIn)
                        }
                    }
                    th {
                        onclick: move |_| {
                            streaks.write().sort_by(SortByField::CurrentStreak);
                        },
                        {
                            streaks.read().field_and_emoji(SortByField::CurrentStreak)
                        }
                    }
                    th {
                        onclick: move |_| {
                            streaks.write().sort_by(SortByField::LongestStreak);
                        },
                        {
                            streaks.read().field_and_emoji(SortByField::LongestStreak)
                        }
                    }
                    th {
                        onclick: move |_| {
                            streaks.write().sort_by(SortByField::TotalCheckins);
                        },
                        {
                            streaks.read().field_and_emoji(SortByField::TotalCheckins)
                        }
                    }
                    th { "Actions" }
                }
            }
            tbody {
                for streak in streaks.read().streak_list.iter() {
                    {
                    let id = streak.id.clone();
                    let streak_name = &streak.task;
                    let frequency = &streak.frequency.to_string();
                    let emoji = &streak.emoji_status();
                    let check_in = match &streak.last_checkin {
                        Some(date) => date.to_string(),
                        None => "None".to_string(),
                    };
                    
                    let current_streak = &streak.current_streak.to_string();
                    let longest_streak = &streak.longest_streak.to_string();
                    let total_checkins = &streak.total_checkins.to_string();
                    
                    rsx! {
                        tr { class: "streak", key: "{id}",
                            td { class: "streak-name", "{streak_name}" }
                            td { class: "streak-frequency", "{frequency}" }
                            td { class: "streak-emoji", "{emoji}" }
                            td { class: "streak-check-in", "{check_in}" }
                            td { class: "streak-current-streak", "{current_streak}" }
                            td { class: "streak-longest-streak", "{longest_streak}" }
                            td { class: "streak-total-checkins", "{total_checkins}" }
                            td { class: "streak-actions",
                                button { class: "button is-success", onclick: move |_| {
                                    streaks.write().checkin(&id)
                                    }, "✓"
                                }
                                button { class: "button is-danger", onclick: move |_| {
                                    show_popup.set(Some(id));
                                }, "×"
                                }
                            }
                        }
                    }
                    }
                }
            }
            tfoot {
                tr {
                    td { colspan: 8, "Streaks: {streaks.read().streak_list.len()}" }
                }
            }
        }
    }
}

fn streak_form(mut streaks: Signal<Streaks>) -> Element {
    let mut values = use_signal(HashMap::new);
    let mut submitted_values = use_signal(HashMap::new);

    let mut task_signal = use_signal(String::new);
    let mut freq_signal = use_signal(FormValue::default);
    let freq_value = FormValue {
        0: vec!["Daily".to_string()],
    };

    rsx!(
        if !submitted_values.read().is_empty() {
            h2 { "Submitted!" }
        }

        div {
            form {
                id: "streak-form",
                class: "form columns is-1 is-0-mobile",
                oninput: move |event| {
                    values.set(event.values());
                },
                onsubmit: move |event| {
                    submitted_values.set(event.values());
                    let values = submitted_values.read();
                    let task = values.get("task").expect("Unable to get task value");
                    let default_frequency = FormValue(vec!["Daily".to_string()]);
                    let freq = values.get("frequency").unwrap_or(&default_frequency);
                    match freq.as_value().as_str() {
                        "Daily" => streaks.write().new_streak(&task.as_value(), Frequency::Daily),
                        "Weekly" => streaks.write().new_streak(&task.as_value(), Frequency::Weekly),
                        _ => streaks.write().new_streak(&task.as_value(), Frequency::Daily),
                    };
                    task_signal.set(String::new());
                    freq_signal
                        .set(FormValue {
                            0: vec!["Daily".to_string()],
                        });
                    streaks.write().load_streaks();
                },
                div { class: "column is-half",
                    input {
                        class: "input",
                        r#type: "text",
                        name: "task",
                        placeholder: "Task",
                        value: task_signal.read().clone().into_value(),
                        oninput: move |event| {
                            task_signal.set(event.data().value());
                        }
                    }
                }
                div { class: "column",
                    div { class: "select mr-2",
                        select {
                            class: "select",
                            name: "frequency",
                            oninput: move |_| {
                                freq_signal.set(freq_value.clone());
                            },
                            option { "Daily" }
                            option { "Weekly" }
                        }
                    }
                    button { class: "button", r#type: "submit", "Add" }
                }
            }
        }
    )
}

fn popup(mut is_open: Signal<Option<Uuid>>, mut streaks: Signal<Streaks>) -> Element {
    let mut streak = None;
    let signal_id = is_open.read().clone();
    if let Some(id) = signal_id {
        streak = streaks.read().get_by_ident(id);
        if streak.is_none() {
            is_open.set(None);
        }
        streak = streak.clone();
    }

    rsx! {
        div { class: if is_open.read().is_some() { "modal is-active" } else { "modal" },
            div { class: "modal-background" }
            div { class: "modal-content",
                div { class: "box",
                    h1 { class: "is-size-3", "Delete this streak?" }
                    p { class: "is-size-5 has-text-centered",
                        {streak.as_ref().map_or("", |s| &s.task)}
                    }
                    div { class: "columns",
                        div { class: "column",
                            h3 { "Frequency" }
                            p { {streak.as_ref().map_or("", |s| &s.frequency.as_str())} }
                        }
                        div { class: "column",
                            h3 { "Status" }
                            p { {streak.as_ref().map_or("", |s| &s.emoji_status())} }
                        }
                        div { class: "column",
                            h3 { "Last Checkin" }
                            p {
                                {streak.as_ref().map_or("None".to_string(), |s| s.last_checkin.unwrap().to_string())}
                            }
                        }
                        div { class: "column",
                            h3 { "Current Streak" }
                            p {
                                {streak.as_ref().map_or("".to_string(), |s| s.current_streak.to_string())}
                            }
                        }
                        div { class: "column",
                            h3 { "Longest Streak" }
                            p {
                                {streak.as_ref().map_or("".to_string(), |s| s.longest_streak.to_string())}
                            }
                        }
                        div { class: "column",
                            h3 { "Total Checkins" }
                            p {
                                {streak.as_ref().map_or("".to_string(), |s| s.total_checkins.to_string())}
                            }
                        }
                    }
                    button {
                        class: "button is-danger",
                        onclick: move |_| {
                            streaks.write().delete(&is_open.read().unwrap());
                            streaks.write().load_streaks();
                            is_open.set(None);
                        },
                        "Delete"
                    }
                }
            }
            button {
                onclick: move |_| {
                    is_open.set(None);
                },
                class: "modal-close is-large",
                aria_label: "close"
            }
        }
    }
}

struct Streaks {
    db: Database,
    streak_list: Vec<Streak>,
    sort_by: SortByField,
    sort_dir: SortByDirection,
    filter_by: FilterByStatus,
}

impl Streaks {
    fn new() -> Self {
        let db = Database::new(&get_database_url()).expect("Failed to connect to database");
        let mut streaks = Self {
            db,
            streak_list: vec![],
            sort_by: SortByField::Task,
            sort_dir: SortByDirection::Ascending,
            filter_by: FilterByStatus::All,
        };

        streaks.load_streaks();
        streaks
    }

    fn load_streaks(&mut self) {
        let sort_by = self.sort_by.clone();
        let sort_dir = self.sort_dir.clone();
        let filter_by = self.filter_by.clone();
        let streaks = self.db.get_sorted(sort_by, sort_dir);
        let filtered_streaks = streaks
            .into_iter()
            .filter(|streak| match filter_by {
                FilterByStatus::All => true,
                FilterByStatus::Done => streak.status() == Status::Done,
                FilterByStatus::Missed => streak.status() == Status::Missed,
                FilterByStatus::Waiting => streak.status() == Status::Waiting,
            })
            .collect();
        self.streak_list = filtered_streaks;
    }

    fn refresh(&mut self) {
        let mut streak_signal = use_signal(Streaks::new);
        streak_signal.write().load_streaks();
    }

    fn delete(&mut self, id: &Uuid) {
        match self.db.delete(*id) {
            Ok(_) => {
                let _ = self.db.save();
                self.load_streaks()
            }
            Err(e) => eprintln!("Failed to delete streak: {}", e),
        }
    }

    fn checkin(&mut self, id: &Uuid) {
        match self.db.checkin(*id) {
            Ok(_) => {
                let _ = self.db.save();
                self.load_streaks()
            }
            Err(e) => eprintln!("Failed to checkin: {}", e),
        }
    }

    fn new_streak(&mut self, task: &str, frequency: Frequency) {
        let streak = Streak {
            task: task.to_string(),
            frequency,
            ..Default::default()
        };
        match self.db.add(streak) {
            Ok(_) => {
                let _ = self.db.save();
                self.load_streaks();
            }
            Err(e) => eprintln!("Failed to add streak: {}", e),
        }
    }

    fn sort_by(&mut self, field: SortByField) {
        self.sort_by = field;
        self.sort_dir = match self.sort_dir {
            SortByDirection::Ascending => SortByDirection::Descending,
            SortByDirection::Descending => SortByDirection::Ascending,
        };
        self.load_streaks();
    }

    fn field_and_emoji(&self, field: SortByField) -> String {
        let sorted_field = self.sort_by.clone();
        let field_name = field.to_string()[..1].to_uppercase() + &field.to_string()[1..];
        let field_name = field_name.replace("_", " ");
        if field != sorted_field {
            return format!("{field_name} ");
        }
        let sort_dir = self.sort_dir.clone();
        let emoji = match sort_dir {
            SortByDirection::Ascending => "⬆",
            SortByDirection::Descending => "⬇",
        };
        format!("{field_name} {emoji}")
    }

    fn get_by_ident(&self, id: Uuid) -> Option<Streak> {
        let mut db = self.db.clone();
        db.get_by_id(&id.to_string()[..5])
    }

    fn search(&mut self, search: String) {
        self.streak_list = self.db.search(&search);
    }

    fn filter_by(&mut self, field: FilterByStatus) {
        self.filter_by = field;
        self.load_streaks();
    }
}

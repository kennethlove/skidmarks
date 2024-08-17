use crate::cli::get_database_url;
use crate::sorting::{SortByDirection, SortByField};
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
    _ = use_global_shortcut("CmdOrCtrl+Q", move || {
        std::process::exit(0);
    });
    _ = use_global_shortcut("CmdOrCtrl+R", move || {
        streaks.write().refresh();
    });
    rsx! {
        head {
            link { rel: "stylesheet", href: asset!("./assets/bulma.min.css") }
            link { rel: "stylesheet", href: asset!("./assets/streaks.css") }
        }
        div {
            head {
                link {
                    rel: "stylesheet",
                    href: "https://fonts.googleapis.com/icon?family=Material+Icons"
                }
            }

            header { style: "background-color: {catppuccin::PALETTE.mocha.colors.mauve.hex.to_string()}",
                h1 { style: "color: {catppuccin::PALETTE.mocha.colors.crust.hex.to_string()}",
                    "Skidmarks"
                }
            }
            main { {streak_table(streaks)} }
            aside { {streak_form(streaks)} }
        }
    }
}

fn streak_table(mut streaks: Signal<Streaks>) -> Element {
    // let mut streaks = use_signal(|| Streaks::new());
    rsx! {
        table { class: "table is-striped is-hoverable is-narrow is-fullwidth",
            thead {
                tr {
                    th { "Task" }
                    th { "Freq" }
                    th { "Status" }
                    th { "Last Check In" }
                    th { "Current Streak" }
                    th { "Longest Streak" }
                    th { "Total" }
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
                                button { class: "button", onclick: move |_| {
                                    streaks.write().checkin(&id)
                                    }, "✓"
                                }
                                button { class: "button", onclick: move |_| {
                                    streaks.write().delete(&id)
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
    // let mut streaks = use_signal(Streaks::new);

    rsx!(
        if !submitted_values.read().is_empty() {
            h2 { "Submitted!" }
        }

        form {
            id: "streak-form",
            class: "form",
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
                    _ => eprintln!("Invalid frequency"),
                };
                streaks.write().load_streaks();
            },
            input {
                class: "input",
                r#type: "text",
                name: "task",
                placeholder: "Task",
                oninput: move |event| {
                    values.set(event.values());
                }
            }
            div { class: "select",
                select {
                    class: "select",
                    name: "frequency",
                    oninput: move |event| {
                        values.set(event.values());
                    },
                    option { "Daily" }
                    option { "Weekly" }
                }
            }
            button { class: "button", r#type: "submit", "Add" }
        }
    )
}

struct Streaks {
    db: Database,
    streak_list: Vec<Streak>,
    sort_by: SortByField,
    sort_dir: SortByDirection,
}

impl Streaks {
    fn new() -> Self {
        let db = Database::new(&get_database_url()).expect("Failed to connect to database");
        let mut streaks = Self {
            db,
            streak_list: vec![],
            sort_by: SortByField::Task,
            sort_dir: SortByDirection::Ascending,
        };

        streaks.load_streaks();
        streaks
    }

    fn load_streaks(&mut self) {
        let sort_by = self.sort_by.clone();
        let sort_dir = self.sort_dir.clone();
        self.streak_list = self.db.get_sorted(sort_by, sort_dir);
        dbg!(self.streak_list.len());
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
}

use crate::cli::get_database_url;
use crate::sorting::{SortByDirection, SortByField};
use crate::{db::Database, streak::Streak};
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use uuid::Uuid;

pub fn main() {
    LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(WindowBuilder::new().with_resizable(true)))
        .launch(app)
}

fn app() -> Element {
    let mut streaks = use_signal(Streaks::new);

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

            header { style: format!(
                    "background-color: {}",
                    catppuccin::PALETTE.mocha.colors.mauve.hex.to_string(),
                ),
                i { class: "material-icons icon-menu", "menu" }
                h1 { style: "color: {catppuccin::PALETTE.mocha.colors.crust.hex.to_string()}",
                    "Skidmarks"
                }
            }
            main {
                table { class: "table",
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
                                                }, "x"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    tfoot {
                        tr {
                            td { "Streaks: {streaks.read().streak_list.len()}" }
                        }
                    }
                }
            }
        }
    }
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
}

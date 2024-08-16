use crate::cli::get_database_url;
use crate::sorting::{SortByDirection, SortByField};
use crate::{db::Database, streak::Streak};
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;

pub fn main() {
    LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(WindowBuilder::new().with_resizable(true)))
        .launch(app)
}

fn app() -> Element {
    let mut files = use_signal(Files::new);
    let mut streaks = use_signal(Streaks::new);

    rsx! {
        head {
            link { rel: "stylesheet", href: asset!("./assets/fileexplorer.css") }
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
                h1 { "Streaks: " }
                span {}
                i {
                    class: "material-icons",
                    onclick: move |_| files.write().go_up(),
                    "logout"
                }
            }
            main {
                table {
                    thead {
                        tr {
                            th { "Ident" }
                            th { "Task" }
                            th { "Freq" }
                            th { "Status" }
                            th { "Last Check In" }
                            th { "Current Streak" }
                            th { "Longest Streak" }
                            th { "Total" }
                        }
                    }
                    tbody {
                        for streak in streaks.read().streaks.iter() {
                            {
                                let id = &streak.id.to_string()[0..5];
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
                                    tr { class: "streak",
                                        td { class: "streak-id", "{id}" }
                                        td { class: "streak-name", "{streak_name}" }
                                        td { class: "streak-frequency", "{frequency}" }
                                        td { class: "streak-emoji", "{emoji}" }
                                        td { class: "streak-check-in", "{check_in}" }
                                        td { class: "streak-current-streak", "{current_streak}" }
                                        td { class: "streak-longest-streak", "{longest_streak}" }
                                        td { class: "streak-total-checkins", "{total_checkins}" }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(err) = files.read().err.as_ref() {
                    div {
                        code { "{err}" }
                        button { onclick: move |_| files.write().clear_err(), "x" }
                    }
                }
            }
        }
    }
}

struct Streaks {
    streaks: Vec<Streak>,
    sort_by: SortByField,
    sort_dir: SortByDirection,
}

impl Streaks {
    fn new() -> Self {
        let mut streaks = Self {
            streaks: vec![],
            sort_by: SortByField::Task,
            sort_dir: SortByDirection::Ascending,
        };

        streaks.load_streaks();
        streaks
    }

    fn load_streaks(&mut self) {
        let mut db = Database::new(&get_database_url()).expect("Failed to connect to database");
        self.streaks = db.get_all();
    }
}

struct Files {
    path_stack: Vec<String>,
    path_names: Vec<String>,
    err: Option<String>,
}

impl Files {
    fn new() -> Self {
        let mut files = Self {
            path_stack: vec!["./".to_string()],
            path_names: vec![],
            err: None,
        };

        files.reload_path_list();

        files
    }

    fn reload_path_list(&mut self) {
        let cur_path = self.path_stack.last().unwrap();
        let paths = match std::fs::read_dir(cur_path) {
            Ok(e) => e,
            Err(err) => {
                let err = format!("An error occurred: {err:?}");
                self.err = Some(err);
                self.path_stack.pop();
                return;
            }
        };
        let collected = paths.collect::<Vec<_>>();

        self.clear_err();
        self.path_names.clear();

        for path in collected {
            self.path_names
                .push(path.unwrap().path().display().to_string());
        }
    }

    fn go_up(&mut self) {
        if self.path_stack.len() > 1 {
            self.path_stack.pop();
        }
        self.reload_path_list();
    }

    fn enter_dir(&mut self, dir_id: usize) {
        let path = &self.path_names[dir_id];
        self.path_stack.push(path.clone());
        self.reload_path_list();
    }

    fn current(&self) -> &str {
        self.path_stack.last().unwrap()
    }

    fn clear_err(&mut self) {
        self.err = None;
    }
}

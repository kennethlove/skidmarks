use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{
    cli::get_database_url,
    db::Database,
    streak::{Streak, Frequency, Status}
};

pub fn main() {
    launch(App);
}

fn get_streaks() -> Result<Vec<Streak>, std::io::Error> {
    let mut db = Database::new(&get_database_url()).unwrap();
    Ok(db.get_all().unwrap())
}

fn check_in(streak_id: usize) {
    let mut db = Database::new(&get_database_url()).unwrap();
    let mut streak = db.get_one(streak_id as u32).unwrap();
    streak.checkin();
    let _ = db.update(streak_id as u32, &streak);
    let _ = db.save();
}

#[component]
fn StreakListing(streak_id: usize, streak: ReadOnlySignal<Streak>) -> Element {
    let Streak {
        task,
        frequency,
        last_checkin,
        total_checkins,
        ..
    } = streak();

    let date = match last_checkin {
        Some(date) => date.format("%Y-%m-%d").to_string(),
        None => "Never".to_string()
    };
    let emoji = streak().emoji_status();

    rsx! {
        tr {
            td { "{task}"}
            td { "{frequency}" }
            td { "{emoji}" }
            td { "{date}" }
            td { "{total_checkins}"}
            td {
                button {
                    class: "button is-primary",
                    onclick: move |_| { check_in(streak_id) },
                    "CHECK IN"
                }
            }
            td {
                button {
                    class: "button is-danger",
                    "REMOVE"
                }
            }
        }
    }
}

fn Streaks() -> Element {
    let streaks = get_streaks();

    match &streaks {
        Ok(streaks) => {
            rsx! {
                table {
                    class: "table",
                    width: "100%",
                    thead {
                        tr {
                            th { "Task" }
                            th { "Frequency" }
                            th { "Status" }
                            th { "Last Check In" }
                            th { "Total Check Ins" }
                            th {
                                colspan: 2,
                                "Tools"
                            }
                        }
                    }
                    tbody {
                        for (i, streak) in streaks.into_iter().enumerate() {
                            StreakListing { streak_id: i, streak: streak.clone() }
                        }
                    }
                }
            }

        }
        Err(e) => rsx! { "error: {e}" }
    }
}

fn App() -> Element {
    rsx! {
        link {
            rel: "stylesheet",
            href: "assets/bulma.min.css"
        }
        div {
            class: "container is-fluid",
            div {
                class: "panel",
                p {
                    class: "panel-heading",
                    "Skidmarks"
                }
                p {
                    class: "panel-tabs",
                    a {
                        class: "is-active",
                        "All"
                    }
                    a {
                        "To Do"
                    }
                    a {
                        "Done"
                    }
                }
                div {
                    class: "panel-block",
                    p {
                        class: "control",
                        input {
                            class: "input is-link",
                            r#type: "search",
                            placeholder: "Search"
                        }
                    }
                }
                div {
                    class: "panel-block",
                    Streaks { }
                }
                div {
                    class: "panel-block",
                    a {
                        class: "button is-small",
                        "Add New"
                    }
                }
            }
        }
    }
}

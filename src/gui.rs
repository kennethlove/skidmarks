#![allow(non_snake_case)]

use std::thread::Scope;
use chrono::{DateTime, Local, Utc};
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

#[component]
fn StreaksTable() -> Element {
    let mut db = use_context::<Database>();
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
                for (i, streak) in db.get_all().unwrap().into_iter().enumerate() {
                    StreakListing { streak_id: i }
                }
            }
        }
    }
}

#[component]
fn StreakListing(streak_id: usize) -> Element {
    let mut db = use_context::<Database>();
    let initial_streak = db.get_one(streak_id as u32).unwrap();
    let mut streak = use_signal(|| initial_streak);

    let date = match streak().last_checkin {
        Some(date) => date.format("%Y-%m-%d").to_string(),
        None => "Never".to_string()
    };
    let emoji = streak().emoji_status();

    rsx! {
        tr {
            td { "{streak().task}"}
            td { "{streak().frequency}" }
            td { "{emoji}" }
            td { "{date}" }
            td { "{streak().total_checkins}"}
            td {
                button {
                    class: "button is-primary is-small",
                    onclick: move |_| {
                        let mut updated_streak = ((*streak))().clone();
                        updated_streak.checkin();
                        db.update(streak_id as u32, &updated_streak).unwrap();
                        db.save().unwrap();
                        streak.set(updated_streak);
                    },
                    "CHECK IN"
                }
            }
            td {
                button {
                    class: "button is-danger is-small",
                    "REMOVE"
                }
            }
        }
    }
}

#[component]
fn Streaks() -> Element {
    rsx! {
        StreaksTable { }
    }
}

fn App() -> Element {
    use_context_provider(|| Database::new(&get_database_url()).unwrap());

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

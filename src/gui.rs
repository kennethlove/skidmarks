#![allow(non_snake_case)]

use dioxus::prelude::*;
use native_dialog::{MessageDialog, MessageType};
use uuid::Uuid;

use crate::{cli::get_database_url, db::Database};
use crate::streak::Streak;

pub fn main() {
    launch(App);
}

#[component]
fn StreaksTable(mut db: Signal<Database>) -> Element {
    // let mut db = use_context::<Database>();
    let streaks = db().get_all().unwrap();
    let streaks = streaks.into_values();

    rsx! {
        table { class: "table", width: "100%",
            thead {
                tr {
                    th { "Task" }
                    th { "Frequency" }
                    th { "Status" }
                    th { "Last Check In" }
                    th { "Total Check Ins" }
                    th { colspan: 2, "Tools" }
                }
            }
            tbody {
                for streak in streaks {
                    StreakListing { streak: streak, db: db }
                }
            }
        }
    }
}

fn check_in(streak_id: Uuid) -> Streak {
    let mut db = use_context::<Database>();
    let mut streak = db.get_one(streak_id).unwrap();
    streak.checkin();
    db.update(streak_id, streak.clone()).unwrap();
    db.save().unwrap();

    streak
}

fn add_streak(streak: Streak, mut db: Signal<Database>) {
    db().add(streak).unwrap();
    db().save().unwrap();
}

#[component]
fn CheckInButton(streak_id: Uuid, db: Signal<Database>) -> Element {
    // let mut db = use_context::<Database>();
    rsx! {
        button {
            class: "button is-primary is-small",
            onclick: move |_| {
                let updated_streak = check_in(streak_id);
                db().update(streak_id, updated_streak).unwrap();
            },
            "CHECK IN"
        }
    }
}

#[component]
fn RemoveButton(streak_id: Uuid, db: Signal<Database>) -> Element {
    rsx! {
        button {
            class: "button is-danger is-small",
            onclick: move |_| {
                let confirm = MessageDialog::new()
                    .set_type(MessageType::Info)
                    .set_title("Remove Streak")
                    .set_text("Are you sure you want to remove this streak?")
                    .show_confirm()
                    .unwrap();

                match confirm {
                    true => {
                        db().delete(streak_id).unwrap();
                        db().save().unwrap();
                    }
                    _ => {}
                }
            },
            "REMOVE"
        }
    }
}

#[component]
fn StreakListing(streak: Streak, db: Signal<Database>) -> Element {
    let date = match streak.last_checkin {
        Some(date) => date.format("%Y-%m-%d").to_string(),
        None => "Never".to_string(),
    };
    let emoji = streak.emoji_status();

    rsx! {
        tr {
            td { "{streak.task}" }
            td { "{streak.frequency}" }
            td { "{emoji}" }
            td { "{date}" }
            td { "{streak.total_checkins}" }
            td {
                CheckInButton { streak_id: streak.id, db: db }
            }
            td {
                RemoveButton { streak_id: streak.id, db: db }
            }
        }
    }
}

#[component]
fn Streaks(db: Signal<Database>) -> Element {
    rsx! {
        div { class: "panel-block", StreaksTable { db } }
        div { class: "panel-block", NewStreak { db } }
    }
}

fn App() -> Element {
    let mut db = use_signal(|| Database::new(&get_database_url()).unwrap());

    rsx! {
        link { rel: "stylesheet", href: "assets/bulma.min.css" }
        div { class: "container is-fluid",
            div { class: "panel",
                p { class: "panel-heading", "Skidmarks" }
                div { class: "panel-block",
                    p { class: "control",
                        input {
                            class: "input",
                            r#type: "search",
                            placeholder: "Search"
                        }
                    }
                }
                Streaks { db }
            }
        }
    }
}

#[component]
fn NewStreak(db: Signal<Database>) -> Element {
    let mut new_streak = use_signal(|| "".to_string());
    let mut new_streak_type = use_signal(|| "Daily".to_string());

    rsx! {
        input {
            class: "input",
            r#type: "text",
            placeholder: "New Streak",
            oninput: move |evt| new_streak.set(evt.value().clone())
        }
        div { class: "select",
            select {
                class: "select",
                oninput: move |evt| { new_streak_type.set(evt.data.value().clone()) },
                option { "Daily" }
                option { "Weekly" }
            }
        }
        button {
            class: "button",
            onclick: move |_| {
                let new = match new_streak_type().as_str() {
                    "Daily" => {
                        let streak = Streak::new_daily(new_streak().clone());
                        streak
                    }
                    "Weekly" => {
                        let streak = Streak::new_weekly(new_streak().clone());
                        streak
                    }
                    _ => Streak::default(),
                };
                add_streak(new, db);
            },
            "Add New"
        }
    }
}

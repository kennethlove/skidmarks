#![allow(non_snake_case)]

use dioxus::prelude::*;
use native_dialog::{MessageDialog, MessageType};
use crate::{
    cli::get_database_url,
    db::Database,
};
use crate::streak::Streak;

pub fn main() {
    launch(App);
}

#[component]
fn StreaksTable() -> Element {
    let db = use_context::<Database>();
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
                for i in 0..db.num_tasks() {
                    StreakListing { streak_id: i }
                }
            }
        }
    }
}

fn check_in(streak_id: usize) -> Streak {
    let mut db = use_context::<Database>();
    let mut streak = db.get_one(streak_id as u32).unwrap();
    streak.checkin();
    db.update(streak_id as u32, &streak).unwrap();
    db.save().unwrap();

    streak

}

#[component]
fn StreakListing(streak_id: usize) -> Element {
    let mut db = use_context::<Database>();
    let initial_streak = match db.get_one(streak_id as u32) {
        Some(streak) => streak,
        None => { return None; }
    };
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
                        let updated_streak = check_in(streak_id);
                        streak.set(updated_streak);
                    },
                    "CHECK IN"
                }
            }
            td {
                button {
                    class: "button is-danger is-small",
                    onclick: move |_| {
                        let yes = MessageDialog::new()
                            .set_type(MessageType::Info)
                            .set_title("Delete Streak")
                            .set_text("Are you sure you want to delete this streak?")
                            .show_confirm().unwrap();

                        if yes {
                            db.delete(streak_id as u32).unwrap();
                            db.save().unwrap();
                            streak.write();
                        }
                    },
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

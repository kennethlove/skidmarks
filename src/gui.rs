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

pub fn get_streaks() -> Result<Vec<Streak>, std::io::Error> {
    let db = Database::new(&get_database_url());
    Ok(db.unwrap().get_all().unwrap())
}

#[component]
fn StreakListing(streak: ReadOnlySignal<Streak>) -> Element {
    let Streak {
        task,
        frequency,
        last_checkin,
        total_checkins,
        ..
    } = streak();

    let date = streak().last_checkin?.format("%Y-%m-%d");
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
                        for streak in streaks {
                            StreakListing { streak: streak.clone() }
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

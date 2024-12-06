use dioxus::dioxus_core::internal::generational_box::GenerationalRef;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use std::cell::Ref;

use streak::Streak;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

#[component]
fn App() -> Element {
    // Build cool things ✌️

    rsx! {
        // Global app resources
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        Router::<Route> {}
    }
}

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        Streaks {}
        Echo {}
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            Link {
                to: Route::Home {},
                "Home"
            }
        }

        Outlet::<Route> {}
    }
}

/// Echo component that demonstrates fullstack server functions.
#[component]
fn Echo() -> Element {
    let mut response = use_signal(|| String::new());

    rsx! {
        div {
            id: "echo",
            h4 { "ServerFn Echo" }
            input {
                placeholder: "Type here to echo...",
                oninput:  move |event| async move {
                    let data = echo_server(event.value()).await.unwrap();
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p {
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}

#[component]
fn Streaks() -> Element {
    let mut streak_signal: Signal<Vec<Streak>> = use_signal(|| vec![]);
    let streak_request = use_resource(move || async move { streak_server().await });

    rsx! {
        div {
            id: "streaks",
            h4 {
                "Streaks"
            }
            match streak_request() {
                Some(Ok(response)) => rsx! {
                    ul {
                        for streak in response {
                            li { {streak.task }}
                        }
                    }
                },
                Some(Err(err)) => rsx! {
                    p {
                        {format!("Trouble loading streaks: {}", err)}
                    }
                },
                None => rsx! {
                    p {
                        "Loading..."
                    }
                }
            }
        }
    }
}

/// Echo the user input on the server.
#[server(EchoServer)]
async fn echo_server(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

#[server(StreakServer)]
async fn streak_server() -> Result<Vec<Streak>, ServerFnError> {
    let response = reqwest::get("http://localhost:3000/streak").await;
    let streaks = response?.json::<Vec<Streak>>().await.unwrap();
    dioxus_logger::tracing::info!("streak_server done");
    Ok(streaks)
}

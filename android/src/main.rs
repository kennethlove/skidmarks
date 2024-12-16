use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use android::components::App;
use android::create_db;

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    create_db().expect("Failed to create database");
    launch(App);
}


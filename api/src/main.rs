use axum::error_handling::HandleErrorLayer;
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, LOCATION},
    Response,
};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{extract::Path, http::StatusCode, BoxError, Router};
use skidmarks_api::{endpoints, AppState};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use shared::db::{get_database_url, Database};

#[tokio::main]
async fn main() {
    let shared_state = AppState::new();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors_layer = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_headers(vec!["content-type".parse().unwrap()])
        .allow_methods(vec![
            "GET".parse().unwrap(),
            "POST".parse().unwrap(),
            "PUT".parse().unwrap(),
            "DELETE".parse().unwrap(),
        ]);

    let timeout_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|error: BoxError| async move {
            if error.is::<tower::timeout::error::Elapsed>() {
                Ok(StatusCode::REQUEST_TIMEOUT)
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                ))
            }
        }))
        .timeout(Duration::from_secs(10));

    let app = Router::new()
        .route("/", get(endpoints::root))
        .route("/streak", get(endpoints::list).post(endpoints::create))
        .route("/streak/{identifier}", get(endpoints::detail))
        .layer(cors_layer)
        .layer(TraceLayer::new_for_http())
        .layer(timeout_layer)
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

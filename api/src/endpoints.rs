use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use shared::streak::Streak;
use uuid::Uuid;

pub async fn root() -> impl IntoResponse {
    r#"Skidmarks API

    GET /streak - list of streaks
    GET /streak/<id> - an individual streak
    "#
}

pub async fn list(State(state): State<AppState>) -> Json<Vec<Streak>> {
    let streaks = state.database.lock().unwrap().get_all();
    Json(streaks)
}

pub async fn detail(
    Path(identifier): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let streak = state.database.lock().unwrap().get_one(identifier);
    match streak {
        Some(streak) => Ok(Json(streak)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create(
    State(mut state): State<AppState>,
    Json(streak): Json<Streak>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut db = state.database.lock().unwrap();
    let streak = db.add(streak);
    db.save().unwrap();
    match streak {
        Ok(streak) => Ok((StatusCode::CREATED, Json(streak))),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

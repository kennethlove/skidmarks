use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use streak::filtering::FilterByStatus;
use streak::sorting::get_sort_order;
use streak::{Frequency, Streak};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct QueryParams {
    sort_by: Option<String>,
    search: Option<String>,
    frequency: Option<String>,
    status: Option<String>,
}

impl Default for QueryParams {
    fn default() -> Self {
        Self {
            sort_by: None,
            search: None,
            frequency: None,
            status: None,
        }
    }
}

pub async fn root() -> impl IntoResponse {
    r#"Skidmarks API

    GET /streak - list of streaks
        * ?search=<TERM> - search for streaks with a term
        * ?frequency=daily|weekly - filter streaks by frequency
        * ?status=all|done|missed|waiting - filter streaks by status
        * ?sort_by=id|task|frequency|last_checkin|current_streak|longest_streak|total_checkins - sort streaks
    POST /streak - create a streak
    GET /streak/<id> - an individual streak
    PUT /streak/<id> - update an individual streak
    DELETE /streak/<id> - delete an individual streak
    "#
}

pub async fn list(
    params: Option<Query<QueryParams>>,
    State(state): State<AppState>,
) -> Json<Vec<Streak>> {
    let mut db = state.database.lock().unwrap();
    let mut streaks: Vec<Streak> = db.get_all();

    let Query(params) = params.unwrap_or_default();
    if let Some(sort_by) = params.sort_by {
        let (field, direction) = get_sort_order(&sort_by);
        streaks = db.get_sorted(&field, &direction);
    }

    if let Some(search) = params.search {
        streaks = db.search(&search);
    }

    if let Some(frequency) = params.frequency {
        match frequency.to_lowercase().as_str() {
            "daily" => {
                streaks = db.get_by_frequency(Frequency::Daily);
            }
            "weekly" => {
                streaks = db.get_by_frequency(Frequency::Weekly);
            }
            _ => {}
        }
    }

    if let Some(status) = params.status {
        let filter = match status.to_lowercase().as_str() {
            "done" => FilterByStatus::Done,
            "missed" => FilterByStatus::Missed,
            "waiting" => FilterByStatus::Waiting,
            _ => FilterByStatus::All,
        };
        streaks = db.get_filtered(filter);
    }

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
    State(state): State<AppState>,
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

pub async fn update(
    Path(identifier): Path<Uuid>,
    State(state): State<AppState>,
    Json(streak): Json<Streak>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut db = state.database.lock().unwrap();
    let existing_streak = db.get_one(identifier);
    match existing_streak {
        Some(mut current_streak) => {
            current_streak.longest_streak = streak.longest_streak;
            current_streak.current_streak = streak.current_streak;
            current_streak.frequency = streak.frequency;
            current_streak.last_checkin = streak.last_checkin;
            current_streak.total_checkins = streak.total_checkins;
            current_streak.task = streak.task;

            match db.update(identifier, current_streak) {
                Ok(updated_streak) => {
                    db.save().unwrap();
                    Ok((StatusCode::OK, Json(updated_streak)))
                }
                _ => Err(StatusCode::BAD_REQUEST),
            }
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete(
    Path(identifier): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut db = state.database.lock().unwrap();
    match db.delete(identifier) {
        Ok(_) => {
            db.save().unwrap();
            Ok(StatusCode::NO_CONTENT)
        }
        _ => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn checkin(
    Path(identifier): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut db = state.database.lock().unwrap();
    match db.checkin(identifier) {
        Ok(updated_streak) => {
            db.save().unwrap();
            Ok((StatusCode::OK, Json(updated_streak)))
        }
        Err(e) => {
            eprintln!("{}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

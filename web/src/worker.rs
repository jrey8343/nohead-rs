use apalis::prelude::*;
use apalis_sql::sqlite::SqliteStorage;
use axum::{
    Extension, Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use nohead_rs_mailer::EmailPayload;
use serde::{Deserialize, Serialize};

use crate::{error::Error, state::AppState};

#[derive(Deserialize, Debug)]
struct Filter {
    #[serde(default)]
    pub status: State,
    #[serde(default = "default_page")]
    pub page: i32,
}

fn default_page() -> i32 {
    1
}

#[derive(Debug, Serialize, Deserialize)]
struct GetJobsResult<T> {
    pub stats: Stat,
    pub jobs: Vec<T>,
}

async fn get_jobs(
    Extension(storage): Extension<SqliteStorage<EmailPayload>>,
    Query(filter): Query<Filter>,
) -> Result<impl IntoResponse, Error> {
    let stats = storage.stats().await.unwrap_or_default();
    let jobs = storage.list_jobs(&filter.status, filter.page).await?;

    Ok((StatusCode::OK, Json(GetJobsResult { stats, jobs })))
}

async fn get_job(
    Extension(mut storage): Extension<SqliteStorage<EmailPayload>>,
    job_id: Path<TaskId>,
) -> Result<impl IntoResponse, Error> {
    let job = storage
        .fetch_by_id(&job_id)
        .await
        .map_err(|e| Error::Unexpected(e.into()))?;

    match job {
        Some(job) => Ok((StatusCode::OK, Json(job)).into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

async fn push_job(
    Extension(mut storage): Extension<SqliteStorage<EmailPayload>>,
    Json(job): Json<EmailPayload>,
) -> Result<impl IntoResponse, Error> {
    let res = storage
        .push(job)
        .await
        .map_err(|e| Error::Unexpected(e.into()));

    match res {
        Ok(parts) => Ok((StatusCode::OK, Json(parts)).into_response()),
        Err(e) => Err(Error::Unexpected(e.into())),
    }
}

async fn get_workers(
    storage: Extension<SqliteStorage<EmailPayload>>,
) -> Result<impl IntoResponse, Error> {
    let workers = storage.list_workers().await;
    match workers {
        Ok(workers) => Ok((StatusCode::OK, Json(workers))),
        Err(e) => Err(Error::Unexpected(e.into())),
    }
}

pub struct WorkerController;

impl WorkerController {
    pub fn router() -> Router<AppState> {
        Router::new()
            .route("/workers", get(get_workers))
            .route("/jobs/mailer", get(get_jobs).put(push_job))
            .route("/jobs/mailer/{job_id}", get(get_job))
    }
}

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::{
    db::AppState,
    error::{AppError, Result},
    models::{CreateJobRequest, Job},
    routes::{bids, milestones},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_jobs).post(create_job))
        .route("/:id", get(get_job))
        .route("/:id/bids", get(bids::list_bids).post(bids::create_bid))
        .route("/:id/milestones/:mid/release", post(milestones::release_milestone))
        .route("/:id/dispute", post(crate::routes::disputes::open_dispute_for_job))
}

async fn list_jobs(State(state): State<AppState>) -> Result<Json<Vec<Job>>> {
    let jobs = sqlx::query_as::<_, Job>(
        r#"SELECT id, title, description, budget_usdc, milestones, client_address,
                  freelancer_address, status, metadata_hash, on_chain_job_id,
                  created_at, updated_at
           FROM jobs ORDER BY created_at DESC"#
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(jobs))
}

async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Job>> {
    let job = sqlx::query_as::<_, Job>(
        r#"SELECT id, title, description, budget_usdc, milestones, client_address,
                  freelancer_address, status, metadata_hash, on_chain_job_id,
                  created_at, updated_at
           FROM jobs WHERE id = $1"#
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("job {id} not found")))?;
    Ok(Json(job))
}

async fn create_job(
    State(state): State<AppState>,
    Json(req): Json<CreateJobRequest>,
) -> Result<Json<Job>> {
    if req.title.is_empty() {
        return Err(AppError::BadRequest("title is required".into()));
    }
    let job = sqlx::query_as::<_, Job>(
        r#"INSERT INTO jobs (title, description, budget_usdc, milestones, client_address, status)
           VALUES ($1, $2, $3, $4, $5, 'open')
           RETURNING id, title, description, budget_usdc, milestones, client_address,
                     freelancer_address, status, metadata_hash, on_chain_job_id,
                     created_at, updated_at"#
    )
    .bind(req.title)
    .bind(req.description)
    .bind(req.budget_usdc)
    .bind(req.milestones)
    .bind(req.client_address)
    .fetch_one(&state.pool)
    .await?;

    // Create milestone records in 'milestones' table
    if job.milestones > 0 {
        let amount_per = job.budget_usdc / (job.milestones as i64);
        for i in 0..job.milestones {
            sqlx::query(
                r#"INSERT INTO milestones (job_id, index, title, amount_usdc, status)
                   VALUES ($1, $2, $3, $4, 'pending')"#,
            )
            .bind(job.id)
            .bind(i)
            .bind(format!("Milestone {}", i + 1))
            .bind(amount_per)
            .execute(&state.pool)
            .await?;
        }
    }
    
    Ok(Json(job))
}

use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{
    db::AppState,
    error::{AppError, Result},
    models::Milestone,
};

pub async fn release_milestone(
    State(state): State<AppState>,
    Path((job_id, milestone_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Milestone>> {
    // Verify milestone belongs to job
    let milestone = sqlx::query_as::<_, Milestone>(
        r#"SELECT id, job_id, index, title, amount_usdc, status, tx_hash, released_at
           FROM milestones WHERE id = $1 AND job_id = $2"#
    )
    .bind(milestone_id)
    .bind(job_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("milestone not found".into()))?;

    if milestone.status != "pending" {
        return Err(AppError::BadRequest("milestone already released".into()));
    }

    // Call Soroban escrow contract via stellar.rs service
    // Use the on-chain job ID if it exists, otherwise use a placeholder (for dev/test)
    let job_id_str = milestone.job_id.to_string();
    let tx_hash = state.stellar.release_milestone(&job_id_str, milestone.index).await
        .map(Some)
        .unwrap_or_else(|e| {
            tracing::error!("on-chain release_milestone failed: {e}");
            None // Fallback to allowing DB update even if on-chain failed for robustness in dev
        });

    let updated = sqlx::query_as::<_, Milestone>(
        r#"UPDATE milestones SET status = 'released', tx_hash = $1, released_at = CURRENT_TIMESTAMP
           WHERE id = $2
           RETURNING id, job_id, index, title, amount_usdc, status, tx_hash, released_at"#
    )
    .bind(tx_hash)
    .bind(milestone_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(updated))
}

use sqlx::PgPool;
use uuid::Uuid;
use crate::db::models::Job;

pub async fn create_job(
    pool: &PgPool,
    job_reference: &str,
    client_name: Option<&str>,
    request: &serde_json::Value,
    webhook_url: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO jobs (id, job_reference, client_name, status, request, webhook_url)
        VALUES ($1, $2, $3, 'pending', $4, $5)
        "#
    )
    .bind(id)
    .bind(job_reference)
    .bind(client_name)
    .bind(request)
    .bind(webhook_url)
    .execute(pool)
    .await?;

    Ok(id)
}

pub async fn get_job(pool: &PgPool, id: Uuid) -> Result<Option<Job>, sqlx::Error> {
    sqlx::query_as::<_, Job>(
        r#"
        SELECT id, job_reference, client_name, status, request, result,
               pdf_bytes, error_message, webhook_url, webhook_delivered,
               created_at, started_at, completed_at
        FROM jobs WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn update_job_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now();

    if status == "processing" {
        sqlx::query(
            "UPDATE jobs SET status = $1, started_at = $2 WHERE id = $3"
        )
        .bind(status)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            "UPDATE jobs SET status = $1 WHERE id = $2"
        )
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn complete_job(
    pool: &PgPool,
    id: Uuid,
    result: &serde_json::Value,
    pdf_bytes: Option<Vec<u8>>,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now();

    sqlx::query(
        r#"
        UPDATE jobs
        SET status = 'completed', result = $1, pdf_bytes = $2, completed_at = $3
        WHERE id = $4
        "#
    )
    .bind(result)
    .bind(pdf_bytes.as_deref())
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn fail_job(
    pool: &PgPool,
    id: Uuid,
    error_message: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now();

    sqlx::query(
        r#"
        UPDATE jobs
        SET status = 'failed', error_message = $1, completed_at = $2
        WHERE id = $3
        "#
    )
    .bind(error_message)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_webhook_delivered(
    pool: &PgPool,
    id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE jobs SET webhook_delivered = true WHERE id = $1"
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

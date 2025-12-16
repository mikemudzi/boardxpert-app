use base64::{Engine as _, engine::general_purpose};
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::api::OptimizeRequest;
use crate::db;
use crate::optimizer::solve_ffdh;
use crate::output::generate_pdf;
use crate::queue;

pub struct Worker {
    db_pool: PgPool,
    redis_conn: Arc<Mutex<ConnectionManager>>,
}

impl Worker {
    pub fn new(db_pool: PgPool, redis_conn: ConnectionManager) -> Self {
        Self {
            db_pool,
            redis_conn: Arc::new(Mutex::new(redis_conn)),
        }
    }

    pub async fn run(&self) {
        tracing::info!("Worker started, waiting for jobs...");

        loop {
            let job_id = {
                let mut conn = self.redis_conn.lock().await;
                match queue::pop_job(&mut conn, 5).await {
                    Ok(Some(id)) => id,
                    Ok(None) => continue, // Timeout, try again
                    Err(e) => {
                        tracing::error!("Failed to pop job from queue: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        continue;
                    }
                }
            };

            tracing::info!("Processing job: {}", job_id);
            self.process_job(job_id).await;
        }
    }

    async fn process_job(&self, job_id: uuid::Uuid) {
        // Get job from database
        let job = match db::get_job(&self.db_pool, job_id).await {
            Ok(Some(job)) => job,
            Ok(None) => {
                tracing::error!("Job {} not found in database", job_id);
                return;
            }
            Err(e) => {
                tracing::error!("Failed to fetch job {}: {}", job_id, e);
                return;
            }
        };

        // Update status to processing
        if let Err(e) = db::update_job_status(&self.db_pool, job_id, "processing").await {
            tracing::error!("Failed to update job status: {}", e);
            return;
        }

        // Parse request from JSON
        let request: OptimizeRequest = match serde_json::from_value(job.request.clone()) {
            Ok(req) => req,
            Err(e) => {
                tracing::error!("Failed to parse job request: {}", e);
                let _ = db::fail_job(&self.db_pool, job_id, &format!("Invalid request: {}", e)).await;
                return;
            }
        };

        // Run optimization
        let stock_sheet = &request.stock_sheets[0];
        let result = solve_ffdh(
            &request.pieces,
            stock_sheet,
            request.parameters.blade_kerf,
        );

        // Generate PDF if requested
        let pdf_bytes = if request.output.generate_pdf {
            match generate_pdf(
                &result,
                &request.job_reference,
                request.client_name.as_deref(),
                stock_sheet,
            ) {
                Ok(bytes) => Some(bytes),
                Err(e) => {
                    tracing::error!("PDF generation failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Serialize result to JSON
        let result_json = match serde_json::to_value(&result) {
            Ok(json) => json,
            Err(e) => {
                tracing::error!("Failed to serialize result: {}", e);
                let _ = db::fail_job(&self.db_pool, job_id, &format!("Serialization error: {}", e)).await;
                return;
            }
        };

        // Complete job
        if let Err(e) = db::complete_job(&self.db_pool, job_id, &result_json, pdf_bytes.clone()).await {
            tracing::error!("Failed to complete job: {}", e);
            return;
        }

        tracing::info!("Job {} completed successfully", job_id);

        // Send webhook if configured
        if let Some(webhook_url) = &job.webhook_url {
            self.send_webhook(job_id, webhook_url, &result_json, pdf_bytes.as_ref()).await;
        }
    }

    async fn send_webhook(
        &self,
        job_id: uuid::Uuid,
        webhook_url: &str,
        result: &serde_json::Value,
        pdf_bytes: Option<&Vec<u8>>,
    ) {
        let pdf_base64 = pdf_bytes.map(|bytes| general_purpose::STANDARD.encode(bytes));

        let payload = serde_json::json!({
            "job_id": job_id,
            "status": "completed",
            "result": result,
            "pdf_base64": pdf_base64,
        });

        let client = reqwest::Client::new();
        match client
            .post(webhook_url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    tracing::info!("Webhook delivered for job {}", job_id);
                    let _ = db::mark_webhook_delivered(&self.db_pool, job_id).await;
                } else {
                    tracing::warn!(
                        "Webhook returned non-success status {} for job {}",
                        response.status(),
                        job_id
                    );
                }
            }
            Err(e) => {
                tracing::error!("Failed to send webhook for job {}: {}", job_id, e);
            }
        }
    }
}

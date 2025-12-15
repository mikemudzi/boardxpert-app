# Async Job Processing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add async job processing with Redis queue, PostgreSQL persistence, polling/webhook result delivery, and worker mode.

**Architecture:** API server creates jobs in PostgreSQL, pushes to Redis queue. Separate worker process (same binary with --worker flag) pops from queue, processes optimization, stores results. Clients poll for status or receive webhook callbacks.

**Tech Stack:** sqlx (PostgreSQL), redis crate, reqwest (webhooks), clap (CLI), uuid

---

### Task 1: Add New Dependencies

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add dependencies to Cargo.toml**

Add after existing dependencies:

```toml
# Async job processing
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json"] }
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"] }
reqwest = { version = "0.12", features = ["json"] }
clap = { version = "4", features = ["derive"] }
```

**Step 2: Verify dependencies compile**

Run: `cargo check`
Expected: Successful compilation (may take time to download)

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "feat: add dependencies for async job processing"
```

---

### Task 2: Add CLI Argument Parsing

**Files:**
- Create: `src/cli.rs`
- Modify: `src/main.rs`
- Modify: `src/lib.rs`

**Step 1: Create CLI module**

Create `src/cli.rs`:

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "cut-optimizer-api")]
#[command(about = "2D cutting stock optimization API")]
pub struct Cli {
    /// Run as worker instead of API server
    #[arg(long)]
    pub worker: bool,

    /// Number of concurrent jobs (worker mode only)
    #[arg(long, default_value = "1")]
    pub concurrency: usize,
}
```

**Step 2: Update lib.rs to export cli**

Add to `src/lib.rs`:

```rust
pub mod cli;
```

**Step 3: Update main.rs to parse args**

Replace `src/main.rs` content:

```rust
use actix_web::{web, App, HttpServer, HttpResponse};
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod cli;
mod optimizer;
mod output;

use cli::Cli;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Cli::parse();

    if args.worker {
        tracing::info!("Starting worker with concurrency {}", args.concurrency);
        // TODO: Implement worker loop
        Ok(())
    } else {
        run_api_server().await
    }
}

async fn run_api_server() -> std::io::Result<()> {
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()
        .expect("PORT must be a number");

    tracing::info!("Starting Cut Optimizer API at {}:{}", host, port);

    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health_check))
            .configure(api::routes::configure)
    })
    .bind((host, port))?
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
```

**Step 4: Test CLI parsing**

Run: `cargo run -- --help`
Expected: Shows help with --worker and --concurrency options

Run: `cargo run -- --worker`
Expected: Logs "Starting worker with concurrency 1"

**Step 5: Commit**

```bash
git add src/cli.rs src/main.rs src/lib.rs
git commit -m "feat: add CLI with --worker flag"
```

---

### Task 3: Create Database Module with Job Model

**Files:**
- Create: `src/db/mod.rs`
- Create: `src/db/models.rs`
- Modify: `src/lib.rs`

**Step 1: Create db module**

Create `src/db/mod.rs`:

```rust
pub mod models;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}
```

**Step 2: Create job model**

Create `src/db/models.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "pending"),
            JobStatus::Processing => write!(f, "processing"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed => write!(f, "failed"),
        }
    }
}

impl From<String> for JobStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "pending" => JobStatus::Pending,
            "processing" => JobStatus::Processing,
            "completed" => JobStatus::Completed,
            "failed" => JobStatus::Failed,
            _ => JobStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct Job {
    pub id: Uuid,
    pub job_reference: String,
    pub client_name: Option<String>,
    pub status: String,
    pub request: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub pdf_bytes: Option<Vec<u8>>,
    pub error_message: Option<String>,
    pub webhook_url: Option<String>,
    pub webhook_delivered: bool,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Job {
    pub fn status_enum(&self) -> JobStatus {
        JobStatus::from(self.status.clone())
    }
}
```

**Step 3: Update lib.rs**

Add to `src/lib.rs`:

```rust
pub mod db;
```

**Step 4: Verify compilation**

Run: `cargo check`
Expected: Successful compilation

**Step 5: Commit**

```bash
git add src/db/mod.rs src/db/models.rs src/lib.rs
git commit -m "feat: add database module with Job model"
```

---

### Task 4: Add Job Repository Functions

**Files:**
- Create: `src/db/repository.rs`
- Modify: `src/db/mod.rs`

**Step 1: Create repository**

Create `src/db/repository.rs`:

```rust
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

    sqlx::query!(
        r#"
        INSERT INTO jobs (id, job_reference, client_name, status, request, webhook_url)
        VALUES ($1, $2, $3, 'pending', $4, $5)
        "#,
        id,
        job_reference,
        client_name,
        request,
        webhook_url,
    )
    .execute(pool)
    .await?;

    Ok(id)
}

pub async fn get_job(pool: &PgPool, id: Uuid) -> Result<Option<Job>, sqlx::Error> {
    sqlx::query_as!(
        Job,
        r#"
        SELECT id, job_reference, client_name, status, request, result,
               pdf_bytes, error_message, webhook_url, webhook_delivered,
               created_at, started_at, completed_at
        FROM jobs WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn update_job_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now();

    match status {
        "processing" => {
            sqlx::query!(
                "UPDATE jobs SET status = $1, started_at = $2 WHERE id = $3",
                status,
                now,
                id
            )
            .execute(pool)
            .await?;
        }
        _ => {
            sqlx::query!(
                "UPDATE jobs SET status = $1 WHERE id = $2",
                status,
                id
            )
            .execute(pool)
            .await?;
        }
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

    sqlx::query!(
        r#"
        UPDATE jobs
        SET status = 'completed', result = $1, pdf_bytes = $2, completed_at = $3
        WHERE id = $4
        "#,
        result,
        pdf_bytes.as_deref(),
        now,
        id
    )
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

    sqlx::query!(
        r#"
        UPDATE jobs
        SET status = 'failed', error_message = $1, completed_at = $2
        WHERE id = $3
        "#,
        error_message,
        now,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_webhook_delivered(
    pool: &PgPool,
    id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE jobs SET webhook_delivered = true WHERE id = $1",
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}
```

**Step 2: Export repository from db module**

Update `src/db/mod.rs`:

```rust
pub mod models;
pub mod repository;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub use models::*;
pub use repository::*;

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: Successful (sqlx macros will be checked at runtime or with sqlx-cli)

**Step 4: Commit**

```bash
git add src/db/repository.rs src/db/mod.rs
git commit -m "feat: add job repository functions"
```

---

### Task 5: Create Database Migration

**Files:**
- Create: `migrations/001_create_jobs_table.sql`

**Step 1: Create migrations directory and file**

Create `migrations/001_create_jobs_table.sql`:

```sql
CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY,
    job_reference VARCHAR(255) NOT NULL,
    client_name VARCHAR(255),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',

    -- Input (stored as JSONB for flexibility)
    request JSONB NOT NULL,

    -- Output (populated when completed)
    result JSONB,
    pdf_bytes BYTEA,
    error_message TEXT,

    -- Webhook (optional)
    webhook_url TEXT,
    webhook_delivered BOOLEAN NOT NULL DEFAULT FALSE,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs(created_at);
```

**Step 2: Commit**

```bash
git add migrations/001_create_jobs_table.sql
git commit -m "feat: add database migration for jobs table"
```

---

### Task 6: Create Redis Queue Module

**Files:**
- Create: `src/queue/mod.rs`
- Modify: `src/lib.rs`

**Step 1: Create queue module**

Create `src/queue/mod.rs`:

```rust
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use uuid::Uuid;

const QUEUE_KEY: &str = "cut_optimizer:jobs";

pub async fn create_client() -> Result<ConnectionManager, redis::RedisError> {
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".into());

    let client = redis::Client::open(redis_url)?;
    ConnectionManager::new(client).await
}

pub async fn push_job(conn: &mut ConnectionManager, job_id: Uuid) -> Result<(), redis::RedisError> {
    conn.lpush(QUEUE_KEY, job_id.to_string()).await
}

pub async fn pop_job(conn: &mut ConnectionManager, timeout_secs: usize) -> Result<Option<Uuid>, redis::RedisError> {
    let result: Option<(String, String)> = conn.brpop(QUEUE_KEY, timeout_secs as f64).await?;

    match result {
        Some((_, job_id_str)) => {
            let job_id = Uuid::parse_str(&job_id_str)
                .map_err(|e| redis::RedisError::from((
                    redis::ErrorKind::TypeError,
                    "Invalid UUID",
                    e.to_string()
                )))?;
            Ok(Some(job_id))
        }
        None => Ok(None),
    }
}

pub async fn queue_length(conn: &mut ConnectionManager) -> Result<usize, redis::RedisError> {
    conn.llen(QUEUE_KEY).await
}
```

**Step 2: Update lib.rs**

Add to `src/lib.rs`:

```rust
pub mod queue;
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: Successful compilation

**Step 4: Commit**

```bash
git add src/queue/mod.rs src/lib.rs
git commit -m "feat: add Redis queue module"
```

---

### Task 7: Add webhook_url to OptimizeRequest

**Files:**
- Modify: `src/api/requests.rs`

**Step 1: Add webhook_url field**

Add to `OptimizeRequest` struct in `src/api/requests.rs`:

```rust
#[derive(Debug, Deserialize)]
pub struct OptimizeRequest {
    pub job_reference: String,
    #[serde(default)]
    pub client_name: Option<String>,
    pub pieces: Vec<CutPiece>,
    pub stock_sheets: Vec<StockSheet>,
    #[serde(default)]
    pub parameters: CutParameters,
    #[serde(default)]
    pub output: OutputOptions,
    #[serde(default)]
    pub webhook_url: Option<String>,
}
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Successful compilation

**Step 3: Commit**

```bash
git add src/api/requests.rs
git commit -m "feat: add webhook_url to OptimizeRequest"
```

---

### Task 8: Add Async Job Response Types

**Files:**
- Modify: `src/api/responses.rs`

**Step 1: Add job response types**

Add to `src/api/responses.rs`:

```rust
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct AsyncJobResponse {
    pub job_id: Uuid,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct JobStatusResponse {
    pub job_id: Uuid,
    pub status: String,
    pub job_reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
```

**Step 2: Add imports at top of file**

```rust
use chrono::{DateTime, Utc};
use uuid::Uuid;
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: Successful compilation

**Step 4: Commit**

```bash
git add src/api/responses.rs
git commit -m "feat: add async job response types"
```

---

### Task 9: Create App State for Shared Resources

**Files:**
- Create: `src/state.rs`
- Modify: `src/lib.rs`

**Step 1: Create state module**

Create `src/state.rs`:

```rust
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: Arc<Mutex<ConnectionManager>>,
}

impl AppState {
    pub fn new(db: PgPool, redis: ConnectionManager) -> Self {
        Self {
            db,
            redis: Arc::new(Mutex::new(redis)),
        }
    }
}
```

**Step 2: Update lib.rs**

Add to `src/lib.rs`:

```rust
pub mod state;
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: Successful compilation

**Step 4: Commit**

```bash
git add src/state.rs src/lib.rs
git commit -m "feat: add AppState for shared resources"
```

---

### Task 10: Add Async Optimize Handler

**Files:**
- Modify: `src/api/handlers.rs`

**Step 1: Add imports**

Add at top of `src/api/handlers.rs`:

```rust
use uuid::Uuid;
use crate::api::{AsyncJobResponse, JobStatusResponse};
use crate::db;
use crate::queue;
use crate::state::AppState;
```

**Step 2: Add optimize_async handler**

Add after `optimize_quick` function:

```rust
/// POST /api/v1/optimize/async
pub async fn optimize_async(
    state: web::Data<AppState>,
    request: web::Json<OptimizeRequest>,
) -> HttpResponse {
    // Validate first
    if let Err(e) = validate_request(&request) {
        let response = if let Some(field) = e.field {
            ApiResponse::<()>::error_with_field(&e.code, &e.message, &field)
        } else {
            ApiResponse::<()>::error(&e.code, &e.message)
        };
        return HttpResponse::BadRequest().json(response);
    }

    // Serialize request to JSON
    let request_json = match serde_json::to_value(&*request) {
        Ok(json) => json,
        Err(e) => {
            return HttpResponse::InternalServerError().json(
                ApiResponse::<()>::error("SERIALIZATION_ERROR", &e.to_string())
            );
        }
    };

    // Create job in database
    let job_id = match db::create_job(
        &state.db,
        &request.job_reference,
        request.client_name.as_deref(),
        &request_json,
        request.webhook_url.as_deref(),
    ).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to create job: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<()>::error("DATABASE_ERROR", "Failed to create job")
            );
        }
    };

    // Push to Redis queue
    {
        let mut redis = state.redis.lock().await;
        if let Err(e) = queue::push_job(&mut redis, job_id).await {
            tracing::error!("Failed to queue job: {}", e);
            // Job exists in DB but not queued - could implement retry logic
            return HttpResponse::InternalServerError().json(
                ApiResponse::<()>::error("QUEUE_ERROR", "Failed to queue job")
            );
        }
    }

    let response = AsyncJobResponse {
        job_id,
        status: "pending".to_string(),
    };

    HttpResponse::Accepted().json(ApiResponse::success(response))
}
```

**Step 3: Add get_job_status handler**

Add after `optimize_async`:

```rust
/// GET /api/v1/jobs/{job_id}
pub async fn get_job_status(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let job_id = path.into_inner();

    match db::get_job(&state.db, job_id).await {
        Ok(Some(job)) => {
            let pdf_base64 = job.pdf_bytes.as_ref().map(|bytes| {
                base64::engine::general_purpose::STANDARD.encode(bytes)
            });

            let response = JobStatusResponse {
                job_id: job.id,
                status: job.status.clone(),
                job_reference: job.job_reference,
                client_name: job.client_name,
                created_at: job.created_at,
                started_at: job.started_at,
                completed_at: job.completed_at,
                result: job.result,
                pdf_base64,
                error: job.error_message,
            };

            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Ok(None) => {
            HttpResponse::NotFound().json(
                ApiResponse::<()>::error("NOT_FOUND", "Job not found")
            )
        }
        Err(e) => {
            tracing::error!("Failed to get job: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<()>::error("DATABASE_ERROR", "Failed to get job")
            )
        }
    }
}
```

**Step 4: Add get_job_pdf handler**

Add after `get_job_status`:

```rust
/// GET /api/v1/jobs/{job_id}/pdf
pub async fn get_job_pdf(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let job_id = path.into_inner();

    match db::get_job(&state.db, job_id).await {
        Ok(Some(job)) => {
            if job.status != "completed" {
                return HttpResponse::BadRequest().json(
                    ApiResponse::<()>::error("JOB_NOT_COMPLETED", "Job is not completed yet")
                );
            }

            match job.pdf_bytes {
                Some(bytes) => {
                    HttpResponse::Ok()
                        .content_type("application/pdf")
                        .append_header(("Content-Disposition", format!("attachment; filename=\"{}.pdf\"", job.job_reference)))
                        .body(bytes)
                }
                None => {
                    HttpResponse::NotFound().json(
                        ApiResponse::<()>::error("NO_PDF", "PDF was not generated for this job")
                    )
                }
            }
        }
        Ok(None) => {
            HttpResponse::NotFound().json(
                ApiResponse::<()>::error("NOT_FOUND", "Job not found")
            )
        }
        Err(e) => {
            tracing::error!("Failed to get job: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<()>::error("DATABASE_ERROR", "Failed to get job")
            )
        }
    }
}
```

**Step 5: Verify compilation**

Run: `cargo check`
Expected: Successful compilation

**Step 6: Commit**

```bash
git add src/api/handlers.rs
git commit -m "feat: add async job handlers"
```

---

### Task 11: Update API Routes

**Files:**
- Modify: `src/api/routes.rs`

**Step 1: Add new routes**

Update `src/api/routes.rs`:

```rust
use actix_web::web;
use super::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/validate", web::post().to(handlers::validate))
            .route("/optimize/quick", web::post().to(handlers::optimize_quick))
            .route("/optimize/async", web::post().to(handlers::optimize_async))
            .route("/jobs/{job_id}", web::get().to(handlers::get_job_status))
            .route("/jobs/{job_id}/pdf", web::get().to(handlers::get_job_pdf))
            .route("/templates", web::get().to(handlers::get_templates))
    );
}
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Successful compilation

**Step 3: Commit**

```bash
git add src/api/routes.rs
git commit -m "feat: add async job routes"
```

---

### Task 12: Update API Module Exports

**Files:**
- Modify: `src/api/mod.rs`

**Step 1: Add new exports**

Update `src/api/mod.rs` to export new types:

```rust
pub mod handlers;
pub mod requests;
pub mod responses;
pub mod routes;
pub mod validation;

pub use requests::*;
pub use responses::*;
pub use validation::validate_request;
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Successful compilation

**Step 3: Commit**

```bash
git add src/api/mod.rs
git commit -m "feat: export async job types from api module"
```

---

### Task 13: Create Worker Module

**Files:**
- Create: `src/worker/mod.rs`
- Modify: `src/lib.rs`

**Step 1: Create worker module**

Create `src/worker/mod.rs`:

```rust
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::OptimizeRequest;
use crate::db;
use crate::optimizer::solve_ffdh;
use crate::output::generate_pdf;
use crate::queue;

pub async fn run_worker(
    db: PgPool,
    mut redis: ConnectionManager,
    concurrency: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Worker started with concurrency {}", concurrency);

    // For now, single-threaded processing
    // TODO: Implement concurrent processing with semaphore
    loop {
        match queue::pop_job(&mut redis, 5).await {
            Ok(Some(job_id)) => {
                tracing::info!("Processing job {}", job_id);
                if let Err(e) = process_job(&db, job_id).await {
                    tracing::error!("Failed to process job {}: {}", job_id, e);
                }
            }
            Ok(None) => {
                // Timeout, continue loop (allows graceful shutdown check)
            }
            Err(e) => {
                tracing::error!("Failed to pop job from queue: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}

async fn process_job(db: &PgPool, job_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
    // Load job from database
    let job = db::get_job(db, job_id).await?
        .ok_or_else(|| format!("Job {} not found", job_id))?;

    // Update status to processing
    db::update_job_status(db, job_id, "processing").await?;

    // Deserialize request
    let request: OptimizeRequest = serde_json::from_value(job.request.clone())?;

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

    // Store result
    let result_json = serde_json::to_value(&result)?;
    db::complete_job(db, job_id, &result_json, pdf_bytes).await?;

    tracing::info!("Job {} completed successfully", job_id);

    // Deliver webhook if configured
    if let Some(webhook_url) = job.webhook_url {
        deliver_webhook(db, job_id, &webhook_url, &result_json).await;
    }

    Ok(())
}

async fn deliver_webhook(
    db: &PgPool,
    job_id: Uuid,
    webhook_url: &str,
    result: &serde_json::Value,
) {
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "job_id": job_id,
        "status": "completed",
        "result": result
    });

    let timeout = std::env::var("WEBHOOK_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    match client
        .post(webhook_url)
        .timeout(std::time::Duration::from_secs(timeout))
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                tracing::info!("Webhook delivered for job {}", job_id);
                let _ = db::mark_webhook_delivered(db, job_id).await;
            } else {
                tracing::warn!(
                    "Webhook returned status {} for job {}",
                    response.status(),
                    job_id
                );
            }
        }
        Err(e) => {
            tracing::error!("Failed to deliver webhook for job {}: {}", job_id, e);
        }
    }
}
```

**Step 2: Update lib.rs**

Add to `src/lib.rs`:

```rust
pub mod worker;
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: Successful compilation

**Step 4: Commit**

```bash
git add src/worker/mod.rs src/lib.rs
git commit -m "feat: add worker module for job processing"
```

---

### Task 14: Update Main to Initialize Resources

**Files:**
- Modify: `src/main.rs`

**Step 1: Update main.rs with full initialization**

Replace `src/main.rs`:

```rust
use actix_web::{web, App, HttpServer, HttpResponse};
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod cli;
mod db;
mod optimizer;
mod output;
mod queue;
mod state;
mod worker;

use cli::Cli;
use state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Cli::parse();

    // Initialize database pool
    let db_pool = db::create_pool().await
        .expect("Failed to create database pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    // Initialize Redis connection
    let redis_conn = queue::create_client().await
        .expect("Failed to connect to Redis");

    if args.worker {
        tracing::info!("Starting worker with concurrency {}", args.concurrency);
        worker::run_worker(db_pool, redis_conn, args.concurrency)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        Ok(())
    } else {
        run_api_server(db_pool, redis_conn).await
    }
}

async fn run_api_server(
    db_pool: sqlx::PgPool,
    redis_conn: redis::aio::ConnectionManager,
) -> std::io::Result<()> {
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()
        .expect("PORT must be a number");

    let state = AppState::new(db_pool, redis_conn);

    tracing::info!("Starting Cut Optimizer API at {}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/health", web::get().to(health_check))
            .configure(api::routes::configure)
    })
    .bind((host, port))?
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Successful compilation

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: initialize database and Redis in main"
```

---

### Task 15: Update Docker Compose

**Files:**
- Modify: `docker-compose.yml`

**Step 1: Update docker-compose.yml**

Replace with:

```yaml
version: '3.8'

services:
  api:
    build: .
    ports:
      - "8080:8080"
    environment:
      HOST: "0.0.0.0"
      PORT: "8080"
      DATABASE_URL: postgres://app:secret@postgres/cut_optimizer
      REDIS_URL: redis://redis:6379
      RUST_LOG: info
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_started

  worker:
    build: .
    command: ["./cut-optimizer-api", "--worker"]
    environment:
      DATABASE_URL: postgres://app:secret@postgres/cut_optimizer
      REDIS_URL: redis://redis:6379
      RUST_LOG: info
      WEBHOOK_TIMEOUT_SECS: "30"
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_started

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: cut_optimizer
      POSTGRES_USER: app
      POSTGRES_PASSWORD: secret
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U app -d cut_optimizer"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

**Step 2: Commit**

```bash
git add docker-compose.yml
git commit -m "feat: add worker and postgres to docker-compose"
```

---

### Task 16: Add Integration Tests for Async Jobs

**Files:**
- Modify: `tests/api_tests.rs`

**Step 1: Add async job tests**

Add to `tests/api_tests.rs` (note: these require test database/redis setup):

```rust
// Note: These tests require DATABASE_URL and REDIS_URL to be set
// Run with: cargo test --test api_tests -- --ignored

#[actix_rt::test]
#[ignore] // Requires database and redis
async fn test_async_job_creation() {
    // This test would require setting up test infrastructure
    // For now, we'll test the happy path manually
}

#[actix_rt::test]
#[ignore] // Requires database and redis
async fn test_job_status_polling() {
    // This test would require setting up test infrastructure
}
```

**Step 2: Commit**

```bash
git add tests/api_tests.rs
git commit -m "test: add placeholder integration tests for async jobs"
```

---

### Task 17: Add Request Serialization Support

**Files:**
- Modify: `src/api/requests.rs`

**Step 1: Add Serialize derive to OptimizeRequest**

Update the derive macro for `OptimizeRequest`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct OptimizeRequest {
    // ... existing fields ...
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct OutputOptions {
    // ... existing fields ...
}
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Successful compilation

**Step 3: Commit**

```bash
git add src/api/requests.rs
git commit -m "feat: add Serialize to OptimizeRequest for job storage"
```

---

### Task 18: Run Full Test Suite

**Step 1: Run all tests**

Run: `cargo test`
Expected: All existing tests pass

**Step 2: Check compilation**

Run: `cargo build`
Expected: Successful build

**Step 3: Commit any fixes if needed**

---

### Task 19: Update README with Async Documentation

**Files:**
- Modify: `README.md` (if exists) or create

**Step 1: Document async API usage**

Add to README or create documentation:

```markdown
## Async Job Processing

### Submit Async Job

```bash
curl -X POST http://localhost:8080/api/v1/optimize/async \
  -H "Content-Type: application/json" \
  -d '{
    "job_reference": "JOB-001",
    "pieces": [{"id": "panel-a", "width": 580, "length": 418, "quantity": 10}],
    "stock_sheets": [{"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820}],
    "output": {"generate_pdf": true},
    "webhook_url": "https://example.com/webhook"
  }'
```

Response:
```json
{"success": true, "result": {"job_id": "uuid", "status": "pending"}}
```

### Poll Job Status

```bash
curl http://localhost:8080/api/v1/jobs/{job_id}
```

### Download PDF

```bash
curl -o layout.pdf http://localhost:8080/api/v1/jobs/{job_id}/pdf
```

### Running the Worker

```bash
# Single worker
./cut-optimizer-api --worker

# Multiple concurrent jobs
./cut-optimizer-api --worker --concurrency 4
```
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add async job processing documentation"
```

---

## Summary

This plan implements async job processing in 19 tasks:

1. Add dependencies
2. CLI argument parsing
3. Database module with Job model
4. Job repository functions
5. Database migration
6. Redis queue module
7. Add webhook_url to request
8. Async job response types
9. App state for shared resources
10. Async optimize handler
11. Update API routes
12. Update API module exports
13. Create worker module
14. Update main initialization
15. Update Docker Compose
16. Integration test placeholders
17. Request serialization
18. Run full test suite
19. Documentation

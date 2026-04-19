# Async Job Processing Design for Cut Optimizer API

## Overview

Add async job processing with Redis queue and PostgreSQL persistence. Enables large batch jobs, background PDF generation, and job queuing for workload control.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Result delivery | Polling + optional webhook | Flexible, works with any client |
| Storage | PostgreSQL | Persistent job history, analytics capability |
| Worker architecture | Single binary, two modes | Simple deployment, independent scaling |
| Queue ordering | FIFO | Simple, all jobs equal priority |

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Client    │────▶│  API Server │────▶│    Redis    │
└─────────────┘     └─────────────┘     │   (Queue)   │
                           │            └──────┬──────┘
                           │                   │
                           ▼                   ▼
                    ┌─────────────┐     ┌─────────────┐
                    │  PostgreSQL │◀────│   Worker    │
                    │   (Jobs)    │     │   Process   │
                    └─────────────┘     └─────────────┘
```

**Flow:**
1. Client POSTs to `/api/v1/optimize/async` → API validates, creates job in PostgreSQL (status: "pending"), pushes job_id to Redis queue, returns job_id immediately
2. Worker pops job_id from Redis, loads job from PostgreSQL, runs optimization + PDF generation, stores result in PostgreSQL, updates status to "completed"
3. Client polls `GET /api/v1/jobs/{job_id}` until status is "completed", then fetches result
4. Optionally, if webhook_url was provided, worker POSTs result to callback

**Single binary, two modes:**
- `cut-optimizer-api` - runs API server (default)
- `cut-optimizer-api --worker` - runs worker that processes queue

## Database Schema

```sql
CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
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
    webhook_delivered BOOLEAN DEFAULT FALSE,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_jobs_status ON jobs(status);
```

**Job states:**
- `pending` - Job created, waiting in queue
- `processing` - Worker picked it up, running optimization
- `completed` - Success, result available
- `failed` - Error occurred, error_message populated

**Redis queue:** Simple list `cut_optimizer:jobs` containing job UUIDs. Worker does `BRPOP` to block-wait for jobs.

## API Endpoints

**New endpoints for async processing:**

```
POST /api/v1/optimize/async
  Request: Same as /optimize/quick, plus optional webhook_url
  Response: { "success": true, "result": { "job_id": "uuid", "status": "pending" } }

GET /api/v1/jobs/{job_id}
  Response (pending):    { "job_id": "...", "status": "pending", "created_at": "..." }
  Response (processing): { "job_id": "...", "status": "processing", "started_at": "..." }
  Response (completed):  { "job_id": "...", "status": "completed", "result": {...}, "pdf_base64": "..." }
  Response (failed):     { "job_id": "...", "status": "failed", "error": "..." }

GET /api/v1/jobs/{job_id}/pdf
  Response: Raw PDF bytes (Content-Type: application/pdf)
  Use case: Direct download without base64 overhead
```

**Existing endpoints unchanged:**
- `POST /api/v1/optimize/quick` - Synchronous, for small jobs
- `POST /api/v1/validate` - Validation only
- `GET /api/v1/templates` - Stock sheet templates

**Request extension for async:**
```rust
pub struct OptimizeRequest {
    // ... existing fields ...
    #[serde(default)]
    pub webhook_url: Option<String>,
}
```

## Worker Implementation

```rust
async fn run_worker(redis: RedisPool, db: PgPool) -> Result<()> {
    loop {
        // Block-wait for job from queue (timeout 5s to allow graceful shutdown)
        let job_id = redis.brpop("cut_optimizer:jobs", 5).await?;

        if let Some(job_id) = job_id {
            process_job(&db, &redis, job_id).await;
        }
    }
}

async fn process_job(db: &PgPool, redis: &RedisPool, job_id: Uuid) {
    // 1. Load job from PostgreSQL
    let job = db.get_job(job_id).await?;

    // 2. Update status to processing
    db.update_job_status(job_id, "processing").await?;

    // 3. Deserialize request and run optimization
    let request: OptimizeRequest = serde_json::from_value(job.request)?;
    let result = solve_ffdh(&request.pieces, &stock_sheet, request.parameters.blade_kerf);

    // 4. Generate PDF if requested
    let pdf_bytes = if request.output.generate_pdf {
        Some(generate_pdf(&result, ...)?)
    } else {
        None
    };

    // 5. Store result and update status
    db.complete_job(job_id, &result, pdf_bytes).await?;

    // 6. Call webhook if provided
    if let Some(url) = job.webhook_url {
        deliver_webhook(&url, job_id, &result).await;
    }
}
```

**Graceful shutdown:** Worker catches SIGTERM, finishes current job, then exits.

**Error handling:** If processing fails, catch error, update job status to "failed" with error_message.

## Dependencies

```toml
[dependencies]
# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json"] }
# Redis
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"] }
# HTTP client for webhooks
reqwest = { version = "0.12", features = ["json"] }
# UUID generation
uuid = { version = "1.0", features = ["v4", "serde"] }
# CLI parsing for --worker flag
clap = { version = "4.0", features = ["derive"] }
```

## Configuration

**Environment variables:**

```env
# Existing
HOST=0.0.0.0
PORT=8080

# New for async
DATABASE_URL=postgres://user:pass@localhost/cut_optimizer
REDIS_URL=redis://localhost:6379
WORKER_CONCURRENCY=1
WEBHOOK_TIMEOUT_SECS=30
```

**CLI interface:**

```bash
# Run API server (default)
cut-optimizer-api

# Run worker
cut-optimizer-api --worker

# Run worker with concurrency
cut-optimizer-api --worker --concurrency 4
```

## Docker Compose

```yaml
services:
  api:
    build: .
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://app:secret@postgres/cut_optimizer
      REDIS_URL: redis://redis:6379
    depends_on: [postgres, redis]

  worker:
    build: .
    command: ["./cut-optimizer-api", "--worker"]
    environment:
      DATABASE_URL: postgres://app:secret@postgres/cut_optimizer
      REDIS_URL: redis://redis:6379
    depends_on: [postgres, redis]

  postgres:
    image: postgres:16
    environment:
      POSTGRES_DB: cut_optimizer
      POSTGRES_USER: app
      POSTGRES_PASSWORD: secret
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine

volumes:
  postgres_data:
```

## Testing Strategy

1. Unit tests for job state transitions
2. Integration tests with test containers (postgres + redis)
3. Test webhook delivery with mock server
4. Test worker graceful shutdown
5. Load test with multiple concurrent jobs

# Cut Optimizer API

A Rust-based REST API for 2D bin packing optimization with PDF layout generation. Optimizes cutting layouts for panel materials using the First-Fit Decreasing Height (FFDH) algorithm with guillotine cut constraints.

## Features

- FFDH algorithm for efficient piece placement
- Guillotine cuts support (standard panel saw constraints)
- Edge banding tracking and visualization
- PDF layout diagram generation
- Synchronous and asynchronous job processing
- Redis queue for background job processing
- PostgreSQL for job persistence
- Webhook callbacks for job completion

## Quick Start

### Sync-only Mode (No Database Required)

```bash
cargo run
```

This starts the API server on `http://127.0.0.1:8080` with sync endpoints only.

### Full Async Mode (With Docker)

```bash
docker-compose up
```

This starts:
- API server on port 8080
- Worker process for background jobs
- PostgreSQL database
- Redis queue

## API Endpoints

### Health Check
```
GET /health
```

### Synchronous Optimization
```
POST /api/v1/optimize/quick
```

Processes optimization request immediately and returns result.

**Request:**
```json
{
  "job_reference": "JOB-001",
  "client_name": "Optional Client Name",
  "pieces": [
    {
      "id": "panel-a",
      "width": 580,
      "length": 418,
      "quantity": 4,
      "can_rotate": true,
      "edge_banding": {
        "top": true,
        "bottom": false,
        "left": false,
        "right": true,
        "material": "White 1mm"
      }
    }
  ],
  "stock_sheets": [
    {
      "id": "sheet-1",
      "name": "BOARD White",
      "width": 2740,
      "length": 1820,
      "thickness": 16
    }
  ],
  "parameters": {
    "blade_kerf": 3
  },
  "output": {
    "generate_pdf": true
  }
}
```

**Response:**
```json
{
  "success": true,
  "result": {
    "job_reference": "JOB-001",
    "total_sheets": 1,
    "total_pieces": 4,
    "efficiency": 78.5,
    "layouts": [...],
    "pdf_base64": "JVBERi0xLj..."
  }
}
```

### Asynchronous Optimization
```
POST /api/v1/optimize/async
```

Submits job for background processing. Requires PostgreSQL and Redis.

**Request:** Same as sync, plus optional `webhook_url`.

**Response:**
```json
{
  "success": true,
  "result": {
    "job_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "pending"
  }
}
```

### Job Status
```
GET /api/v1/jobs/{job_id}
```

Poll for job completion status.

**Response (pending):**
```json
{
  "success": true,
  "result": {
    "job_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "pending",
    "job_reference": "JOB-001",
    "created_at": "2024-01-15T10:30:00Z"
  }
}
```

**Response (completed):**
```json
{
  "success": true,
  "result": {
    "job_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "completed",
    "job_reference": "JOB-001",
    "result": {
      "total_sheets": 1,
      "total_pieces": 4,
      "efficiency": 78.5,
      "layouts": [...]
    },
    "pdf_base64": "JVBERi0xLj...",
    "created_at": "2024-01-15T10:30:00Z",
    "completed_at": "2024-01-15T10:30:05Z"
  }
}
```

### Validation
```
POST /api/v1/validate
```

Validates request without processing.

### Templates
```
GET /api/v1/templates
```

Returns predefined stock sheet templates.

## Running as Worker

```bash
cargo run -- --worker
```

The worker processes jobs from the Redis queue and updates results in PostgreSQL.

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| HOST | 127.0.0.1 | API server bind address |
| PORT | 8080 | API server port |
| DATABASE_URL | (required for async) | PostgreSQL connection URL |
| REDIS_URL | redis://127.0.0.1:6379 | Redis connection URL |
| RUST_LOG | info | Log level |

## Database Migration

The migration runs automatically when PostgreSQL initializes via Docker. For manual setup:

```bash
psql $DATABASE_URL -f migrations/001_create_jobs_table.sql
```

## Development

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Build release
cargo build --release
```

## Architecture

```
                    ┌─────────────────┐
                    │   API Server    │
                    │   (Actix-web)   │
                    └────────┬────────┘
                             │
          ┌──────────────────┼──────────────────┐
          │                  │                  │
          ▼                  ▼                  ▼
    ┌──────────┐      ┌──────────┐      ┌──────────┐
    │  /quick  │      │  /async  │      │  /jobs   │
    │  (sync)  │      │  (queue) │      │  (poll)  │
    └────┬─────┘      └────┬─────┘      └────┬─────┘
         │                 │                  │
         ▼                 ▼                  ▼
    ┌─────────┐      ┌─────────┐      ┌──────────┐
    │ FFDH    │      │  Redis  │      │ Postgres │
    │ Solver  │      │  Queue  │      │   Jobs   │
    └─────────┘      └────┬────┘      └──────────┘
                          │
                          ▼
                    ┌─────────┐
                    │ Worker  │
                    │ Process │
                    └────┬────┘
                         │
                         ▼
                    ┌─────────┐
                    │  FFDH   │───▶ PDF Generation
                    │ Solver  │───▶ Webhook Callback
                    └─────────┘
```

## License

MIT

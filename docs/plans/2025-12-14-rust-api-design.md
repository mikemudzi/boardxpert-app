# Cut Optimizer API - Rust Architecture Design

## Overview

A high-performance REST API for 2D cutting stock optimization, designed for carpenters and manufacturing environments. Optimizes how rectangular pieces are cut from sheet materials (melamine, MDF, plywood) while minimizing waste.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust | Performance for NP-hard optimization, memory safety |
| Web Framework | Actix-web | Fast async HTTP, production-proven |
| Algorithm | FFDH + Guillotine | Simple to implement, 80-90% optimal, extensible |
| Job Handling | Sync/Async split | Quick response for small jobs, queue for large |
| PDF Generation | printpdf | Precise control for layout diagrams |
| Deployment | Docker containers | Scalable, predictable performance |

## System Architecture

```
                                    +------------------+
                                    |  Load Balancer   |
                                    +--------+---------+
                                             |
              +------------------------------+------------------------------+
              |                              |                              |
     +--------v--------+           +---------v--------+          +---------v--------+
     |   API Server    |           |   API Server     |          |   API Server     |
     |   (Actix-web)   |           |   (Actix-web)    |          |   (Actix-web)    |
     +--------+--------+           +---------+--------+          +---------+--------+
              |                              |                              |
              +------------------------------+------------------------------+
                                             |
                        +--------------------+--------------------+
                        |                    |                    |
               +--------v--------+  +--------v--------+  +--------v-------+
               |   Redis         |  |   PostgreSQL    |  |  Object Store  |
               |   (Jobs/Cache)  |  |   (Results)     |  |  (PDFs/SVGs)   |
               +-----------------+  +-----------------+  +----------------+
```

### Components

- **API Servers (Actix-web)**: Stateless HTTP handlers, horizontally scalable
- **Redis**: Job queue for async processing, result caching
- **PostgreSQL**: Persistent job history and results
- **Object Store (S3/MinIO)**: Generated PDF and SVG files

## Request Flow

### Quick Jobs (< 50 pieces)
```
Client -> POST /api/v1/optimize/quick -> Process inline -> Return result
```

### Large Jobs (50+ pieces)
```
Client -> POST /api/v1/optimize -> Returns job_id immediately
Client -> GET /api/v1/jobs/{id} -> Poll for status/progress
Client -> GET /api/v1/jobs/{id}/pdf -> Fetch completed output
```

### Job Lifecycle

```
+----------+    +-----------+    +------------+    +-----------+
| PENDING  |--->| PROCESSING|--->| GENERATING |--->| COMPLETED |
+----------+    +-----------+    +------------+    +-----------+
     |               |                                   |
     |               v                                   |
     |          +---------+                              |
     +--------->| FAILED  |<-----------------------------+
                +---------+
```

## Project Structure

```
cut-optimizer-api/
├── Cargo.toml
├── src/
│   ├── main.rs                 # Entry point, server startup
│   ├── lib.rs                  # Library exports for testing
│   │
│   ├── api/
│   │   ├── mod.rs
│   │   ├── routes.rs           # Route definitions
│   │   ├── handlers.rs         # Request handlers
│   │   ├── requests.rs         # Input validation types
│   │   └── responses.rs        # Output types
│   │
│   ├── optimizer/
│   │   ├── mod.rs
│   │   ├── types.rs            # Piece, Sheet, Layout structs
│   │   ├── guillotine.rs       # Guillotine cut logic
│   │   ├── ffdh.rs             # First Fit Decreasing Height
│   │   └── solver.rs           # Main solver orchestration
│   │
│   ├── jobs/
│   │   ├── mod.rs
│   │   ├── queue.rs            # Redis job queue
│   │   ├── worker.rs           # Background job processor
│   │   └── progress.rs         # Progress tracking
│   │
│   ├── output/
│   │   ├── mod.rs
│   │   ├── pdf.rs              # PDF generation
│   │   └── svg.rs              # SVG generation
│   │
│   └── storage/
│       ├── mod.rs
│       ├── postgres.rs         # Job persistence
│       └── s3.rs               # File storage
│
├── tests/                      # Integration tests
└── benches/                    # Performance benchmarks
```

## Core Data Types

### Input Types

```rust
pub struct OptimizeRequest {
    pub job_reference: String,
    pub client_name: Option<String>,
    pub pieces: Vec<CutPiece>,
    pub stock_sheets: Vec<StockSheet>,
    pub parameters: CutParameters,
    pub output: OutputOptions,
}

pub struct CutPiece {
    pub id: String,
    pub width: u32,              // millimeters
    pub length: u32,
    pub quantity: u32,
    pub label: Option<String>,
    pub can_rotate: bool,
    pub edge_banding: Option<EdgeBanding>,
}

pub struct StockSheet {
    pub id: String,
    pub name: String,            // "BOARD White"
    pub width: u32,
    pub length: u32,
    pub thickness: u32,
    pub quantity: Option<u32>,   // None = unlimited
    pub cost: Option<f64>,
}

pub struct CutParameters {
    pub blade_kerf: f64,         // saw blade width (typically 3-4mm)
    pub edge_margin: u32,        // unusable edge of sheet
    pub guillotine_cuts: bool,   // enforce guillotine constraint
    pub priority: Priority,      // MinimizeWaste or MinimizeSheets
}

pub enum Priority {
    MinimizeWaste,
    MinimizeSheets,
}

pub struct EdgeBanding {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
    pub material: String,        // "White 1mm", "Oak 2mm"
}
```

### Output Types

```rust
pub struct PlacedPiece {
    pub piece_id: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub length: u32,
    pub rotated: bool,
}

pub struct SheetLayout {
    pub sheet_id: String,
    pub pieces: Vec<PlacedPiece>,
    pub waste_percentage: f64,
}

pub struct OptimizeResult {
    pub layouts: Vec<SheetLayout>,
    pub total_sheets: u32,
    pub total_waste_percentage: f64,
}
```

## Algorithm: FFDH with Guillotine Constraints

### Step 1: Sort pieces by height descending
```
Input:  [300x200, 500x400, 200x150, 500x300, 400x400]
Sorted: [500x400, 500x300, 400x400, 300x200, 200x150]
```

### Step 2: Shelf packing

```
Sheet (2740 x 1820):
+----------------------------------------------------------+
| +---------+---------+---------+                          |
| | 500x400 | 500x300 | 400x400 |   <-- Shelf 1 (h=500)    |
| |         |         |         |                          |
| +---------+----+----+---------+                          |
| |   300x200    |   200x150    |   <-- Shelf 2 (h=300)    |
| +--------------+--------------+                          |
| |          Remaining          |                          |
+----------------------------------------------------------+
```

### Step 3: Guillotine constraint

Each shelf is a horizontal cut across the full sheet. Pieces within a shelf are separated by vertical cuts. All cuts are edge-to-edge.

### Step 4: Blade kerf

Add `blade_kerf` between pieces:
```
Actual space = piece_width + blade_kerf
```

### Pseudocode

```
for each piece (sorted by height desc):
    for each existing shelf:
        if piece fits in shelf's remaining width:
            place piece in shelf
            break

    if piece not placed:
        if sheet has room for new shelf:
            create shelf with height = piece.height
            place piece
        else:
            start new sheet
```

## Edge Banding

When a piece is rotated 90 degrees, edge banding rotates too:

```rust
impl PlacedPiece {
    pub fn effective_edge_banding(&self, original: &EdgeBanding) -> EdgeBanding {
        if self.rotated {
            EdgeBanding {
                top: original.left,
                right: original.top,
                bottom: original.right,
                left: original.bottom,
                material: original.material.clone(),
            }
        } else {
            original.clone()
        }
    }
}
```

PDF output draws dashed lines on edges requiring banding.

## API Endpoints

### Health & Info
```
GET  /health                     -> Server status, version, uptime
```

### Templates
```
GET  /api/v1/templates           -> Predefined stock sheets
```

### Synchronous (small jobs)
```
POST /api/v1/validate            -> Validate request without optimizing
POST /api/v1/optimize/quick      -> Run optimization, wait for result
```

### Asynchronous (large jobs)
```
POST /api/v1/optimize            -> Submit job, returns { job_id }
GET  /api/v1/jobs/:id            -> Job status and progress
GET  /api/v1/jobs/:id/result     -> Optimization result (JSON)
GET  /api/v1/jobs/:id/pdf        -> Download PDF
GET  /api/v1/jobs/:id/svg        -> Download SVG
DELETE /api/v1/jobs/:id          -> Cancel pending job
```

### Job History
```
GET  /api/v1/jobs                -> List recent jobs (paginated)
```

### Sync/Async Threshold

```rust
const SYNC_PIECE_LIMIT: usize = 50;
```

## Error Handling

### Response Format

```json
{
    "success": false,
    "error": {
        "code": "PIECE_TOO_LARGE",
        "message": "Piece 'panel-a' (2800x600mm) exceeds stock sheet (2740x1820mm)",
        "field": "pieces[0]"
    }
}
```

### Error Types

```rust
pub enum ApiError {
    // Validation errors (400)
    InvalidRequest { message: String, field: Option<String> },
    PieceTooLarge { piece_id: String, piece_size: (u32, u32), max_size: (u32, u32) },
    NoStockSheets,
    NoPieces,

    // Job errors (404, 409)
    JobNotFound { job_id: String },
    JobNotReady { job_id: String, status: JobStatus },
    JobAlreadyCancelled { job_id: String },

    // Processing errors (422)
    OptimizationFailed { reason: String },
    PdfGenerationFailed { reason: String },

    // System errors (500, 503)
    DatabaseError,
    QueueError,
    StorageError,
    ServiceOverloaded,
}
```

### Validation Checks

1. At least one piece and one stock sheet
2. All pieces fit in at least one stock sheet
3. Quantities are positive integers
4. Blade kerf is reasonable (0-10mm)
5. Dimensions are positive

## Configuration

### Environment Variables

```bash
# Server
HOST=0.0.0.0
PORT=8080
WORKERS=4
REQUEST_TIMEOUT_SECS=30

# Redis
REDIS_URL=redis://localhost:6379
JOB_QUEUE_NAME=cut_optimizer_jobs
CACHE_TTL_SECS=3600

# PostgreSQL
DATABASE_URL=postgres://user:pass@localhost/cut_optimizer
MAX_DB_CONNECTIONS=20

# Object Storage
S3_ENDPOINT=http://localhost:9000
S3_BUCKET=cut-optimizer-outputs
S3_ACCESS_KEY=minioadmin
S3_SECRET_KEY=minioadmin

# Optimization
MAX_PIECES_SYNC=50
OPTIMIZATION_TIMEOUT_SECS=120
MAX_SHEETS_PER_JOB=100

# Logging
RUST_LOG=info,cut_optimizer=debug
```

### Config Loading

```rust
#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub redis_url: String,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Config {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()?,
            // ...
        })
    }
}
```

## PDF Output Format

Each page contains:
- Material name and dimensions
- Job metadata (reference, client, date)
- Layout number and sheet count
- Cutting list table (Id, Length, Width, Qty)
- Edge banding legend
- Scaled layout diagram with piece labels and dimensions

## Future Enhancements

1. **Algorithm improvements**: Implement Mellouli & Dammak pattern generation for better optimization on large jobs
2. **Grain direction**: Support for wood grain alignment constraints
3. **Multiple materials per job**: Already supported in data model
4. **Cost optimization**: Use sheet costs in Priority::MinimizeCost mode
5. **WebAssembly**: Compile optimizer to WASM for client-side preview

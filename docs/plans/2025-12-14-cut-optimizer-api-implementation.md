# Cut Optimizer API - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust REST API that optimizes 2D cutting layouts for sheet materials, returning JSON results and PDF diagrams.

**Architecture:** Actix-web handles HTTP requests. The FFDH algorithm places pieces onto sheets using shelf-packing with guillotine constraints. Small jobs run synchronously; large jobs queue to Redis for async processing.

**Tech Stack:** Rust, Actix-web, Serde, printpdf, Redis, PostgreSQL, S3

---

## Phase 1: Project Setup & Core Types

### Task 1: Initialize Rust Project

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "cut-optimizer-api"
version = "0.1.0"
edition = "2021"
description = "2D cutting stock optimization API"

[dependencies]
actix-web = "4"
actix-rt = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
actix-rt = "2"
```

**Step 2: Create src/lib.rs**

```rust
pub mod optimizer;
pub mod api;
```

**Step 3: Create src/main.rs**

```rust
use actix_web::{web, App, HttpServer, HttpResponse};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()
        .expect("PORT must be a number");

    tracing::info!("Starting server at {}:{}", host, port);

    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health_check))
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

**Step 4: Build and verify**

Run: `cargo build`
Expected: Compiles without errors

**Step 5: Test health endpoint**

Run: `cargo run &` then `curl http://localhost:8080/health`
Expected: `{"status":"healthy","version":"0.1.0"}`

**Step 6: Commit**

```bash
git add Cargo.toml src/
git commit -m "feat: initialize Rust project with health endpoint"
```

---

### Task 2: Define Core Optimizer Types

**Files:**
- Create: `src/optimizer/mod.rs`
- Create: `src/optimizer/types.rs`

**Step 1: Create src/optimizer/mod.rs**

```rust
pub mod types;

pub use types::*;
```

**Step 2: Create src/optimizer/types.rs**

```rust
use serde::{Deserialize, Serialize};

/// A rectangular piece to be cut from stock sheets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutPiece {
    pub id: String,
    pub width: u32,
    pub length: u32,
    pub quantity: u32,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default = "default_true")]
    pub can_rotate: bool,
    #[serde(default)]
    pub edge_banding: Option<EdgeBanding>,
}

fn default_true() -> bool {
    true
}

/// Edge banding specification for a piece
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeBanding {
    #[serde(default)]
    pub top: bool,
    #[serde(default)]
    pub bottom: bool,
    #[serde(default)]
    pub left: bool,
    #[serde(default)]
    pub right: bool,
    #[serde(default)]
    pub material: String,
}

impl EdgeBanding {
    /// Rotate edge banding 90 degrees clockwise
    pub fn rotated(&self) -> Self {
        EdgeBanding {
            top: self.left,
            right: self.top,
            bottom: self.right,
            left: self.bottom,
            material: self.material.clone(),
        }
    }
}

/// A stock sheet available for cutting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockSheet {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub length: u32,
    #[serde(default)]
    pub thickness: u32,
    #[serde(default)]
    pub quantity: Option<u32>,
    #[serde(default)]
    pub cost: Option<f64>,
}

/// Parameters controlling the optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutParameters {
    #[serde(default = "default_blade_kerf")]
    pub blade_kerf: f64,
    #[serde(default)]
    pub edge_margin: u32,
    #[serde(default = "default_true")]
    pub guillotine_cuts: bool,
    #[serde(default)]
    pub priority: Priority,
}

fn default_blade_kerf() -> f64 {
    4.0
}

impl Default for CutParameters {
    fn default() -> Self {
        CutParameters {
            blade_kerf: 4.0,
            edge_margin: 0,
            guillotine_cuts: true,
            priority: Priority::MinimizeWaste,
        }
    }
}

/// Optimization priority
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    #[default]
    MinimizeWaste,
    MinimizeSheets,
}

/// A piece placed on a sheet at specific coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedPiece {
    pub piece_id: String,
    pub label: Option<String>,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub length: u32,
    pub rotated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_banding: Option<EdgeBanding>,
}

/// A single sheet with pieces placed on it
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetLayout {
    pub sheet_index: usize,
    pub stock_sheet_id: String,
    pub stock_sheet_name: String,
    pub width: u32,
    pub length: u32,
    pub pieces: Vec<PlacedPiece>,
    pub used_area: u64,
    pub waste_percentage: f64,
}

/// Complete optimization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizeResult {
    pub layouts: Vec<SheetLayout>,
    pub total_sheets: u32,
    pub total_pieces: u32,
    pub total_area: u64,
    pub used_area: u64,
    pub waste_area: u64,
    pub waste_percentage: f64,
}
```

**Step 3: Update src/lib.rs**

```rust
pub mod optimizer;
pub mod api;
```

**Step 4: Verify it compiles**

Run: `cargo build`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add src/optimizer/
git commit -m "feat: add core optimizer types"
```

---

### Task 3: Add Unit Tests for EdgeBanding Rotation

**Files:**
- Modify: `src/optimizer/types.rs`

**Step 1: Add test module to types.rs**

Add at the bottom of `src/optimizer/types.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_banding_rotation() {
        let banding = EdgeBanding {
            top: true,
            right: false,
            bottom: true,
            left: false,
            material: "White 1mm".to_string(),
        };

        let rotated = banding.rotated();

        // After 90 degree clockwise rotation:
        // - top becomes right
        // - right becomes bottom
        // - bottom becomes left
        // - left becomes top
        assert!(!rotated.top);    // was left (false)
        assert!(rotated.right);   // was top (true)
        assert!(!rotated.bottom); // was right (false)
        assert!(rotated.left);    // was bottom (true)
        assert_eq!(rotated.material, "White 1mm");
    }

    #[test]
    fn test_edge_banding_double_rotation() {
        let banding = EdgeBanding {
            top: true,
            right: false,
            bottom: false,
            left: true,
            material: "Oak 2mm".to_string(),
        };

        let rotated_twice = banding.rotated().rotated();

        // 180 degree rotation: top<->bottom, left<->right
        assert!(!rotated_twice.top);
        assert!(rotated_twice.right);
        assert!(rotated_twice.bottom);
        assert!(!rotated_twice.left);
    }
}
```

**Step 2: Run tests**

Run: `cargo test`
Expected: 2 tests pass

**Step 3: Commit**

```bash
git add src/optimizer/types.rs
git commit -m "test: add edge banding rotation tests"
```

---

## Phase 2: FFDH Algorithm Implementation

### Task 4: Create Shelf Data Structure

**Files:**
- Create: `src/optimizer/ffdh.rs`
- Modify: `src/optimizer/mod.rs`

**Step 1: Create src/optimizer/ffdh.rs with Shelf struct**

```rust
use super::types::*;

/// A horizontal shelf within a sheet
#[derive(Debug, Clone)]
pub struct Shelf {
    /// Y position of shelf bottom edge
    pub y: u32,
    /// Height of this shelf (determined by first piece placed)
    pub height: u32,
    /// Remaining width available for pieces
    pub remaining_width: u32,
    /// Pieces placed on this shelf
    pub pieces: Vec<PlacedPiece>,
    /// Current X position for next piece
    pub current_x: u32,
}

impl Shelf {
    pub fn new(y: u32, height: u32, sheet_width: u32) -> Self {
        Shelf {
            y,
            height,
            remaining_width: sheet_width,
            pieces: Vec::new(),
            current_x: 0,
        }
    }

    /// Try to place a piece on this shelf
    /// Returns true if piece was placed, false if it doesn't fit
    pub fn try_place(
        &mut self,
        piece_id: &str,
        label: Option<String>,
        width: u32,
        length: u32,
        rotated: bool,
        edge_banding: Option<EdgeBanding>,
        blade_kerf: u32,
    ) -> bool {
        // Check if piece fits in remaining width
        let space_needed = if self.pieces.is_empty() {
            width
        } else {
            width + blade_kerf
        };

        if space_needed > self.remaining_width {
            return false;
        }

        // Check if piece height fits shelf height
        if length > self.height {
            return false;
        }

        // Calculate x position (add kerf if not first piece)
        let x = if self.pieces.is_empty() {
            0
        } else {
            self.current_x + blade_kerf
        };

        self.pieces.push(PlacedPiece {
            piece_id: piece_id.to_string(),
            label,
            x,
            y: self.y,
            width,
            length,
            rotated,
            edge_banding,
        });

        self.current_x = x + width;
        self.remaining_width = self.remaining_width.saturating_sub(space_needed);

        true
    }

    /// Calculate total used area on this shelf
    pub fn used_area(&self) -> u64 {
        self.pieces.iter()
            .map(|p| p.width as u64 * p.length as u64)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shelf_creation() {
        let shelf = Shelf::new(0, 500, 2740);
        assert_eq!(shelf.y, 0);
        assert_eq!(shelf.height, 500);
        assert_eq!(shelf.remaining_width, 2740);
        assert!(shelf.pieces.is_empty());
    }

    #[test]
    fn test_shelf_place_single_piece() {
        let mut shelf = Shelf::new(0, 500, 2740);

        let placed = shelf.try_place(
            "piece-1",
            Some("Panel A".to_string()),
            580,
            418,
            false,
            None,
            4,
        );

        assert!(placed);
        assert_eq!(shelf.pieces.len(), 1);
        assert_eq!(shelf.pieces[0].x, 0);
        assert_eq!(shelf.pieces[0].y, 0);
        assert_eq!(shelf.remaining_width, 2740 - 580);
    }

    #[test]
    fn test_shelf_place_multiple_pieces_with_kerf() {
        let mut shelf = Shelf::new(0, 500, 2740);

        // First piece at x=0
        shelf.try_place("piece-1", None, 580, 418, false, None, 4);

        // Second piece should be at x = 580 + 4 (kerf)
        shelf.try_place("piece-2", None, 580, 418, false, None, 4);

        assert_eq!(shelf.pieces.len(), 2);
        assert_eq!(shelf.pieces[0].x, 0);
        assert_eq!(shelf.pieces[1].x, 584); // 580 + 4
    }

    #[test]
    fn test_shelf_reject_piece_too_wide() {
        let mut shelf = Shelf::new(0, 500, 1000);

        let placed = shelf.try_place("piece-1", None, 1200, 400, false, None, 4);

        assert!(!placed);
        assert!(shelf.pieces.is_empty());
    }

    #[test]
    fn test_shelf_reject_piece_too_tall() {
        let mut shelf = Shelf::new(0, 400, 2740);

        // Piece length (500) > shelf height (400)
        let placed = shelf.try_place("piece-1", None, 580, 500, false, None, 4);

        assert!(!placed);
    }
}
```

**Step 2: Update src/optimizer/mod.rs**

```rust
pub mod types;
pub mod ffdh;

pub use types::*;
pub use ffdh::*;
```

**Step 3: Run tests**

Run: `cargo test shelf`
Expected: All shelf tests pass

**Step 4: Commit**

```bash
git add src/optimizer/
git commit -m "feat: add Shelf data structure for FFDH"
```

---

### Task 5: Implement FFDH Solver

**Files:**
- Modify: `src/optimizer/ffdh.rs`

**Step 1: Add ExpandedPiece and SheetState structs**

Add after the `Shelf` impl block in `src/optimizer/ffdh.rs`:

```rust
/// A piece expanded by quantity for sorting
#[derive(Debug, Clone)]
struct ExpandedPiece {
    pub original_id: String,
    pub instance_index: u32,
    pub label: Option<String>,
    pub width: u32,
    pub length: u32,
    pub can_rotate: bool,
    pub edge_banding: Option<EdgeBanding>,
}

/// State of a single sheet during packing
#[derive(Debug)]
struct SheetState {
    pub stock_sheet: StockSheet,
    pub shelves: Vec<Shelf>,
    pub current_y: u32,
    pub blade_kerf: u32,
}

impl SheetState {
    fn new(stock_sheet: StockSheet, blade_kerf: u32) -> Self {
        SheetState {
            stock_sheet,
            shelves: Vec::new(),
            current_y: 0,
            blade_kerf,
        }
    }

    fn remaining_height(&self) -> u32 {
        self.stock_sheet.length.saturating_sub(self.current_y)
    }

    fn try_place_piece(&mut self, piece: &ExpandedPiece) -> bool {
        let (width, length, rotated) = self.determine_orientation(piece);

        // Try existing shelves first
        for shelf in &mut self.shelves {
            if shelf.try_place(
                &format!("{}-{}", piece.original_id, piece.instance_index),
                piece.label.clone(),
                width,
                length,
                rotated,
                piece.edge_banding.clone().map(|eb| if rotated { eb.rotated() } else { eb }),
                self.blade_kerf,
            ) {
                return true;
            }
        }

        // Try creating new shelf
        let shelf_height_needed = if self.shelves.is_empty() {
            length
        } else {
            length + self.blade_kerf
        };

        if shelf_height_needed <= self.remaining_height() {
            let shelf_y = if self.shelves.is_empty() {
                0
            } else {
                self.current_y + self.blade_kerf
            };

            let mut new_shelf = Shelf::new(shelf_y, length, self.stock_sheet.width);
            new_shelf.try_place(
                &format!("{}-{}", piece.original_id, piece.instance_index),
                piece.label.clone(),
                width,
                length,
                rotated,
                piece.edge_banding.clone().map(|eb| if rotated { eb.rotated() } else { eb }),
                self.blade_kerf,
            );

            self.current_y = shelf_y + length;
            self.shelves.push(new_shelf);
            return true;
        }

        false
    }

    fn determine_orientation(&self, piece: &ExpandedPiece) -> (u32, u32, bool) {
        // Try original orientation first (length as height)
        if piece.length <= self.remaining_height() && piece.width <= self.stock_sheet.width {
            return (piece.width, piece.length, false);
        }

        // Try rotated if allowed
        if piece.can_rotate {
            if piece.width <= self.remaining_height() && piece.length <= self.stock_sheet.width {
                return (piece.length, piece.width, true);
            }
        }

        // Return original even if it doesn't fit (caller handles failure)
        (piece.width, piece.length, false)
    }

    fn to_layout(&self, sheet_index: usize) -> SheetLayout {
        let pieces: Vec<PlacedPiece> = self.shelves
            .iter()
            .flat_map(|s| s.pieces.clone())
            .collect();

        let sheet_area = self.stock_sheet.width as u64 * self.stock_sheet.length as u64;
        let used_area: u64 = pieces.iter()
            .map(|p| p.width as u64 * p.length as u64)
            .sum();
        let waste_percentage = if sheet_area > 0 {
            ((sheet_area - used_area) as f64 / sheet_area as f64) * 100.0
        } else {
            0.0
        };

        SheetLayout {
            sheet_index,
            stock_sheet_id: self.stock_sheet.id.clone(),
            stock_sheet_name: self.stock_sheet.name.clone(),
            width: self.stock_sheet.width,
            length: self.stock_sheet.length,
            pieces,
            used_area,
            waste_percentage,
        }
    }
}
```

**Step 2: Add the main solve function**

Add after `SheetState` impl:

```rust
/// FFDH (First Fit Decreasing Height) solver
pub fn solve_ffdh(
    pieces: &[CutPiece],
    stock_sheet: &StockSheet,
    blade_kerf: f64,
) -> OptimizeResult {
    let blade_kerf_u32 = blade_kerf.round() as u32;

    // Expand pieces by quantity and sort by length (height) descending
    let mut expanded: Vec<ExpandedPiece> = pieces
        .iter()
        .flat_map(|p| {
            (0..p.quantity).map(move |i| ExpandedPiece {
                original_id: p.id.clone(),
                instance_index: i,
                label: p.label.clone(),
                width: p.width,
                length: p.length,
                can_rotate: p.can_rotate,
                edge_banding: p.edge_banding.clone(),
            })
        })
        .collect();

    // Sort by length descending (FFDH: tallest pieces first)
    expanded.sort_by(|a, b| b.length.cmp(&a.length));

    let total_pieces = expanded.len() as u32;
    let mut sheets: Vec<SheetState> = Vec::new();

    // Place each piece
    for piece in &expanded {
        let mut placed = false;

        // Try existing sheets
        for sheet in &mut sheets {
            if sheet.try_place_piece(piece) {
                placed = true;
                break;
            }
        }

        // Create new sheet if needed
        if !placed {
            let mut new_sheet = SheetState::new(stock_sheet.clone(), blade_kerf_u32);
            new_sheet.try_place_piece(piece);
            sheets.push(new_sheet);
        }
    }

    // Build result
    let layouts: Vec<SheetLayout> = sheets
        .iter()
        .enumerate()
        .map(|(i, s)| s.to_layout(i))
        .collect();

    let total_sheets = layouts.len() as u32;
    let total_area: u64 = layouts.iter()
        .map(|l| l.width as u64 * l.length as u64)
        .sum();
    let used_area: u64 = layouts.iter().map(|l| l.used_area).sum();
    let waste_area = total_area - used_area;
    let waste_percentage = if total_area > 0 {
        (waste_area as f64 / total_area as f64) * 100.0
    } else {
        0.0
    };

    OptimizeResult {
        layouts,
        total_sheets,
        total_pieces,
        total_area,
        used_area,
        waste_area,
        waste_percentage,
    }
}
```

**Step 3: Add solver tests**

Add to the `#[cfg(test)]` module:

```rust
    #[test]
    fn test_solve_single_piece() {
        let pieces = vec![CutPiece {
            id: "panel-a".to_string(),
            width: 580,
            length: 418,
            quantity: 1,
            label: Some("Side Panel".to_string()),
            can_rotate: true,
            edge_banding: None,
        }];

        let stock = StockSheet {
            id: "sheet-1".to_string(),
            name: "BOARD White".to_string(),
            width: 2740,
            length: 1820,
            thickness: 16,
            quantity: None,
            cost: None,
        };

        let result = solve_ffdh(&pieces, &stock, 4.0);

        assert_eq!(result.total_sheets, 1);
        assert_eq!(result.total_pieces, 1);
        assert_eq!(result.layouts[0].pieces.len(), 1);
        assert_eq!(result.layouts[0].pieces[0].x, 0);
        assert_eq!(result.layouts[0].pieces[0].y, 0);
    }

    #[test]
    fn test_solve_multiple_pieces_same_height() {
        let pieces = vec![CutPiece {
            id: "panel".to_string(),
            width: 580,
            length: 418,
            quantity: 4,
            label: None,
            can_rotate: false,
            edge_banding: None,
        }];

        let stock = StockSheet {
            id: "sheet-1".to_string(),
            name: "BOARD White".to_string(),
            width: 2740,
            length: 1820,
            thickness: 16,
            quantity: None,
            cost: None,
        };

        let result = solve_ffdh(&pieces, &stock, 4.0);

        assert_eq!(result.total_sheets, 1);
        assert_eq!(result.total_pieces, 4);

        // All 4 pieces should fit on one shelf (580*4 + 4*3 = 2332 < 2740)
        assert_eq!(result.layouts[0].pieces.len(), 4);
    }

    #[test]
    fn test_solve_pieces_need_multiple_sheets() {
        // Large pieces that each need significant space
        let pieces = vec![CutPiece {
            id: "large-panel".to_string(),
            width: 2000,
            length: 1500,
            quantity: 3,
            label: None,
            can_rotate: false,
            edge_banding: None,
        }];

        let stock = StockSheet {
            id: "sheet-1".to_string(),
            name: "BOARD White".to_string(),
            width: 2740,
            length: 1820,
            thickness: 16,
            quantity: None,
            cost: None,
        };

        let result = solve_ffdh(&pieces, &stock, 4.0);

        // Each piece takes most of a sheet, so need 3 sheets
        assert_eq!(result.total_sheets, 3);
        assert_eq!(result.total_pieces, 3);
    }

    #[test]
    fn test_solve_with_rotation() {
        // Piece is 2000x500, sheet is 2740x1820
        // Without rotation: fits normally
        // This tests that rotation logic doesn't break normal cases
        let pieces = vec![CutPiece {
            id: "panel".to_string(),
            width: 500,
            length: 2000,
            quantity: 1,
            label: None,
            can_rotate: true,
            edge_banding: None,
        }];

        let stock = StockSheet {
            id: "sheet-1".to_string(),
            name: "BOARD White".to_string(),
            width: 2740,
            length: 1820,
            thickness: 16,
            quantity: None,
            cost: None,
        };

        let result = solve_ffdh(&pieces, &stock, 4.0);

        assert_eq!(result.total_sheets, 1);
        assert_eq!(result.total_pieces, 1);
        // Piece is taller than sheet, so should be rotated
        assert!(result.layouts[0].pieces[0].rotated);
    }
```

**Step 4: Run tests**

Run: `cargo test`
Expected: All tests pass

**Step 5: Commit**

```bash
git add src/optimizer/ffdh.rs
git commit -m "feat: implement FFDH solver algorithm"
```

---

## Phase 3: API Layer

### Task 6: Create API Request/Response Types

**Files:**
- Create: `src/api/mod.rs`
- Create: `src/api/requests.rs`
- Create: `src/api/responses.rs`

**Step 1: Create src/api/mod.rs**

```rust
pub mod requests;
pub mod responses;
pub mod handlers;
pub mod routes;

pub use requests::*;
pub use responses::*;
```

**Step 2: Create src/api/requests.rs**

```rust
use serde::Deserialize;
use crate::optimizer::{CutPiece, StockSheet, CutParameters};

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
}

#[derive(Debug, Default, Deserialize)]
pub struct OutputOptions {
    #[serde(default)]
    pub generate_pdf: bool,
    #[serde(default)]
    pub generate_svg: bool,
    #[serde(default)]
    pub include_cutting_list: bool,
    #[serde(default = "default_units")]
    pub units: String,
}

fn default_units() -> String {
    "millimeters".to_string()
}

impl OptimizeRequest {
    /// Count total pieces (sum of quantities)
    pub fn total_piece_count(&self) -> u32 {
        self.pieces.iter().map(|p| p.quantity).sum()
    }
}
```

**Step 3: Create src/api/responses.rs**

```rust
use serde::Serialize;
use crate::optimizer::OptimizeResult;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiErrorDetail>,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(result: T) -> Self {
        ApiResponse {
            success: true,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(code: &str, message: &str) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            result: None,
            error: Some(ApiErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
                field: None,
            }),
        }
    }

    pub fn error_with_field(code: &str, message: &str, field: &str) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            result: None,
            error: Some(ApiErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
                field: Some(field.to_string()),
            }),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OptimizeResponse {
    pub job_reference: String,
    #[serde(flatten)]
    pub result: OptimizeResult,
}
```

**Step 4: Verify compilation**

Run: `cargo build`
Expected: Compiles (handlers and routes not yet created)

**Step 5: Commit**

```bash
git add src/api/
git commit -m "feat: add API request and response types"
```

---

### Task 7: Create Validation Logic

**Files:**
- Create: `src/api/validation.rs`
- Modify: `src/api/mod.rs`

**Step 1: Create src/api/validation.rs**

```rust
use crate::api::requests::OptimizeRequest;
use crate::api::responses::ApiResponse;

pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
}

pub fn validate_request(request: &OptimizeRequest) -> Result<(), ValidationError> {
    // Check for at least one piece
    if request.pieces.is_empty() {
        return Err(ValidationError {
            code: "NO_PIECES".to_string(),
            message: "At least one piece is required".to_string(),
            field: Some("pieces".to_string()),
        });
    }

    // Check for at least one stock sheet
    if request.stock_sheets.is_empty() {
        return Err(ValidationError {
            code: "NO_STOCK_SHEETS".to_string(),
            message: "At least one stock sheet is required".to_string(),
            field: Some("stock_sheets".to_string()),
        });
    }

    // Validate each piece
    for (i, piece) in request.pieces.iter().enumerate() {
        if piece.width == 0 || piece.length == 0 {
            return Err(ValidationError {
                code: "INVALID_DIMENSIONS".to_string(),
                message: format!("Piece '{}' has invalid dimensions ({}x{})", piece.id, piece.width, piece.length),
                field: Some(format!("pieces[{}]", i)),
            });
        }

        if piece.quantity == 0 {
            return Err(ValidationError {
                code: "INVALID_QUANTITY".to_string(),
                message: format!("Piece '{}' has zero quantity", piece.id),
                field: Some(format!("pieces[{}].quantity", i)),
            });
        }

        // Check if piece fits in at least one stock sheet
        let fits_any = request.stock_sheets.iter().any(|sheet| {
            let fits_normal = piece.width <= sheet.width && piece.length <= sheet.length;
            let fits_rotated = piece.can_rotate && piece.length <= sheet.width && piece.width <= sheet.length;
            fits_normal || fits_rotated
        });

        if !fits_any {
            let max_sheet = request.stock_sheets.iter()
                .max_by_key(|s| s.width as u64 * s.length as u64)
                .unwrap();
            return Err(ValidationError {
                code: "PIECE_TOO_LARGE".to_string(),
                message: format!(
                    "Piece '{}' ({}x{}mm) exceeds largest stock sheet ({}x{}mm)",
                    piece.id, piece.width, piece.length, max_sheet.width, max_sheet.length
                ),
                field: Some(format!("pieces[{}]", i)),
            });
        }
    }

    // Validate stock sheets
    for (i, sheet) in request.stock_sheets.iter().enumerate() {
        if sheet.width == 0 || sheet.length == 0 {
            return Err(ValidationError {
                code: "INVALID_SHEET_DIMENSIONS".to_string(),
                message: format!("Stock sheet '{}' has invalid dimensions", sheet.id),
                field: Some(format!("stock_sheets[{}]", i)),
            });
        }
    }

    // Validate blade kerf
    if request.parameters.blade_kerf < 0.0 || request.parameters.blade_kerf > 20.0 {
        return Err(ValidationError {
            code: "INVALID_BLADE_KERF".to_string(),
            message: format!("Blade kerf must be between 0 and 20mm, got {}", request.parameters.blade_kerf),
            field: Some("parameters.blade_kerf".to_string()),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::{CutPiece, StockSheet, CutParameters};
    use crate::api::requests::OutputOptions;

    fn make_valid_request() -> OptimizeRequest {
        OptimizeRequest {
            job_reference: "TEST-001".to_string(),
            client_name: None,
            pieces: vec![CutPiece {
                id: "piece-1".to_string(),
                width: 580,
                length: 418,
                quantity: 1,
                label: None,
                can_rotate: true,
                edge_banding: None,
            }],
            stock_sheets: vec![StockSheet {
                id: "sheet-1".to_string(),
                name: "BOARD White".to_string(),
                width: 2740,
                length: 1820,
                thickness: 16,
                quantity: None,
                cost: None,
            }],
            parameters: CutParameters::default(),
            output: OutputOptions::default(),
        }
    }

    #[test]
    fn test_valid_request_passes() {
        let request = make_valid_request();
        assert!(validate_request(&request).is_ok());
    }

    #[test]
    fn test_no_pieces_fails() {
        let mut request = make_valid_request();
        request.pieces.clear();

        let err = validate_request(&request).unwrap_err();
        assert_eq!(err.code, "NO_PIECES");
    }

    #[test]
    fn test_no_stock_sheets_fails() {
        let mut request = make_valid_request();
        request.stock_sheets.clear();

        let err = validate_request(&request).unwrap_err();
        assert_eq!(err.code, "NO_STOCK_SHEETS");
    }

    #[test]
    fn test_piece_too_large_fails() {
        let mut request = make_valid_request();
        request.pieces[0].width = 3000; // Larger than sheet
        request.pieces[0].can_rotate = false;

        let err = validate_request(&request).unwrap_err();
        assert_eq!(err.code, "PIECE_TOO_LARGE");
    }

    #[test]
    fn test_zero_dimensions_fails() {
        let mut request = make_valid_request();
        request.pieces[0].width = 0;

        let err = validate_request(&request).unwrap_err();
        assert_eq!(err.code, "INVALID_DIMENSIONS");
    }

    #[test]
    fn test_invalid_blade_kerf_fails() {
        let mut request = make_valid_request();
        request.parameters.blade_kerf = 25.0;

        let err = validate_request(&request).unwrap_err();
        assert_eq!(err.code, "INVALID_BLADE_KERF");
    }
}
```

**Step 2: Update src/api/mod.rs**

```rust
pub mod requests;
pub mod responses;
pub mod validation;
pub mod handlers;
pub mod routes;

pub use requests::*;
pub use responses::*;
pub use validation::*;
```

**Step 3: Run tests**

Run: `cargo test validation`
Expected: All validation tests pass

**Step 4: Commit**

```bash
git add src/api/
git commit -m "feat: add request validation with tests"
```

---

### Task 8: Create API Handlers

**Files:**
- Create: `src/api/handlers.rs`

**Step 1: Create src/api/handlers.rs**

```rust
use actix_web::{web, HttpResponse};
use crate::api::{OptimizeRequest, ApiResponse, OptimizeResponse, validate_request};
use crate::optimizer::solve_ffdh;

/// POST /api/v1/validate
pub async fn validate(
    request: web::Json<OptimizeRequest>,
) -> HttpResponse {
    match validate_request(&request) {
        Ok(()) => HttpResponse::Ok().json(ApiResponse::<()>::success(())),
        Err(e) => {
            let response = if let Some(field) = e.field {
                ApiResponse::<()>::error_with_field(&e.code, &e.message, &field)
            } else {
                ApiResponse::<()>::error(&e.code, &e.message)
            };
            HttpResponse::BadRequest().json(response)
        }
    }
}

/// POST /api/v1/optimize/quick
pub async fn optimize_quick(
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

    // For now, use the first stock sheet
    // TODO: Support multiple stock sheet types
    let stock_sheet = &request.stock_sheets[0];

    let result = solve_ffdh(
        &request.pieces,
        stock_sheet,
        request.parameters.blade_kerf,
    );

    let response = OptimizeResponse {
        job_reference: request.job_reference.clone(),
        result,
    };

    HttpResponse::Ok().json(ApiResponse::success(response))
}

/// GET /api/v1/templates
pub async fn get_templates() -> HttpResponse {
    use crate::optimizer::StockSheet;

    let templates = vec![
        StockSheet {
            id: "melamine-white".to_string(),
            name: "BOARD White".to_string(),
            width: 2740,
            length: 1820,
            thickness: 16,
            quantity: None,
            cost: Some(50.0),
        },
        StockSheet {
            id: "mdf-white".to_string(),
            name: "MDF Masonite White".to_string(),
            width: 2750,
            length: 1830,
            thickness: 16,
            quantity: None,
            cost: Some(45.0),
        },
        StockSheet {
            id: "pvc-foam".to_string(),
            name: "PVC FOAM Board".to_string(),
            width: 2750,
            length: 1830,
            thickness: 16,
            quantity: None,
            cost: Some(80.0),
        },
    ];

    HttpResponse::Ok().json(ApiResponse::success(templates))
}
```

**Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/api/handlers.rs
git commit -m "feat: add API handlers for validate, optimize, and templates"
```

---

### Task 9: Create Routes and Update Main

**Files:**
- Create: `src/api/routes.rs`
- Modify: `src/main.rs`

**Step 1: Create src/api/routes.rs**

```rust
use actix_web::web;
use super::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/validate", web::post().to(handlers::validate))
            .route("/optimize/quick", web::post().to(handlers::optimize_quick))
            .route("/templates", web::get().to(handlers::get_templates))
    );
}
```

**Step 2: Update src/main.rs**

```rust
use actix_web::{web, App, HttpServer, HttpResponse};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod optimizer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

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

**Step 3: Update src/lib.rs**

```rust
pub mod optimizer;
pub mod api;
```

**Step 4: Build and test**

Run: `cargo build`
Expected: Compiles

Run in background: `cargo run &`

Test templates:
```bash
curl http://localhost:8080/api/v1/templates
```
Expected: JSON list of stock sheet templates

Test optimize:
```bash
curl -X POST http://localhost:8080/api/v1/optimize/quick \
  -H "Content-Type: application/json" \
  -d '{
    "job_reference": "TEST-001",
    "pieces": [{"id": "panel-a", "width": 580, "length": 418, "quantity": 4}],
    "stock_sheets": [{"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820}]
  }'
```
Expected: JSON response with layout results

**Step 5: Commit**

```bash
git add src/
git commit -m "feat: wire up API routes and complete basic optimization endpoint"
```

---

## Phase 4: Integration Tests

### Task 10: Create Integration Tests

**Files:**
- Create: `tests/api_tests.rs`

**Step 1: Create tests/api_tests.rs**

```rust
use actix_web::{test, web, App};
use cut_optimizer_api::api;

#[actix_rt::test]
async fn test_health_endpoint() {
    let app = test::init_service(
        App::new()
            .route("/health", web::get().to(|| async {
                actix_web::HttpResponse::Ok().json(serde_json::json!({
                    "status": "healthy"
                }))
            }))
    ).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_templates_endpoint() {
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/templates")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["result"].as_array().unwrap().len() > 0);
}

#[actix_rt::test]
async fn test_validate_valid_request() {
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/validate")
        .set_json(serde_json::json!({
            "job_reference": "TEST-001",
            "pieces": [{"id": "panel-a", "width": 580, "length": 418, "quantity": 1}],
            "stock_sheets": [{"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820}]
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_validate_rejects_empty_pieces() {
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/validate")
        .set_json(serde_json::json!({
            "job_reference": "TEST-001",
            "pieces": [],
            "stock_sheets": [{"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820}]
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 400);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(!body["success"].as_bool().unwrap());
    assert_eq!(body["error"]["code"], "NO_PIECES");
}

#[actix_rt::test]
async fn test_optimize_quick_basic() {
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/optimize/quick")
        .set_json(serde_json::json!({
            "job_reference": "TEST-001",
            "pieces": [
                {"id": "panel-a", "width": 580, "length": 418, "quantity": 4}
            ],
            "stock_sheets": [
                {"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820}
            ]
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["result"]["job_reference"], "TEST-001");
    assert_eq!(body["result"]["total_pieces"], 4);
    assert!(body["result"]["total_sheets"].as_u64().unwrap() >= 1);
}

#[actix_rt::test]
async fn test_optimize_quick_with_edge_banding() {
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/optimize/quick")
        .set_json(serde_json::json!({
            "job_reference": "TEST-002",
            "pieces": [
                {
                    "id": "panel-a",
                    "width": 580,
                    "length": 418,
                    "quantity": 2,
                    "edge_banding": {
                        "top": true,
                        "bottom": true,
                        "left": false,
                        "right": false,
                        "material": "White 1mm"
                    }
                }
            ],
            "stock_sheets": [
                {"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820}
            ]
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());

    // Check that edge banding is preserved in output
    let pieces = &body["result"]["layouts"][0]["pieces"];
    assert!(pieces[0]["edge_banding"].is_object());
}
```

**Step 2: Run integration tests**

Run: `cargo test --test api_tests`
Expected: All integration tests pass

**Step 3: Commit**

```bash
git add tests/
git commit -m "test: add API integration tests"
```

---

## Phase 5: Docker Setup

### Task 11: Create Dockerfile

**Files:**
- Create: `Dockerfile`
- Create: `.dockerignore`

**Step 1: Create Dockerfile**

```dockerfile
# Build stage
FROM rust:1.75-slim-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Create dummy source to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual source
COPY src ./src
COPY tests ./tests

# Build for release
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/cut-optimizer-api /app/cut-optimizer-api

# Set environment variables
ENV HOST=0.0.0.0
ENV PORT=8080
ENV RUST_LOG=info

EXPOSE 8080

CMD ["/app/cut-optimizer-api"]
```

**Step 2: Create .dockerignore**

```
target/
.git/
.gitignore
*.md
.env*
docker-compose*.yml
Dockerfile*
```

**Step 3: Build Docker image**

Run: `docker build -t cut-optimizer-api .`
Expected: Image builds successfully

**Step 4: Test Docker image**

Run: `docker run -p 8080:8080 cut-optimizer-api &`
Then: `curl http://localhost:8080/health`
Expected: `{"status":"healthy","version":"0.1.0"}`

**Step 5: Commit**

```bash
git add Dockerfile .dockerignore
git commit -m "feat: add Dockerfile for containerized deployment"
```

---

### Task 12: Create Docker Compose for Local Development

**Files:**
- Create: `docker-compose.yml`

**Step 1: Create docker-compose.yml**

```yaml
version: '3.8'

services:
  api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - HOST=0.0.0.0
      - PORT=8080
      - RUST_LOG=debug,cut_optimizer_api=trace
      - REDIS_URL=redis://redis:6379
      - DATABASE_URL=postgres://postgres:postgres@postgres:5432/cut_optimizer
    depends_on:
      - redis
      - postgres
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  postgres:
    image: postgres:15-alpine
    ports:
      - "5432:5432"
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=postgres
      - POSTGRES_DB=cut_optimizer
    volumes:
      - postgres_data:/var/lib/postgresql/data

  minio:
    image: minio/minio
    ports:
      - "9000:9000"
      - "9001:9001"
    environment:
      - MINIO_ROOT_USER=minioadmin
      - MINIO_ROOT_PASSWORD=minioadmin
    command: server /data --console-address ":9001"
    volumes:
      - minio_data:/data

volumes:
  redis_data:
  postgres_data:
  minio_data:
```

**Step 2: Test docker-compose**

Run: `docker-compose up -d`
Then: `curl http://localhost:8080/health`
Expected: API responds healthy

**Step 3: Commit**

```bash
git add docker-compose.yml
git commit -m "feat: add docker-compose for local development"
```

---

## Summary

This plan covers the core functionality:

1. **Project Setup** - Rust project with Actix-web and health endpoint
2. **Core Types** - Data structures for pieces, sheets, and layouts
3. **FFDH Algorithm** - Shelf-based packing with guillotine constraints
4. **API Layer** - Validation, handlers, and routes
5. **Integration Tests** - End-to-end API testing
6. **Docker** - Containerized deployment

### Not Yet Implemented (Future Tasks)

- Async job processing with Redis queue
- PostgreSQL persistence
- PDF generation with printpdf
- SVG generation
- S3 file storage
- Job status polling endpoints
- Multiple stock sheet type support in single optimization

### Verification Checklist

After completing all tasks:

```bash
# Run all tests
cargo test

# Start server
cargo run

# Test endpoints
curl http://localhost:8080/health
curl http://localhost:8080/api/v1/templates
curl -X POST http://localhost:8080/api/v1/optimize/quick \
  -H "Content-Type: application/json" \
  -d '{"job_reference":"TEST","pieces":[{"id":"a","width":580,"length":418,"quantity":4}],"stock_sheets":[{"id":"s","name":"Board","width":2740,"length":1820}]}'
```

# PDF Generation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Generate PDF layout diagrams from optimization results, returning base64-encoded PDF in API response.

**Architecture:** New `src/output/pdf.rs` module using printpdf library. Each sheet layout becomes one PDF page with header, cutting list sidebar, and scaled diagram showing pieces with dimensions and edge banding.

**Tech Stack:** printpdf 0.7, base64 0.22

---

## Phase 1: Setup and Module Structure

### Task 1: Add Dependencies

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add printpdf and base64 dependencies**

Add to `[dependencies]` section in `Cargo.toml`:

```toml
printpdf = "0.7"
base64 = "0.22"
```

**Step 2: Verify dependencies resolve**

Run: `cargo check`
Expected: Compiles successfully (dependencies download and resolve)

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "feat: add printpdf and base64 dependencies"
```

---

### Task 2: Create Output Module Structure

**Files:**
- Create: `src/output/mod.rs`
- Create: `src/output/pdf.rs`
- Modify: `src/lib.rs`

**Step 1: Create src/output/mod.rs**

```rust
pub mod pdf;

pub use pdf::*;
```

**Step 2: Create src/output/pdf.rs with error type and stub**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfError {
    #[error("Failed to create PDF document: {0}")]
    DocumentCreation(String),
    #[error("Failed to add font: {0}")]
    FontError(String),
    #[error("Failed to save PDF: {0}")]
    SaveError(String),
}

/// Generate PDF bytes from optimization result
pub fn generate_pdf(
    _result: &crate::optimizer::OptimizeResult,
    _job_reference: &str,
    _client_name: Option<&str>,
    _stock_sheet: &crate::optimizer::StockSheet,
) -> Result<Vec<u8>, PdfError> {
    // Stub - will be implemented in subsequent tasks
    Err(PdfError::DocumentCreation("Not implemented".to_string()))
}
```

**Step 3: Update src/lib.rs to include output module**

```rust
pub mod optimizer;
pub mod api;
pub mod output;
```

**Step 4: Verify it compiles**

Run: `cargo build`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add src/output/ src/lib.rs
git commit -m "feat: add output module structure with PDF stub"
```

---

## Phase 2: Core PDF Generation

### Task 3: Implement Basic PDF Document Creation

**Files:**
- Modify: `src/output/pdf.rs`

**Step 1: Implement generate_pdf with empty page**

Replace the stub implementation:

```rust
use thiserror::Error;
use printpdf::*;
use std::io::BufWriter;

use crate::optimizer::{OptimizeResult, StockSheet, SheetLayout};

#[derive(Error, Debug)]
pub enum PdfError {
    #[error("Failed to create PDF document: {0}")]
    DocumentCreation(String),
    #[error("Failed to add font: {0}")]
    FontError(String),
    #[error("Failed to save PDF: {0}")]
    SaveError(String),
}

// A4 dimensions in mm
const PAGE_WIDTH: f32 = 210.0;
const PAGE_HEIGHT: f32 = 297.0;

/// Generate PDF bytes from optimization result
pub fn generate_pdf(
    result: &OptimizeResult,
    job_reference: &str,
    client_name: Option<&str>,
    stock_sheet: &StockSheet,
) -> Result<Vec<u8>, PdfError> {
    let (doc, page1, layer1) = PdfDocument::new(
        "Cut Layout",
        Mm(PAGE_WIDTH),
        Mm(PAGE_HEIGHT),
        "Layer 1",
    );

    // Add built-in font
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| PdfError::FontError(e.to_string()))?;

    // Draw title on first page
    let current_layer = doc.get_page(page1).get_layer(layer1);
    current_layer.use_text("Job Layout", 18.0, Mm(105.0), Mm(285.0), &font);

    // Add additional pages for remaining layouts
    for i in 1..result.layouts.len() {
        let (page, layer) = doc.add_page(Mm(PAGE_WIDTH), Mm(PAGE_HEIGHT), format!("Page {}", i + 1));
        let current_layer = doc.get_page(page).get_layer(layer);
        current_layer.use_text("Job Layout", 18.0, Mm(105.0), Mm(285.0), &font);
    }

    // Save to bytes
    let mut buffer = BufWriter::new(Vec::new());
    doc.save(&mut buffer)
        .map_err(|e| PdfError::SaveError(e.to_string()))?;

    buffer.into_inner()
        .map_err(|e| PdfError::SaveError(e.to_string()))
}
```

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/output/pdf.rs
git commit -m "feat: implement basic PDF document creation"
```

---

### Task 4: Add Page Layout Constants and Header Drawing

**Files:**
- Modify: `src/output/pdf.rs`

**Step 1: Add layout constants and header function**

Add after the existing constants:

```rust
// Layout regions (in mm from bottom-left origin)
const HEADER_Y: f32 = 272.0;       // Header starts here (297 - 25)
const SIDEBAR_WIDTH: f32 = 55.0;
const DIAGRAM_X: f32 = 60.0;
const DIAGRAM_Y: f32 = 10.0;
const DIAGRAM_WIDTH: f32 = 145.0;
const DIAGRAM_HEIGHT: f32 = 255.0;

fn draw_header(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    job_reference: &str,
    client_name: Option<&str>,
    stock_sheet: &StockSheet,
    layout_index: usize,
    total_layouts: usize,
    layout: &SheetLayout,
    total_pieces: u32,
    waste_percentage: f64,
) {
    // Title
    layer.use_text("Job Layout", 16.0, Mm(PAGE_WIDTH / 2.0 - 15.0), Mm(HEADER_Y + 15.0), font_bold);

    // Material info (top-left)
    layer.use_text(&stock_sheet.name, 10.0, Mm(10.0), Mm(HEADER_Y + 15.0), font_bold);
    layer.use_text(
        &format!("Size: {} mm x {} mm x {} mm", stock_sheet.width, stock_sheet.length, stock_sheet.thickness),
        8.0, Mm(10.0), Mm(HEADER_Y + 10.0), font
    );

    // Job info (left side)
    layer.use_text(&format!("Job Reference: {}", job_reference), 8.0, Mm(10.0), Mm(HEADER_Y), font);
    if let Some(client) = client_name {
        layer.use_text(&format!("Client: {}", client), 8.0, Mm(10.0), Mm(HEADER_Y - 5.0), font);
    }

    // Layout info (right side)
    layer.use_text(
        &format!("Layout {} of {} - {} ({}mm x {}mm)",
            layout_index + 1, total_layouts, stock_sheet.name, stock_sheet.width, stock_sheet.length),
        8.0, Mm(PAGE_WIDTH / 2.0), Mm(HEADER_Y), font
    );
    layer.use_text(
        &format!("Sheet Panels: {}    Total Sheets: {}", layout.pieces.len(), total_layouts),
        8.0, Mm(PAGE_WIDTH / 2.0), Mm(HEADER_Y - 5.0), font
    );
    layer.use_text(
        &format!("Job Panels: {}    Job Wastage: {:.2}%", total_pieces, waste_percentage),
        8.0, Mm(PAGE_WIDTH / 2.0), Mm(HEADER_Y - 10.0), font
    );
}
```

**Step 2: Update generate_pdf to use header function**

Replace the generate_pdf function:

```rust
pub fn generate_pdf(
    result: &OptimizeResult,
    job_reference: &str,
    client_name: Option<&str>,
    stock_sheet: &StockSheet,
) -> Result<Vec<u8>, PdfError> {
    let (doc, page1, layer1) = PdfDocument::new(
        "Cut Layout",
        Mm(PAGE_WIDTH),
        Mm(PAGE_HEIGHT),
        "Layer 1",
    );

    let font = doc.add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| PdfError::FontError(e.to_string()))?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| PdfError::FontError(e.to_string()))?;

    // Draw first page
    if let Some(layout) = result.layouts.first() {
        let current_layer = doc.get_page(page1).get_layer(layer1);
        draw_header(
            &current_layer, &font, &font_bold,
            job_reference, client_name, stock_sheet,
            0, result.layouts.len(), layout,
            result.total_pieces, result.waste_percentage,
        );
    }

    // Add pages for remaining layouts
    for (i, layout) in result.layouts.iter().enumerate().skip(1) {
        let (page, layer) = doc.add_page(Mm(PAGE_WIDTH), Mm(PAGE_HEIGHT), format!("Page {}", i + 1));
        let current_layer = doc.get_page(page).get_layer(layer);
        draw_header(
            &current_layer, &font, &font_bold,
            job_reference, client_name, stock_sheet,
            i, result.layouts.len(), layout,
            result.total_pieces, result.waste_percentage,
        );
    }

    let mut buffer = BufWriter::new(Vec::new());
    doc.save(&mut buffer)
        .map_err(|e| PdfError::SaveError(e.to_string()))?;

    buffer.into_inner()
        .map_err(|e| PdfError::SaveError(e.to_string()))
}
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: Compiles without errors (warnings about unused constants are OK)

**Step 4: Commit**

```bash
git add src/output/pdf.rs
git commit -m "feat: add PDF header drawing with job metadata"
```

---

### Task 5: Add Cutting List Sidebar

**Files:**
- Modify: `src/output/pdf.rs`

**Step 1: Add CuttingListEntry struct and build function**

Add after the constants:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct CuttingListEntry {
    id: String,
    length: u32,
    width: u32,
    quantity: u32,
}

fn build_cutting_list(layouts: &[SheetLayout]) -> Vec<CuttingListEntry> {
    let mut entries: HashMap<String, CuttingListEntry> = HashMap::new();

    for layout in layouts {
        for piece in &layout.pieces {
            // Extract original ID (strip instance suffix: "panel-a-0" -> "panel-a")
            let original_id = piece.piece_id.rsplit_once('-')
                .and_then(|(prefix, suffix)| {
                    // Only strip if suffix is numeric
                    if suffix.chars().all(|c| c.is_ascii_digit()) {
                        Some(prefix)
                    } else {
                        None
                    }
                })
                .unwrap_or(&piece.piece_id);

            entries.entry(original_id.to_string())
                .and_modify(|e| e.quantity += 1)
                .or_insert(CuttingListEntry {
                    id: original_id.to_string(),
                    length: piece.length,
                    width: piece.width,
                    quantity: 1,
                });
        }
    }

    let mut list: Vec<_> = entries.into_values().collect();
    list.sort_by(|a, b| a.id.cmp(&b.id));
    list
}
```

**Step 2: Add draw_sidebar function**

```rust
fn draw_sidebar(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    cutting_list: &[CuttingListEntry],
) {
    let mut y = HEADER_Y - 25.0;

    // Cutting List header
    layer.use_text("Cutting List", 9.0, Mm(5.0), Mm(y), font_bold);
    y -= 5.0;

    // Table header
    layer.use_text("Id", 7.0, Mm(5.0), Mm(y), font_bold);
    layer.use_text("Length", 7.0, Mm(15.0), Mm(y), font_bold);
    layer.use_text("Width", 7.0, Mm(30.0), Mm(y), font_bold);
    layer.use_text("Qty", 7.0, Mm(45.0), Mm(y), font_bold);
    y -= 4.0;

    // Table rows
    for entry in cutting_list {
        if y < 30.0 { break; } // Stop if running out of space

        layer.use_text(&entry.id, 7.0, Mm(5.0), Mm(y), font);
        layer.use_text(&entry.length.to_string(), 7.0, Mm(15.0), Mm(y), font);
        layer.use_text(&entry.width.to_string(), 7.0, Mm(30.0), Mm(y), font);
        layer.use_text(&entry.quantity.to_string(), 7.0, Mm(45.0), Mm(y), font);
        y -= 4.0;
    }

    // Edging Legend
    y -= 5.0;
    layer.use_text("Edging Legend", 9.0, Mm(5.0), Mm(y), font_bold);
    y -= 5.0;

    // Draw dashed line sample
    let line = Line {
        points: vec![
            (Point::new(Mm(5.0), Mm(y)), false),
            (Point::new(Mm(35.0), Mm(y)), false),
        ],
        is_closed: false,
    };
    layer.set_outline_color(Color::Rgb(Rgb::new(0.8, 0.0, 0.0, None)));
    layer.set_outline_thickness(0.5);
    // Note: printpdf doesn't support dash patterns directly in basic API
    // We'll draw the legend as text indication instead
    layer.add_line(line);
    layer.use_text("Edge Banding", 7.0, Mm(5.0), Mm(y - 4.0), font);
}
```

**Step 3: Update generate_pdf to draw sidebar**

Add after each draw_header call:

```rust
// In the first page block, after draw_header:
let cutting_list = build_cutting_list(&result.layouts);
draw_sidebar(&current_layer, &font, &font_bold, &cutting_list);

// In the loop for additional pages, after draw_header:
draw_sidebar(&current_layer, &font, &font_bold, &cutting_list);
```

**Step 4: Verify it compiles**

Run: `cargo build`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add src/output/pdf.rs
git commit -m "feat: add cutting list sidebar to PDF"
```

---

### Task 6: Add Sheet Diagram Drawing

**Files:**
- Modify: `src/output/pdf.rs`

**Step 1: Add calculate_scale function**

```rust
fn calculate_scale(sheet_width: u32, sheet_length: u32) -> f32 {
    let scale_x = DIAGRAM_WIDTH / sheet_width as f32;
    let scale_y = DIAGRAM_HEIGHT / sheet_length as f32;
    scale_x.min(scale_y)
}
```

**Step 2: Add draw_diagram function**

```rust
fn draw_diagram(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    layout: &SheetLayout,
    stock_sheet: &StockSheet,
) {
    let scale = calculate_scale(stock_sheet.width, stock_sheet.length);

    // Calculate diagram position (centered in available area)
    let diagram_sheet_width = stock_sheet.width as f32 * scale;
    let diagram_sheet_height = stock_sheet.length as f32 * scale;
    let offset_x = DIAGRAM_X + (DIAGRAM_WIDTH - diagram_sheet_width) / 2.0;
    let offset_y = DIAGRAM_Y + (DIAGRAM_HEIGHT - diagram_sheet_height) / 2.0;

    // Draw sheet outline
    layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    layer.set_outline_thickness(1.0);

    let sheet_rect = Rect::new(
        Mm(offset_x), Mm(offset_y),
        Mm(offset_x + diagram_sheet_width), Mm(offset_y + diagram_sheet_height),
    );
    layer.add_rect(sheet_rect);

    // Draw sheet dimension labels
    layer.use_text(
        &format!("{} mm", stock_sheet.width),
        7.0, Mm(offset_x + diagram_sheet_width / 2.0 - 10.0), Mm(offset_y + diagram_sheet_height + 3.0), font
    );
    layer.use_text(
        &format!("{} mm", stock_sheet.length),
        7.0, Mm(offset_x + diagram_sheet_width + 2.0), Mm(offset_y + diagram_sheet_height / 2.0), font
    );

    // Draw pieces
    for piece in &layout.pieces {
        draw_piece(layer, font, piece, scale, offset_x, offset_y, diagram_sheet_height);
    }
}

fn draw_piece(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    piece: &crate::optimizer::PlacedPiece,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
    diagram_sheet_height: f32,
) {
    let x = offset_x + piece.x as f32 * scale;
    // Y is inverted: piece.y=0 is at top of sheet, but PDF y=0 is at bottom
    let y = offset_y + diagram_sheet_height - (piece.y as f32 + piece.length as f32) * scale;
    let w = piece.width as f32 * scale;
    let h = piece.length as f32 * scale;

    // Draw piece rectangle
    layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    layer.set_outline_thickness(0.5);

    let piece_rect = Rect::new(Mm(x), Mm(y), Mm(x + w), Mm(y + h));
    layer.add_rect(piece_rect);

    // Draw piece ID centered
    let id_short = piece.piece_id.rsplit_once('-')
        .and_then(|(prefix, suffix)| {
            if suffix.chars().all(|c| c.is_ascii_digit()) {
                Some(prefix)
            } else {
                None
            }
        })
        .unwrap_or(&piece.piece_id);

    if w > 8.0 && h > 8.0 {
        layer.use_text(id_short, 8.0, Mm(x + w / 2.0 - 2.0), Mm(y + h / 2.0 - 1.0), font);
    }

    // Draw dimensions if piece is large enough
    if w > 20.0 {
        layer.use_text(
            &format!("{} mm", piece.width),
            5.0, Mm(x + w / 2.0 - 8.0), Mm(y + h - 3.0), font
        );
    }
    if h > 20.0 {
        layer.use_text(
            &format!("{} mm", piece.length),
            5.0, Mm(x + w - 10.0), Mm(y + h / 2.0), font
        );
    }

    // Draw edge banding (red dashed-style lines)
    if let Some(eb) = &piece.edge_banding {
        layer.set_outline_color(Color::Rgb(Rgb::new(0.8, 0.0, 0.0, None)));
        layer.set_outline_thickness(1.0);

        if eb.top {
            let line = Line {
                points: vec![
                    (Point::new(Mm(x), Mm(y + h)), false),
                    (Point::new(Mm(x + w), Mm(y + h)), false),
                ],
                is_closed: false,
            };
            layer.add_line(line);
        }
        if eb.bottom {
            let line = Line {
                points: vec![
                    (Point::new(Mm(x), Mm(y)), false),
                    (Point::new(Mm(x + w), Mm(y)), false),
                ],
                is_closed: false,
            };
            layer.add_line(line);
        }
        if eb.left {
            let line = Line {
                points: vec![
                    (Point::new(Mm(x), Mm(y)), false),
                    (Point::new(Mm(x), Mm(y + h)), false),
                ],
                is_closed: false,
            };
            layer.add_line(line);
        }
        if eb.right {
            let line = Line {
                points: vec![
                    (Point::new(Mm(x + w), Mm(y)), false),
                    (Point::new(Mm(x + w), Mm(y + h)), false),
                ],
                is_closed: false,
            };
            layer.add_line(line);
        }

        // Reset color to black
        layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    }
}
```

**Step 3: Update generate_pdf to draw diagram**

Add after each draw_sidebar call:

```rust
draw_diagram(&current_layer, &font, layout, stock_sheet);
```

**Step 4: Verify it compiles**

Run: `cargo build`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add src/output/pdf.rs
git commit -m "feat: add sheet diagram with pieces to PDF"
```

---

## Phase 3: API Integration

### Task 7: Update OptimizeResponse to Include PDF

**Files:**
- Modify: `src/api/responses.rs`

**Step 1: Add pdf_base64 field to OptimizeResponse**

Update the struct:

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
}

impl ApiResponse<()> {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_base64: Option<String>,
}
```

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/api/responses.rs
git commit -m "feat: add pdf_base64 field to OptimizeResponse"
```

---

### Task 8: Update Handler to Generate PDF

**Files:**
- Modify: `src/api/handlers.rs`

**Step 1: Update optimize_quick handler**

```rust
use actix_web::{web, HttpResponse};
use base64::{Engine as _, engine::general_purpose};
use crate::api::{OptimizeRequest, ApiResponse, OptimizeResponse, validate_request};
use crate::optimizer::solve_ffdh;
use crate::output::generate_pdf;

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
    let stock_sheet = &request.stock_sheets[0];

    let result = solve_ffdh(
        &request.pieces,
        stock_sheet,
        request.parameters.blade_kerf,
    );

    // Generate PDF if requested
    let pdf_base64 = if request.output.generate_pdf {
        match generate_pdf(&result, &request.job_reference, request.client_name.as_deref(), stock_sheet) {
            Ok(bytes) => Some(general_purpose::STANDARD.encode(&bytes)),
            Err(e) => {
                tracing::error!("PDF generation failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    let response = OptimizeResponse {
        job_reference: request.job_reference.clone(),
        result,
        pdf_base64,
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

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/api/handlers.rs
git commit -m "feat: integrate PDF generation into optimize_quick handler"
```

---

## Phase 4: Testing

### Task 9: Add PDF Integration Test

**Files:**
- Modify: `tests/api_tests.rs`

**Step 1: Add test for PDF generation**

Add to the end of the test file:

```rust
#[actix_rt::test]
async fn test_optimize_quick_with_pdf() {
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/optimize/quick")
        .set_json(serde_json::json!({
            "job_reference": "TEST-PDF-001",
            "client_name": "Test Client",
            "pieces": [
                {"id": "panel-a", "width": 580, "length": 418, "quantity": 4}
            ],
            "stock_sheets": [
                {"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820, "thickness": 16}
            ],
            "output": {
                "generate_pdf": true
            }
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["result"]["job_reference"], "TEST-PDF-001");

    // Verify PDF was generated
    let pdf_base64 = body["result"]["pdf_base64"].as_str();
    assert!(pdf_base64.is_some(), "Expected pdf_base64 in response");

    // Verify it's valid base64 that decodes to PDF
    let pdf_bytes = base64::engine::general_purpose::STANDARD
        .decode(pdf_base64.unwrap())
        .expect("Invalid base64");
    assert!(pdf_bytes.len() > 100, "PDF seems too small");
    assert_eq!(&pdf_bytes[0..4], b"%PDF", "Not a valid PDF header");
}

#[actix_rt::test]
async fn test_optimize_quick_without_pdf() {
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/optimize/quick")
        .set_json(serde_json::json!({
            "job_reference": "TEST-NO-PDF",
            "pieces": [
                {"id": "panel-a", "width": 580, "length": 418, "quantity": 2}
            ],
            "stock_sheets": [
                {"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820}
            ],
            "output": {
                "generate_pdf": false
            }
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());

    // Verify PDF was NOT generated
    assert!(body["result"]["pdf_base64"].is_null(), "Expected no pdf_base64 when generate_pdf is false");
}
```

**Step 2: Add base64 to dev-dependencies**

Update `Cargo.toml`:

```toml
[dev-dependencies]
actix-rt = "2"
base64 = "0.22"
```

**Step 3: Run tests**

Run: `cargo test`
Expected: All tests pass

**Step 4: Commit**

```bash
git add tests/api_tests.rs Cargo.toml
git commit -m "test: add PDF generation integration tests"
```

---

### Task 10: Add Unit Tests for Cutting List Builder

**Files:**
- Modify: `src/output/pdf.rs`

**Step 1: Add test module**

Add at the end of `src/output/pdf.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::{SheetLayout, PlacedPiece};

    #[test]
    fn test_build_cutting_list_aggregates_by_id() {
        let layouts = vec![
            SheetLayout {
                sheet_index: 0,
                stock_sheet_id: "sheet-1".to_string(),
                stock_sheet_name: "BOARD White".to_string(),
                width: 2740,
                length: 1820,
                pieces: vec![
                    PlacedPiece {
                        piece_id: "panel-a-0".to_string(),
                        label: None,
                        x: 0, y: 0, width: 580, length: 418,
                        rotated: false, edge_banding: None,
                    },
                    PlacedPiece {
                        piece_id: "panel-a-1".to_string(),
                        label: None,
                        x: 584, y: 0, width: 580, length: 418,
                        rotated: false, edge_banding: None,
                    },
                    PlacedPiece {
                        piece_id: "panel-b-0".to_string(),
                        label: None,
                        x: 0, y: 422, width: 500, length: 300,
                        rotated: false, edge_banding: None,
                    },
                ],
                used_area: 0,
                waste_percentage: 0.0,
            },
        ];

        let cutting_list = build_cutting_list(&layouts);

        assert_eq!(cutting_list.len(), 2);

        let panel_a = cutting_list.iter().find(|e| e.id == "panel-a").unwrap();
        assert_eq!(panel_a.quantity, 2);
        assert_eq!(panel_a.width, 580);
        assert_eq!(panel_a.length, 418);

        let panel_b = cutting_list.iter().find(|e| e.id == "panel-b").unwrap();
        assert_eq!(panel_b.quantity, 1);
    }

    #[test]
    fn test_build_cutting_list_handles_no_suffix() {
        let layouts = vec![
            SheetLayout {
                sheet_index: 0,
                stock_sheet_id: "sheet-1".to_string(),
                stock_sheet_name: "BOARD White".to_string(),
                width: 2740,
                length: 1820,
                pieces: vec![
                    PlacedPiece {
                        piece_id: "A".to_string(), // No numeric suffix
                        label: None,
                        x: 0, y: 0, width: 580, length: 418,
                        rotated: false, edge_banding: None,
                    },
                ],
                used_area: 0,
                waste_percentage: 0.0,
            },
        ];

        let cutting_list = build_cutting_list(&layouts);

        assert_eq!(cutting_list.len(), 1);
        assert_eq!(cutting_list[0].id, "A");
        assert_eq!(cutting_list[0].quantity, 1);
    }

    #[test]
    fn test_calculate_scale_width_constrained() {
        // Wide sheet: 2740 x 1820
        // Available: 145 x 255
        // Scale by width: 145/2740 = 0.0529
        // Scale by height: 255/1820 = 0.1401
        // Should use width (smaller)
        let scale = calculate_scale(2740, 1820);
        assert!((scale - 0.0529).abs() < 0.001);
    }

    #[test]
    fn test_calculate_scale_height_constrained() {
        // Tall sheet: 1000 x 3000
        // Scale by width: 145/1000 = 0.145
        // Scale by height: 255/3000 = 0.085
        // Should use height (smaller)
        let scale = calculate_scale(1000, 3000);
        assert!((scale - 0.085).abs() < 0.001);
    }
}
```

**Step 2: Run tests**

Run: `cargo test pdf`
Expected: All PDF tests pass

**Step 3: Commit**

```bash
git add src/output/pdf.rs
git commit -m "test: add unit tests for cutting list and scale calculation"
```

---

## Phase 5: Final Verification

### Task 11: Run Full Test Suite and Manual Verification

**Files:** None (verification only)

**Step 1: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 2: Start server for manual test**

Run: `cargo run`
Expected: Server starts on port 8080

**Step 3: Test PDF generation manually**

In another terminal, run:
```bash
curl -X POST http://localhost:8080/api/v1/optimize/quick \
  -H "Content-Type: application/json" \
  -d '{
    "job_reference": "MANUAL-TEST",
    "client_name": "Test Client",
    "pieces": [
      {"id": "A", "width": 580, "length": 418, "quantity": 4},
      {"id": "B", "width": 500, "length": 300, "quantity": 2, "edge_banding": {"top": true, "bottom": true, "left": false, "right": false, "material": "White 1mm"}}
    ],
    "stock_sheets": [
      {"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820, "thickness": 16}
    ],
    "output": {"generate_pdf": true}
  }' | jq -r '.result.pdf_base64' | base64 -d > test-output.pdf
```

Expected: `test-output.pdf` is created and can be opened in a PDF viewer

**Step 4: Verify PDF contents**
- Opens without errors
- Shows "Job Layout" title
- Shows job reference and client name
- Shows cutting list in sidebar
- Shows scaled sheet diagram with pieces
- Edge banding pieces (B) have red lines on top/bottom edges

**Step 5: Final commit**

```bash
git add -A
git commit -m "docs: complete PDF generation implementation"
```

---

## Summary

This plan implements PDF generation in 11 tasks across 5 phases:

1. **Setup** (Tasks 1-2): Add dependencies, create module structure
2. **Core PDF** (Tasks 3-6): Document creation, header, sidebar, diagram
3. **API Integration** (Tasks 7-8): Response type, handler integration
4. **Testing** (Tasks 9-10): Integration and unit tests
5. **Verification** (Task 11): Full test suite and manual verification

Each task is self-contained with clear verification steps and commits.

# PDF Generation Design for Cut Optimizer API

## Overview

Generate PDF layout diagrams from optimization results using printpdf. Each page shows one sheet layout with a scaled diagram, cutting list, and job metadata.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Scope | Single-material PDF | Matches current `OptimizeResult` structure |
| Page layout | Portrait A4, fixed regions | Matches example format, works well for standard sheets |
| Edge banding | Dashed red lines | Matches example, visually distinct |
| Piece labels | ID + dimensions | Most useful for carpenters at a glance |
| PDF library | printpdf | Pure Rust, precise control for diagrams |
| Return format | Base64 in JSON | Simple, no storage infrastructure needed |

## Architecture

### Module Structure

```
src/output/
├── mod.rs          # Module exports
└── pdf.rs          # PDF generation logic
```

### Core Function

```rust
pub fn generate_pdf(
    result: &OptimizeResult,
    job_reference: &str,
    client_name: Option<&str>,
    stock_sheet: &StockSheet,
) -> Result<Vec<u8>, PdfError>
```

### Data Flow

1. API handler calls `solve_ffdh()` → gets `OptimizeResult`
2. If `output.generate_pdf` is true, call `generate_pdf()`
3. Base64-encode the bytes and include in response
4. Client decodes and saves/displays PDF

## Page Layout

### Dimensions (A4 Portrait: 210mm × 297mm)

```
+------------------------------------------------------------------+
|                         HEADER (25mm)                             |
| Material name, size         Job Layout title                      |
| Job reference, client       Layout X of Y, statistics             |
+----------------+-------------------------------------------------+
|                |                                                  |
|    SIDEBAR     |              DIAGRAM AREA                        |
|    (55mm)      |              (145mm × 255mm)                     |
|                |                                                  |
| Cutting List   |    +----------------------------------+          |
| Id Len Wid Qty |    |                                  |          |
| A  2450 580  3 |    |    Scaled sheet layout           |          |
| B  418  580 14 |    |    with pieces                   |          |
| ...            |    |                                  |          |
|                |    +----------------------------------+          |
| Edging Legend  |                                                  |
| ---- White 1mm |                                                  |
+----------------+-------------------------------------------------+
```

### Header Region (top 25mm)

- Left: Material name, sheet dimensions (e.g., "BOARD White", "Size: 2740mm x 1820mm x 16mm")
- Center: Title "Job Layout"
- Below: Job reference, client name, date on left; Layout X of Y, occurrences, panel counts, wastage on right

### Left Sidebar (55mm wide)

- Cutting List table: Id, Length, Width, Qty columns
- Edging Legend: dashed line sample with material name

### Diagram Area (145mm × 255mm)

- Sheet rectangle scaled to fit while maintaining aspect ratio
- Sheet dimensions labeled on edges
- Pieces drawn with solid black outlines
- Piece ID centered in each piece
- Width/length dimensions on edges (if piece large enough)
- Dashed red lines on edges with banding

## Scaling System

### Constants

```rust
const DIAGRAM_X: f64 = 60.0;       // Start after sidebar
const DIAGRAM_Y: f64 = 10.0;       // Bottom margin
const DIAGRAM_WIDTH: f64 = 145.0;  // Available width
const DIAGRAM_HEIGHT: f64 = 255.0; // Available height
```

### Scale Calculation

```rust
fn calculate_scale(sheet_width: u32, sheet_length: u32) -> f64 {
    let scale_x = DIAGRAM_WIDTH / sheet_width as f64;
    let scale_y = DIAGRAM_HEIGHT / sheet_length as f64;
    scale_x.min(scale_y)  // Use smaller to fit both dimensions
}
```

For standard 2740×1820mm sheet: scale ≈ 0.053, renders at ~145mm × ~96mm

### Coordinate Transform

- printpdf origin: bottom-left of page
- Sheet origin: bottom-left of diagram area
- X transform: `x_pdf = DIAGRAM_X + (piece.x * scale)`
- Y transform: `y_pdf = DIAGRAM_Y + DIAGRAM_HEIGHT - ((piece.y + piece.length) * scale)`

## Drawing Functions

### Primitives

```rust
fn draw_rect(layer: &PdfLayerReference, x: f64, y: f64, w: f64, h: f64, stroke: Color);
fn draw_dashed_line(layer: &PdfLayerReference, x1: f64, y1: f64, x2: f64, y2: f64, color: Color);
fn draw_text_centered(layer: &PdfLayerReference, text: &str, x: f64, y: f64, font: &IndirectFontRef, size: f64);
fn draw_text(layer: &PdfLayerReference, text: &str, x: f64, y: f64, font: &IndirectFontRef, size: f64);
```

### Per-Piece Drawing

```rust
fn draw_piece(layer: &PdfLayerReference, piece: &PlacedPiece, scale: f64, fonts: &Fonts) {
    // 1. Calculate transformed coordinates
    let x = DIAGRAM_X + piece.x as f64 * scale;
    let y = /* transformed y */;
    let w = piece.width as f64 * scale;
    let h = piece.length as f64 * scale;

    // 2. Draw solid outline
    draw_rect(layer, x, y, w, h, Color::black());

    // 3. Draw piece ID centered
    draw_text_centered(layer, &piece.piece_id, x + w/2.0, y + h/2.0, &fonts.regular, 10.0);

    // 4. Draw dimensions (if piece large enough)
    if w > 15.0 {
        draw_text_centered(layer, &format!("{} mm", piece.width), x + w/2.0, y + h - 3.0, &fonts.small, 6.0);
    }
    if h > 15.0 {
        // Rotated text for vertical dimension
        draw_text_centered(layer, &format!("{} mm", piece.length), x + w - 3.0, y + h/2.0, &fonts.small, 6.0);
    }

    // 5. Draw edge banding (dashed red lines)
    if let Some(eb) = &piece.edge_banding {
        let red = Color::Rgb(Rgb::new(0.8, 0.0, 0.0, None));
        if eb.top { draw_dashed_line(layer, x, y+h, x+w, y+h, red); }
        if eb.bottom { draw_dashed_line(layer, x, y, x+w, y, red); }
        if eb.left { draw_dashed_line(layer, x, y, x, y+h, red); }
        if eb.right { draw_dashed_line(layer, x+w, y, x+w, y+h, red); }
    }
}
```

## Cutting List Generation

### Aggregation

```rust
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
            // Extract original ID (strip instance suffix)
            let original_id = piece.piece_id.rsplit_once('-')
                .map(|(prefix, _)| prefix)
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

## API Integration

### New Dependencies (Cargo.toml)

```toml
[dependencies]
printpdf = "0.7"
base64 = "0.21"
```

### Response Update

```rust
#[derive(Debug, Serialize)]
pub struct OptimizeResponse {
    pub job_reference: String,
    #[serde(flatten)]
    pub result: OptimizeResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_base64: Option<String>,
}
```

### Handler Integration

```rust
pub async fn optimize_quick(request: web::Json<OptimizeRequest>) -> HttpResponse {
    // ... validation and solve_ffdh() ...

    let pdf_base64 = if request.output.generate_pdf {
        match generate_pdf(&result, &request.job_reference, request.client_name.as_deref(), stock_sheet) {
            Ok(bytes) => Some(base64::engine::general_purpose::STANDARD.encode(&bytes)),
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
```

## Error Handling

- PDF generation failures log error but don't fail the request
- Client receives optimization result even if PDF generation fails
- `PdfError` enum covers: font loading, page creation, IO errors

## Testing Strategy

1. Unit tests for `build_cutting_list()` aggregation
2. Unit tests for `calculate_scale()` with various sheet sizes
3. Integration test: request with `generate_pdf: true`, verify response contains `pdf_base64`
4. Manual verification: decode base64, open PDF, visual inspection

use thiserror::Error;
use printpdf::*;
use std::io::BufWriter;
use std::collections::HashMap;

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

// Layout regions (in mm from bottom-left origin)
const HEADER_Y: f32 = 272.0;       // Header starts here (297 - 25)
const SIDEBAR_WIDTH: f32 = 55.0;
const DIAGRAM_X: f32 = 60.0;
const DIAGRAM_Y: f32 = 10.0;
const DIAGRAM_WIDTH: f32 = 145.0;
const DIAGRAM_HEIGHT: f32 = 255.0;

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

fn calculate_scale(sheet_width: u32, sheet_length: u32) -> f32 {
    let scale_x = DIAGRAM_WIDTH / sheet_width as f32;
    let scale_y = DIAGRAM_HEIGHT / sheet_length as f32;
    scale_x.min(scale_y)
}

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

    // Draw sheet outline (stroked, not filled)
    layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    layer.set_outline_thickness(1.0);

    // Draw sheet as lines to get stroke-only rectangle
    let sheet_outline = Line {
        points: vec![
            (Point::new(Mm(offset_x), Mm(offset_y)), false),
            (Point::new(Mm(offset_x + diagram_sheet_width), Mm(offset_y)), false),
            (Point::new(Mm(offset_x + diagram_sheet_width), Mm(offset_y + diagram_sheet_height)), false),
            (Point::new(Mm(offset_x), Mm(offset_y + diagram_sheet_height)), false),
        ],
        is_closed: true,
    };
    layer.add_line(sheet_outline);

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

    // Draw piece rectangle (stroked, not filled)
    layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    layer.set_outline_thickness(0.5);

    // Draw piece as lines to get stroke-only rectangle
    let piece_outline = Line {
        points: vec![
            (Point::new(Mm(x), Mm(y)), false),
            (Point::new(Mm(x + w), Mm(y)), false),
            (Point::new(Mm(x + w), Mm(y + h)), false),
            (Point::new(Mm(x), Mm(y + h)), false),
        ],
        is_closed: true,
    };
    layer.add_line(piece_outline);

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

    let font = doc.add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| PdfError::FontError(e.to_string()))?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| PdfError::FontError(e.to_string()))?;

    // Build cutting list once for all pages
    let cutting_list = build_cutting_list(&result.layouts);

    // Draw first page
    if let Some(layout) = result.layouts.first() {
        let current_layer = doc.get_page(page1).get_layer(layer1);
        draw_header(
            &current_layer, &font, &font_bold,
            job_reference, client_name, stock_sheet,
            0, result.layouts.len(), layout,
            result.total_pieces, result.waste_percentage,
        );
        draw_sidebar(&current_layer, &font, &font_bold, &cutting_list);
        draw_diagram(&current_layer, &font, layout, stock_sheet);
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
        draw_sidebar(&current_layer, &font, &font_bold, &cutting_list);
        draw_diagram(&current_layer, &font, layout, stock_sheet);
    }

    let mut buffer = BufWriter::new(Vec::new());
    doc.save(&mut buffer)
        .map_err(|e| PdfError::SaveError(e.to_string()))?;

    buffer.into_inner()
        .map_err(|e| PdfError::SaveError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::{PlacedPiece, SheetLayout};

    #[test]
    fn test_build_cutting_list_single_piece_type() {
        let layouts = vec![SheetLayout {
            sheet_index: 0,
            stock_sheet_id: "sheet-1".to_string(),
            stock_sheet_name: "BOARD White".to_string(),
            width: 2740,
            length: 1820,
            pieces: vec![
                PlacedPiece {
                    piece_id: "panel-a-0".to_string(),
                    label: None,
                    x: 0,
                    y: 0,
                    width: 580,
                    length: 418,
                    rotated: false,
                    edge_banding: None,
                },
                PlacedPiece {
                    piece_id: "panel-a-1".to_string(),
                    label: None,
                    x: 584,
                    y: 0,
                    width: 580,
                    length: 418,
                    rotated: false,
                    edge_banding: None,
                },
            ],
            used_area: 484880,
            waste_percentage: 90.3,
        }];

        let list = build_cutting_list(&layouts);

        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "panel-a");
        assert_eq!(list[0].quantity, 2);
        assert_eq!(list[0].width, 580);
        assert_eq!(list[0].length, 418);
    }

    #[test]
    fn test_build_cutting_list_multiple_piece_types() {
        let layouts = vec![SheetLayout {
            sheet_index: 0,
            stock_sheet_id: "sheet-1".to_string(),
            stock_sheet_name: "BOARD White".to_string(),
            width: 2740,
            length: 1820,
            pieces: vec![
                PlacedPiece {
                    piece_id: "panel-a-0".to_string(),
                    label: None,
                    x: 0,
                    y: 0,
                    width: 580,
                    length: 418,
                    rotated: false,
                    edge_banding: None,
                },
                PlacedPiece {
                    piece_id: "panel-b-0".to_string(),
                    label: None,
                    x: 584,
                    y: 0,
                    width: 300,
                    length: 200,
                    rotated: false,
                    edge_banding: None,
                },
                PlacedPiece {
                    piece_id: "panel-a-1".to_string(),
                    label: None,
                    x: 0,
                    y: 422,
                    width: 580,
                    length: 418,
                    rotated: false,
                    edge_banding: None,
                },
            ],
            used_area: 545080,
            waste_percentage: 89.1,
        }];

        let list = build_cutting_list(&layouts);

        assert_eq!(list.len(), 2);
        // Sorted alphabetically
        assert_eq!(list[0].id, "panel-a");
        assert_eq!(list[0].quantity, 2);
        assert_eq!(list[1].id, "panel-b");
        assert_eq!(list[1].quantity, 1);
    }

    #[test]
    fn test_build_cutting_list_across_multiple_layouts() {
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
                        x: 0,
                        y: 0,
                        width: 580,
                        length: 418,
                        rotated: false,
                        edge_banding: None,
                    },
                ],
                used_area: 242440,
                waste_percentage: 95.1,
            },
            SheetLayout {
                sheet_index: 1,
                stock_sheet_id: "sheet-1".to_string(),
                stock_sheet_name: "BOARD White".to_string(),
                width: 2740,
                length: 1820,
                pieces: vec![
                    PlacedPiece {
                        piece_id: "panel-a-1".to_string(),
                        label: None,
                        x: 0,
                        y: 0,
                        width: 580,
                        length: 418,
                        rotated: false,
                        edge_banding: None,
                    },
                    PlacedPiece {
                        piece_id: "panel-a-2".to_string(),
                        label: None,
                        x: 584,
                        y: 0,
                        width: 580,
                        length: 418,
                        rotated: false,
                        edge_banding: None,
                    },
                ],
                used_area: 484880,
                waste_percentage: 90.3,
            },
        ];

        let list = build_cutting_list(&layouts);

        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "panel-a");
        assert_eq!(list[0].quantity, 3);
    }

    #[test]
    fn test_build_cutting_list_piece_id_without_suffix() {
        let layouts = vec![SheetLayout {
            sheet_index: 0,
            stock_sheet_id: "sheet-1".to_string(),
            stock_sheet_name: "BOARD White".to_string(),
            width: 2740,
            length: 1820,
            pieces: vec![
                PlacedPiece {
                    piece_id: "custom-piece".to_string(), // No numeric suffix
                    label: None,
                    x: 0,
                    y: 0,
                    width: 580,
                    length: 418,
                    rotated: false,
                    edge_banding: None,
                },
            ],
            used_area: 242440,
            waste_percentage: 95.1,
        }];

        let list = build_cutting_list(&layouts);

        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "custom-piece");
        assert_eq!(list[0].quantity, 1);
    }

    #[test]
    fn test_calculate_scale_wide_sheet() {
        // Wide sheet: 2740mm x 1820mm
        let scale = calculate_scale(2740, 1820);
        // Should be limited by width
        let expected = DIAGRAM_WIDTH / 2740.0;
        assert!((scale - expected).abs() < 0.001);
    }

    #[test]
    fn test_calculate_scale_tall_sheet() {
        // Tall sheet: 1000mm x 3000mm
        let scale = calculate_scale(1000, 3000);
        // Should be limited by height
        let expected = DIAGRAM_HEIGHT / 3000.0;
        assert!((scale - expected).abs() < 0.001);
    }

    #[test]
    fn test_calculate_scale_square_sheet() {
        // Square sheet: 1000mm x 1000mm
        let scale = calculate_scale(1000, 1000);
        // Should use the smaller scale factor
        let scale_x = DIAGRAM_WIDTH / 1000.0;
        let scale_y = DIAGRAM_HEIGHT / 1000.0;
        let expected = scale_x.min(scale_y);
        assert!((scale - expected).abs() < 0.001);
    }
}

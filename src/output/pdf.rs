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

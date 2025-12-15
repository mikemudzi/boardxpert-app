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

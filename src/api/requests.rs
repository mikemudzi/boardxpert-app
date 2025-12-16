use serde::{Deserialize, Serialize};
use crate::optimizer::{CutPiece, StockSheet, CutParameters};

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Default, Deserialize, Serialize)]
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

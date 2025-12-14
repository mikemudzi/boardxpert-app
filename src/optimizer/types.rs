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

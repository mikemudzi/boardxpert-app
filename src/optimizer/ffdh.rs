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
}

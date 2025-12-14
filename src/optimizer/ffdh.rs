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

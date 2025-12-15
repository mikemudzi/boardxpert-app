use crate::api::requests::OptimizeRequest;

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

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
    // TODO: Support multiple stock sheet types
    let stock_sheet = &request.stock_sheets[0];

    let result = solve_ffdh(
        &request.pieces,
        stock_sheet,
        request.parameters.blade_kerf,
    );

    // Generate PDF if requested
    let pdf_base64 = if request.output.generate_pdf {
        match generate_pdf(
            &result,
            &request.job_reference,
            request.client_name.as_deref(),
            stock_sheet,
        ) {
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

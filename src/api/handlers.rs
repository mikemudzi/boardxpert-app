use actix_web::{web, HttpResponse};
use base64::{Engine as _, engine::general_purpose};
use uuid::Uuid;
use crate::api::{
    OptimizeRequest, ApiResponse, OptimizeResponse, AsyncJobResponse,
    JobStatusResponse, AppState, validate_request
};
use crate::db;
use crate::queue;
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

/// POST /api/v1/optimize/async - Submit job for async processing
pub async fn optimize_async(
    state: web::Data<AppState>,
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

    // Serialize request to JSON for storage
    let request_json = match serde_json::to_value(&*request) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("Failed to serialize request: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("SERIALIZATION_ERROR", "Failed to serialize request"));
        }
    };

    // Create job in database
    let job_id = match db::create_job(
        &state.db_pool,
        &request.job_reference,
        request.client_name.as_deref(),
        &request_json,
        request.webhook_url.as_deref(),
    ).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to create job: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", "Failed to create job"));
        }
    };

    // Push job to Redis queue
    {
        let mut conn = state.redis_conn.lock().await;
        if let Err(e) = queue::push_job(&mut conn, job_id).await {
            tracing::error!("Failed to queue job: {}", e);
            // Job was created but not queued - mark as failed
            let _ = db::fail_job(&state.db_pool, job_id, "Failed to queue job").await;
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("QUEUE_ERROR", "Failed to queue job"));
        }
    }

    let response = AsyncJobResponse {
        job_id,
        status: "pending".to_string(),
    };

    HttpResponse::Accepted().json(ApiResponse::success(response))
}

/// GET /api/v1/jobs/{job_id} - Get job status
pub async fn get_job_status(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let job_id = path.into_inner();

    let job = match db::get_job(&state.db_pool, job_id).await {
        Ok(Some(job)) => job,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("NOT_FOUND", "Job not found"));
        }
        Err(e) => {
            tracing::error!("Failed to fetch job: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("DATABASE_ERROR", "Failed to fetch job"));
        }
    };

    // Parse result JSON if present
    let result = job.result.and_then(|r| {
        serde_json::from_value(r).ok()
    });

    // Encode PDF as base64 if present
    let pdf_base64 = job.pdf_bytes.map(|bytes| {
        general_purpose::STANDARD.encode(&bytes)
    });

    // Get client_name from request JSON
    let client_name = job.request.get("client_name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let response = JobStatusResponse {
        job_id: job.id,
        status: job.status,
        job_reference: job.job_reference,
        client_name,
        result,
        pdf_base64,
        error_message: job.error_message,
        created_at: job.created_at,
        started_at: job.started_at,
        completed_at: job.completed_at,
    };

    HttpResponse::Ok().json(ApiResponse::success(response))
}

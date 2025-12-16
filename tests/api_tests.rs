use actix_web::{test, web, App};
use cut_optimizer_api::api;

#[actix_rt::test]
async fn test_health_endpoint() {
    let app = test::init_service(
        App::new()
            .route("/health", web::get().to(|| async {
                actix_web::HttpResponse::Ok().json(serde_json::json!({
                    "status": "healthy"
                }))
            }))
    ).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_templates_endpoint() {
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/templates")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["result"].as_array().unwrap().len() > 0);
}

#[actix_rt::test]
async fn test_validate_valid_request() {
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/validate")
        .set_json(serde_json::json!({
            "job_reference": "TEST-001",
            "pieces": [{"id": "panel-a", "width": 580, "length": 418, "quantity": 1}],
            "stock_sheets": [{"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820}]
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_validate_rejects_empty_pieces() {
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/validate")
        .set_json(serde_json::json!({
            "job_reference": "TEST-001",
            "pieces": [],
            "stock_sheets": [{"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820}]
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 400);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(!body["success"].as_bool().unwrap());
    assert_eq!(body["error"]["code"], "NO_PIECES");
}

#[actix_rt::test]
async fn test_optimize_quick_basic() {
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/optimize/quick")
        .set_json(serde_json::json!({
            "job_reference": "TEST-001",
            "pieces": [
                {"id": "panel-a", "width": 580, "length": 418, "quantity": 4}
            ],
            "stock_sheets": [
                {"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820}
            ]
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["result"]["job_reference"], "TEST-001");
    assert_eq!(body["result"]["total_pieces"], 4);
    assert!(body["result"]["total_sheets"].as_u64().unwrap() >= 1);
}

#[actix_rt::test]
async fn test_optimize_quick_with_edge_banding() {
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/optimize/quick")
        .set_json(serde_json::json!({
            "job_reference": "TEST-002",
            "pieces": [
                {
                    "id": "panel-a",
                    "width": 580,
                    "length": 418,
                    "quantity": 2,
                    "edge_banding": {
                        "top": true,
                        "bottom": true,
                        "left": false,
                        "right": false,
                        "material": "White 1mm"
                    }
                }
            ],
            "stock_sheets": [
                {"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820}
            ]
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());

    // Check that edge banding is preserved in output
    let pieces = &body["result"]["layouts"][0]["pieces"];
    assert!(pieces[0]["edge_banding"].is_object());
}

#[actix_rt::test]
async fn test_optimize_quick_with_pdf_generation() {
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/optimize/quick")
        .set_json(serde_json::json!({
            "job_reference": "PDF-TEST-001",
            "client_name": "Test Client",
            "pieces": [
                {
                    "id": "panel-a",
                    "width": 580,
                    "length": 418,
                    "quantity": 3,
                    "edge_banding": {
                        "top": true,
                        "bottom": false,
                        "left": false,
                        "right": true,
                        "material": "White 1mm"
                    }
                },
                {
                    "id": "panel-b",
                    "width": 300,
                    "length": 200,
                    "quantity": 2
                }
            ],
            "stock_sheets": [
                {"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820, "thickness": 16}
            ],
            "output": {
                "generate_pdf": true
            }
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["result"]["job_reference"], "PDF-TEST-001");

    // Verify PDF was generated
    let pdf_base64 = body["result"]["pdf_base64"].as_str();
    assert!(pdf_base64.is_some(), "pdf_base64 should be present");

    let pdf_data = pdf_base64.unwrap();
    assert!(!pdf_data.is_empty(), "pdf_base64 should not be empty");

    // Verify it's valid base64 by decoding
    use base64::{Engine as _, engine::general_purpose};
    let decoded = general_purpose::STANDARD.decode(pdf_data);
    assert!(decoded.is_ok(), "pdf_base64 should be valid base64");

    let pdf_bytes = decoded.unwrap();
    // PDF files start with %PDF-
    assert!(pdf_bytes.starts_with(b"%PDF-"), "Decoded data should be a valid PDF");
}

#[actix_rt::test]
async fn test_optimize_quick_without_pdf_generation() {
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/optimize/quick")
        .set_json(serde_json::json!({
            "job_reference": "NO-PDF-TEST",
            "pieces": [
                {"id": "panel-a", "width": 580, "length": 418, "quantity": 2}
            ],
            "stock_sheets": [
                {"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820}
            ],
            "output": {
                "generate_pdf": false
            }
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());

    // Verify PDF was NOT generated (field should be null or missing)
    assert!(body["result"]["pdf_base64"].is_null(), "pdf_base64 should be null when not requested");
}

// Note: Async job tests require PostgreSQL and Redis infrastructure.
// These tests verify request/response structure without actual database.

#[actix_rt::test]
async fn test_async_optimize_without_appstate_returns_error() {
    // When AppState is not configured, async endpoints should fail gracefully
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/v1/optimize/async")
        .set_json(serde_json::json!({
            "job_reference": "ASYNC-TEST-001",
            "pieces": [
                {"id": "panel-a", "width": 580, "length": 418, "quantity": 2}
            ],
            "stock_sheets": [
                {"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820}
            ]
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Without AppState, this should return 500 Internal Server Error
    // because the handler expects app_data<AppState>
    assert_eq!(resp.status(), 500);
}

#[actix_rt::test]
async fn test_async_optimize_validates_request_via_sync() {
    // Test validation logic using optimize/quick (async uses same validation)
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    // Invalid request - no pieces (test via sync endpoint since async needs AppState)
    let req = test::TestRequest::post()
        .uri("/api/v1/optimize/quick")
        .set_json(serde_json::json!({
            "job_reference": "ASYNC-TEST-002",
            "pieces": [],
            "stock_sheets": [
                {"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820}
            ]
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Should return 400 Bad Request for validation error
    assert_eq!(resp.status(), 400);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(!body["success"].as_bool().unwrap());
    assert_eq!(body["error"]["code"], "NO_PIECES");
}

#[actix_rt::test]
async fn test_job_status_without_appstate_returns_error() {
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/jobs/550e8400-e29b-41d4-a716-446655440000")
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Without AppState, this should return 500 Internal Server Error
    assert_eq!(resp.status(), 500);
}

#[actix_rt::test]
async fn test_request_with_webhook_url() {
    // Test that webhook_url is accepted in requests
    let app = test::init_service(
        App::new().configure(api::routes::configure)
    ).await;

    // Use optimize/quick which doesn't need AppState
    let req = test::TestRequest::post()
        .uri("/api/v1/optimize/quick")
        .set_json(serde_json::json!({
            "job_reference": "WEBHOOK-TEST",
            "pieces": [
                {"id": "panel-a", "width": 580, "length": 418, "quantity": 1}
            ],
            "stock_sheets": [
                {"id": "sheet-1", "name": "BOARD White", "width": 2740, "length": 1820}
            ],
            "webhook_url": "https://example.com/webhook"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Should succeed - webhook_url is accepted but ignored for sync requests
    assert!(resp.status().is_success());
}

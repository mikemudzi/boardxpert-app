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

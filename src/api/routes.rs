use actix_web::web;
use super::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/validate", web::post().to(handlers::validate))
            .route("/optimize/quick", web::post().to(handlers::optimize_quick))
            .route("/optimize/async", web::post().to(handlers::optimize_async))
            .route("/jobs/{job_id}", web::get().to(handlers::get_job_status))
            .route("/templates", web::get().to(handlers::get_templates))
    );
}

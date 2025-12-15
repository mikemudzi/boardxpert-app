use actix_web::web;
use super::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/validate", web::post().to(handlers::validate))
            .route("/optimize/quick", web::post().to(handlers::optimize_quick))
            .route("/templates", web::get().to(handlers::get_templates))
    );
}

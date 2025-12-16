use actix_web::{web, App, HttpServer, HttpResponse};
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod cli;
mod optimizer;
mod output;

use cli::Cli;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Cli::parse();

    if args.worker {
        tracing::info!("Starting worker with concurrency {}", args.concurrency);
        // TODO: Implement worker loop
        Ok(())
    } else {
        run_api_server().await
    }
}

async fn run_api_server() -> std::io::Result<()> {
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()
        .expect("PORT must be a number");

    tracing::info!("Starting Cut Optimizer API at {}:{}", host, port);

    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health_check))
            .configure(api::routes::configure)
    })
    .bind((host, port))?
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

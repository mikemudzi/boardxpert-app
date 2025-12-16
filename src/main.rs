use actix_web::{web, App, HttpServer, HttpResponse};
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod cli;
mod db;
mod optimizer;
mod output;
mod queue;
mod worker;

use api::AppState;
use cli::Cli;
use worker::Worker;

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
        run_worker(args.concurrency).await
    } else {
        run_api_server().await
    }
}

async fn run_worker(concurrency: usize) -> std::io::Result<()> {
    tracing::info!("Starting worker with concurrency {}", concurrency);

    // Initialize database pool
    let db_pool = db::create_pool().await
        .expect("Failed to create database pool");

    // Initialize Redis connection
    let redis_conn = queue::create_client().await
        .expect("Failed to create Redis connection");

    let worker = Worker::new(db_pool, redis_conn);

    // Run worker (this blocks forever)
    worker.run().await;

    Ok(())
}

async fn run_api_server() -> std::io::Result<()> {
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()
        .expect("PORT must be a number");

    // Check if async mode is enabled (DATABASE_URL set)
    let async_enabled = std::env::var("DATABASE_URL").is_ok();

    if async_enabled {
        run_api_server_with_async(host, port).await
    } else {
        run_api_server_sync_only(host, port).await
    }
}

async fn run_api_server_with_async(host: String, port: u16) -> std::io::Result<()> {
    tracing::info!("Starting Cut Optimizer API at {}:{} (async mode enabled)", host, port);

    // Initialize database pool
    let db_pool = db::create_pool().await
        .expect("Failed to create database pool");

    // Initialize Redis connection
    let redis_conn = queue::create_client().await
        .expect("Failed to create Redis connection");

    let app_state = AppState::new(db_pool, redis_conn);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/health", web::get().to(health_check))
            .configure(api::routes::configure)
    })
    .bind((host, port))?
    .run()
    .await
}

async fn run_api_server_sync_only(host: String, port: u16) -> std::io::Result<()> {
    tracing::info!("Starting Cut Optimizer API at {}:{} (sync-only mode)", host, port);
    tracing::warn!("DATABASE_URL not set - async endpoints will return 503");

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

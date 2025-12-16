use redis::aio::ConnectionManager;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared application state for API handlers
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub redis_conn: Arc<Mutex<ConnectionManager>>,
}

impl AppState {
    pub fn new(db_pool: PgPool, redis_conn: ConnectionManager) -> Self {
        Self {
            db_pool,
            redis_conn: Arc::new(Mutex::new(redis_conn)),
        }
    }
}

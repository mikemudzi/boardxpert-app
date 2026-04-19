use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use uuid::Uuid;

const QUEUE_KEY: &str = "cut_optimizer:jobs";

pub async fn create_client() -> Result<ConnectionManager, redis::RedisError> {
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".into());

    let client = redis::Client::open(redis_url)?;
    ConnectionManager::new(client).await
}

pub async fn push_job(conn: &mut ConnectionManager, job_id: Uuid) -> Result<(), redis::RedisError> {
    conn.lpush(QUEUE_KEY, job_id.to_string()).await
}

pub async fn pop_job(conn: &mut ConnectionManager, timeout_secs: usize) -> Result<Option<Uuid>, redis::RedisError> {
    let result: Option<(String, String)> = conn.brpop(QUEUE_KEY, timeout_secs as f64).await?;

    match result {
        Some((_, job_id_str)) => {
            let job_id = Uuid::parse_str(&job_id_str)
                .map_err(|e| redis::RedisError::from((
                    redis::ErrorKind::TypeError,
                    "Invalid UUID",
                    e.to_string()
                )))?;
            Ok(Some(job_id))
        }
        None => Ok(None),
    }
}

pub async fn queue_length(conn: &mut ConnectionManager) -> Result<usize, redis::RedisError> {
    conn.llen(QUEUE_KEY).await
}

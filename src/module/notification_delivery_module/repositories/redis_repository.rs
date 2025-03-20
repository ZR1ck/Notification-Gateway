use std::sync::Arc;

use deadpool_redis::{Pool, PoolError};
use log::info;
use redis::AsyncCommands;

pub struct RedisRepository {
    pub pool: Arc<Pool>,
}

impl RedisRepository {
    pub fn new(pool: Arc<Pool>) -> Self {
        Self { pool }
    }

    pub async fn push_to_queue(&self, key: &str, value: &str) -> Result<(), PoolError> {
        let mut redis_conn = self.pool.get().await?;
        let _: () = redis_conn.lpush(key, value).await?;
        info!("Redis push to: {}", key);
        Ok(())
    }
}

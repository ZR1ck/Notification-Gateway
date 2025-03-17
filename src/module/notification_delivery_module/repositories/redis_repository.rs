use std::sync::Arc;

use deadpool_redis::Pool;

pub struct RedisRepository {
    pub pool: Arc<Pool>,
}

impl RedisRepository {
    pub fn new(pool: Arc<Pool>) -> Self {
        Self { pool }
    }
}

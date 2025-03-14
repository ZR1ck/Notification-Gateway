use std::env;

use deadpool_redis::{Config, Pool, Runtime};

pub fn create_redis_pool() -> Pool {
    let redis_url = env::var("REDIS_URL").expect("REDIS URL must be set");

    Config::from_url(redis_url)
        .create_pool(Some(Runtime::Tokio1))
        .expect("Cannot create redis pool")
}

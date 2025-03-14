use std::env;

use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn create_database_pool() -> PgPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Cannot create database pool")
}

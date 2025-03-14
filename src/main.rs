use actix_web::{App, HttpServer};
use config::{database::create_database_pool, redis::create_redis_pool};
use dotenvy::dotenv;
use env_logger::Env;
use log::info;
use sqlx::migrate;

mod config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // init dotenv
    dotenv().ok();

    // init logger
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // create redis pool
    let redis_pool = create_redis_pool();
    info!("Redis pool created");

    // create postgres pool
    let pg_pool = create_database_pool().await;
    info!("Pg pool created");

    // run migrations
    let migrator = migrate::Migrator::new(std::path::Path::new("./migrations"))
        .await
        .expect("Cannot create migrator");
    migrator.run(&pg_pool).await.expect("Migration failed");
    info!("Migration success");

    info!("Starting server...");

    HttpServer::new(|| App::new())
        .workers(1)
        .bind(("localhost", 8080))?
        .run()
        .await
}

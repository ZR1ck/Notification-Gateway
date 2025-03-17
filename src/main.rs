use std::sync::Arc;

use actix::Actor;
use actix_web::{web, App, HttpServer};
use config::{database::create_database_pool, redis::create_redis_pool};
use dotenvy::dotenv;
use env_logger::Env;
use log::info;
use module::{
    notification_delivery_module::NotiDelivModule, notification_service_module::NotiServiceModule,
};
use sqlx::migrate;

mod config;
mod module;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // init dotenv
    dotenv().ok();

    // init logger
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // create redis pool
    let redis_pool = Arc::new(create_redis_pool());
    info!("Redis pool created");

    // create postgres pool
    let pg_pool = Arc::new(create_database_pool().await);
    info!("Pg pool created");

    // run migrations
    let migrator = migrate::Migrator::new(std::path::Path::new("./migrations"))
        .await
        .expect("Cannot create migrator");
    migrator.run(&*pg_pool).await.expect("Migration failed");
    info!("Migration success");

    // init modules
    let noti_srv_module = NotiServiceModule::new(pg_pool.clone(), redis_pool.clone());
    let noti_deliv_module = NotiDelivModule::new(pg_pool.clone(), redis_pool.clone());
    let worker_addr = noti_deliv_module.queue_worker.start();

    info!("Starting server...");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(noti_srv_module.noti_controller.clone()))
            .app_data(web::Data::new(
                noti_deliv_module.queue_worker_controller.clone(),
            ))
            .app_data(web::Data::new(worker_addr.clone()))
            .configure(NotiServiceModule::routes_config)
            .configure(NotiDelivModule::routes_config)
    })
    .workers(1)
    .bind(("localhost", 8080))?
    .run()
    .await
}

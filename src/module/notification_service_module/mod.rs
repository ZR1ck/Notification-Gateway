use std::sync::Arc;

use actix_web::web;
use controllers::notification_controller::NotificationController;
use deadpool_redis::Pool;
use repository::{notification_repository::NotificationRepo, redis_repository::RedisRepository};
use services::notification_service::NotificationService;
use sqlx::PgPool;

pub mod controllers;
pub mod errors;
pub mod models;
pub mod repository;
pub mod services;

pub struct NotiServiceModule {
    pub noti_controller: Arc<NotificationController>,
}

impl NotiServiceModule {
    pub fn new(pg_pool: Arc<PgPool>, redis_pool: Arc<Pool>) -> Self {
        // init repositories
        let noti_repo = Arc::new(NotificationRepo::new(pg_pool));
        let redis_repo = Arc::new(RedisRepository::new(redis_pool));

        // init services
        let noti_service = Arc::new(NotificationService::new(
            noti_repo.clone(),
            redis_repo.clone(),
        ));

        // init controllers
        let noti_controller = NotificationController::new(noti_service.clone());

        // generate module
        Self {
            noti_controller: Arc::new(noti_controller),
        }
    }

    pub fn routes_config(cfg: &mut web::ServiceConfig) {
        NotificationController::routes(cfg);
    }
}

use std::sync::Arc;

use actix_web::web;
use controllers::worker_controller::WorkerController;
use deadpool_redis::Pool;
use repositories::{notification_repository::NotificationRepo, redis_repository::RedisRepository};
use sqlx::PgPool;
use workers::queue_worker::QueueWorker;

pub mod controllers;
pub mod errors;
pub mod models;
pub mod repositories;
pub mod services;
pub mod workers;

pub struct NotiDelivModule {
    pub queue_worker_controller: Arc<WorkerController>,
    pub queue_worker: QueueWorker,
}

impl NotiDelivModule {
    pub fn new(pg_pool: Arc<PgPool>, redis_pool: Arc<Pool>) -> Self {
        // init repositories
        let noti_repo = Arc::new(NotificationRepo::new(pg_pool));
        let redis_repo = Arc::new(RedisRepository::new(redis_pool));

        // init services
        // let noti_service = Arc::new(NotificationService::new(
        //     noti_repo.clone(),
        //     redis_repo.clone(),
        // ));
        let queue_worker = QueueWorker::new(redis_repo.clone(), noti_repo.clone());

        // init controllers
        let queue_worker_controller = WorkerController;

        // generate module
        Self {
            queue_worker_controller: Arc::new(queue_worker_controller),
            queue_worker: queue_worker,
        }
    }

    pub fn routes_config(cfg: &mut web::ServiceConfig) {
        WorkerController::routes(cfg);
    }
}

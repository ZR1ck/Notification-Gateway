use std::{collections::HashMap, sync::Arc};

use actix::{Actor, Addr, Recipient};
use actix_web::web;
use deadpool_redis::Pool;
use repositories::{notification_repository::NotificationRepo, redis_repository::RedisRepository};
use sqlx::PgPool;
use utils::fcm_token_manager::TokenManager;
use workers::queue_worker::{
    email_worker::EmailWorker, notification_worker_actor::NotificationWorkerActor,
    push_worker::PushWorker, NotificationMessage, QueueWorker,
};

pub mod controllers;
pub mod errors;
pub mod models;
pub mod repositories;
pub mod services;
pub mod utils;
pub mod workers;

pub struct NotiDelivModule {
    pub queue_worker_addr: Addr<QueueWorker>,
}

impl NotiDelivModule {
    pub async fn new(pg_pool: Arc<PgPool>, redis_pool: Arc<Pool>) -> Self {
        // init repositories
        let noti_repo = Arc::new(NotificationRepo::new(pg_pool));
        let redis_repo = Arc::new(RedisRepository::new(redis_pool));

        // init services
        // let noti_service = Arc::new(NotificationService::new(
        //     noti_repo.clone(),
        //     redis_repo.clone(),
        // ));

        let token_manager = TokenManager::new().await;

        let push_worker = NotificationWorkerActor::new(
            Arc::new(PushWorker::new(token_manager).await),
            noti_repo.clone(),
            redis_repo.clone(),
        )
        .start();
        let email_worker = NotificationWorkerActor::new(
            Arc::new(EmailWorker::new()),
            noti_repo.clone(),
            redis_repo.clone(),
        )
        .start();

        let mut workers: HashMap<String, Recipient<NotificationMessage>> = HashMap::new();

        workers.insert("push".to_string(), push_worker.recipient());
        workers.insert("email".to_string(), email_worker.recipient());

        let queue_worker = QueueWorker::new(redis_repo.clone(), noti_repo.clone(), workers);
        let queue_worker_addr = queue_worker.start();

        // init controllers

        // generate module
        Self { queue_worker_addr }
    }

    pub fn routes_config(_cfg: &mut web::ServiceConfig) {}
}

use std::sync::Arc;

use actix_web::web;
use controllers::notification_controller::NotificationController;
use repository::notification_repository::NotificationRepo;
use services::notification_service::NotificationService;
use sqlx::PgPool;

pub mod controllers;
pub mod models;
pub mod repository;
pub mod repository_trait;
pub mod services;

pub struct NotiServiceModule {
    pub noti_controller: Arc<NotificationController>,
}

impl NotiServiceModule {
    pub fn new(pool: Arc<PgPool>) -> Self {
        let noti_repo = NotificationRepo::new(pool);

        let noti_service = NotificationService::new(Arc::new(noti_repo));

        let noti_controller = NotificationController::new(Arc::new(noti_service));

        Self {
            noti_controller: Arc::new(noti_controller),
        }
    }

    pub fn routes_config(cfg: &mut web::ServiceConfig) {
        NotificationController::routes(cfg);
    }
}

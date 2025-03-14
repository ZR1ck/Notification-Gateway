use std::sync::Arc;

use sqlx::PgPool;

use crate::module::notification_service_module::repository_trait::notification::NotificationRepository;

pub struct NotificationRepo {
    pool: Arc<PgPool>,
}

impl NotificationRepo {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

impl NotificationRepository for NotificationRepo {}

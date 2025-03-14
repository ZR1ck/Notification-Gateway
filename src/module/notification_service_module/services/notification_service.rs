use std::sync::Arc;

use crate::module::notification_service_module::repository::notification_repository::NotificationRepo;

pub struct NotificationService {
    noti_repo: Arc<NotificationRepo>,
}

impl NotificationService {
    pub fn new(noti_repo: Arc<NotificationRepo>) -> Self {
        Self { noti_repo }
    }
}

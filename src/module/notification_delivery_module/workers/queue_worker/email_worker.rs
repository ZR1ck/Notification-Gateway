use std::sync::Arc;

use async_trait::async_trait;
use log::info;

use crate::module::notification_delivery_module::{
    errors::NotiDeliverError, models::notification::NotificationDeQueue,
    repositories::notification_repository::NotificationRepo,
};

use super::notification_worker_actor::NotificationWorker;

pub struct EmailWorker;

impl EmailWorker {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl NotificationWorker for EmailWorker {
    async fn send(
        &self,
        _notification: &NotificationDeQueue,
        _repo: Arc<NotificationRepo>,
    ) -> Result<(), NotiDeliverError> {
        info!("Email Sent: {}", 1);
        Ok(())
    }
}

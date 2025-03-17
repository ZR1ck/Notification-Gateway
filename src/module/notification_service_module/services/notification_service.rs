use std::{env, sync::Arc};

use log::error;

use crate::module::notification_service_module::{
    errors::NotiSrvError,
    models::notification::{NotificationEnQueue, NotificationRequest, NotificationResponse},
    repository::{notification_repository::NotificationRepo, redis_repository::RedisRepository},
};

pub struct NotificationService {
    noti_repo: Arc<NotificationRepo>,
    redis_repo: Arc<RedisRepository>,
}

impl NotificationService {
    pub fn new(noti_repo: Arc<NotificationRepo>, redis_repo: Arc<RedisRepository>) -> Self {
        Self {
            noti_repo,
            redis_repo,
        }
    }

    pub async fn send(
        &self,
        notification_request: NotificationRequest,
    ) -> Result<NotificationResponse, NotiSrvError> {
        // save notification into database
        let noti_id = self
            .noti_repo
            .insert(&notification_request)
            .await
            .map_err(|e| {
                error!("Database insert error: {}", e.to_string());
                NotiSrvError::DatabaseError(e)
            })?;

        // generate value
        let enqueue_value = NotificationEnQueue {
            notification_id: noti_id.clone(),
            recipient: notification_request.recipient,
            channel: notification_request.channel,
            template_id: notification_request.template_id.unwrap_or_default(),
            data: notification_request.data,
        };

        let job = serde_json::json!(enqueue_value);

        let queue_key = env::var("QUEUE_KEY").map_err(|e| {
            error!("Missing env: {}", e);
            NotiSrvError::MissingEnvError(e)
        })?;

        // push job into redis queue
        let _push_result = self
            .redis_repo
            .push_to_queue(&queue_key, &job.to_string())
            .await
            .map_err(|e| {
                error!("Redis push error: {}", e.to_string());
                NotiSrvError::RedisQueuePushError(e)
            })?;

        let response = NotificationResponse {
            id: noti_id,
            status: "queued".to_string(),
        };

        Ok(response)
    }
}

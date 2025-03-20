use std::{env, sync::Arc};

use log::error;
use serde_json::Value;

use crate::module::notification_service_module::{
    errors::NotiSrvError,
    models::{
        notification::{
            NotificationChannel, NotificationEnQueue, NotificationRequest, NotificationResponse,
        },
        payload::{EmailPayload, Payload, PushPayload},
    },
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
        // Validate payload
        if !self.validate_payload(&notification_request.channel, &notification_request.payload) {
            return Err(NotiSrvError::InvalidDataField);
        }

        let recipient_type = notification_request
            .recipient_type
            .clone()
            .unwrap_or_default();

        match notification_request.channel {
            NotificationChannel::Push => {
                if recipient_type.is_empty() {
                    return Err(NotiSrvError::InvalidDataField);
                }
            }
            _ => (),
        }

        // Save notification into database
        let noti_id = self
            .noti_repo
            .insert(&notification_request)
            .await
            .map_err(|e| {
                error!("Database insert error: {}", e.to_string());
                NotiSrvError::DatabaseError(e)
            })?;

        // Generate value
        let enqueue_value = NotificationEnQueue {
            notification_id: noti_id.clone(),
            recipient: notification_request.recipient,
            recipient_type,
            channel: notification_request.channel.to_string(),
            template_id: notification_request.template_id.unwrap_or_default(),
            payload: notification_request.payload,
        };

        let job = serde_json::json!(enqueue_value);

        let queue_key = env::var("QUEUE_KEY").map_err(|e| {
            error!("Missing env: {}", e);
            NotiSrvError::MissingEnvError(e)
        })?;

        // Push job into redis queue
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

    fn validate_payload(&self, channel: &NotificationChannel, payload: &Value) -> bool {
        match channel {
            NotificationChannel::Push => PushPayload::validate_payload(payload),
            NotificationChannel::Email => EmailPayload::validate_payload(payload),
            _ => false,
        }
    }
}

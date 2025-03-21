use std::{env, sync::Arc};

use async_trait::async_trait;
use log::{error, info, warn};
use serde_json::json;

use crate::module::notification_delivery_module::{
    errors::NotiDeliverError,
    models::{notification::NotificationDeQueue, push_payload::PushPayload},
    repositories::notification_repository::NotificationRepo,
    utils::fcm_token_manager::TokenManager,
};

use super::notification_worker_actor::NotificationWorker;

/// `PushWorker` is responsible for sending push notifications using Firebase Cloud Messaging.
/// It fetches FCM tokens, constructs the request payload, and sends notifications via HTTP requests
pub struct PushWorker {
    client: reqwest::Client,
    url: String,
    token_manager: TokenManager,
}

impl PushWorker {
    /// Creates a new instance of `PushWorker`
    pub async fn new(token_manager: TokenManager) -> Self {
        let project_id = env::var("PROJECT_ID").expect("PROJECT_ID must be set");
        Self {
            client: reqwest::Client::new(),
            url: format!(
                "https://fcm.googleapis.com/v1/projects/{}/messages:send",
                project_id
            ),
            token_manager,
        }
    }

    /// Attempts to send a push notification request to FCM
    async fn try_send(
        &self,
        token: &str,
        message: &serde_json::Value,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .post(&self.url)
            .bearer_auth(token)
            .body(message.to_string())
            .send()
            .await
    }
}

#[async_trait]
impl NotificationWorker for PushWorker {
    /// Sends a notification using FCM
    async fn send(
        &self,
        notification: &NotificationDeQueue,
        repo: Arc<NotificationRepo>,
    ) -> Result<(), NotiDeliverError> {
        // Parse the notification payload into `PushPayload`
        let payload =
            serde_json::from_value::<PushPayload>(notification.payload.clone()).map_err(|e| {
                error!("Invalid data type: {}", e);
                NotiDeliverError::JsonParseError
            })?;

        let recipient_type = match notification.recipient_type.clone() {
            Some(value) => value,
            None => {
                error!("Missing recipient_type");
                return Err(NotiDeliverError::JsonParseError);
            }
        };

        // Construct the FCM request message
        let message = json!({
            "message": {
                recipient_type: notification.recipient.clone(),
                "notification": {
                    "title": payload.title,
                    "body": payload.body
                }
            }
        });

        // Try sending the request, and retry once if unauthorized (401)
        for _i in 0..1 {
            let token = match self.token_manager.get_token() {
                Some(token) => token,
                None => {
                    error!("Empty token");
                    return Err(NotiDeliverError::NoneValue);
                }
            };

            // Attempt to send the notification
            match self.try_send(token.as_str(), &message).await {
                Ok(response) => {
                    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
                        // Token expired, attempt to refresh
                        warn!("Token expired, refreshing token...");
                        self.token_manager.update_token().await;
                        continue;
                    } else if response.status().is_success() {
                        // Successfully sent notification, update database status
                        let result = repo
                            .update_notification_status(&notification.notification_id, "sent")
                            .await
                            .map_err(|e| {
                                error!("Update error: {}", e);
                                NotiDeliverError::DatabaseError(e)
                            })?;
                        info!("Update row affected: {}", result);
                        return Ok(());
                    } else {
                        // Failed to send notification, update status to "failed"
                        let result = repo
                            .update_notification_status(&notification.notification_id, "failed")
                            .await
                            .map_err(|e| {
                                error!("Update error: {}", e);
                                NotiDeliverError::DatabaseError(e)
                            })?;
                        info!("Update row affected: {}", result);
                        return Err(NotiDeliverError::RequestFailed);
                    }
                }
                Err(e) => {
                    error!("Can not send request: {}", e);
                    return Err(NotiDeliverError::RequestError(e));
                }
            };
        }

        error!("Can not send request");
        Err(NotiDeliverError::RequestFailed)
    }
}

use std::{env, sync::Arc};

use async_trait::async_trait;
use log::{error, info};

use crate::module::notification_delivery_module::{
    errors::NotiDeliverError,
    models::{
        email_payload::{Attachments, EmailPayload, ReplyTo},
        notification::NotificationDeQueue,
    },
    repositories::notification_repository::NotificationRepo,
};

use super::notification_worker_actor::NotificationWorker;

pub struct EmailWorker {
    client: reqwest::Client,
    url: String,
    api_key: String,
}

impl EmailWorker {
    pub fn new() -> Self {
        let client = reqwest::Client::new();
        let url = "https://api.sendgrid.com/v3/mail/send".to_string();
        let api_key = env::var("SENDGRID_API_KEY").expect("SENDGRID_API_KEY must be set");

        Self {
            client,
            url,
            api_key,
        }
    }

    async fn try_send(
        &self,
        message: &serde_json::Value,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .post(&self.url)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
            .body(message.to_string())
            .send()
            .await
    }
}

#[async_trait]
impl NotificationWorker for EmailWorker {
    async fn send(
        &self,
        notification: &NotificationDeQueue,
        repo: Arc<NotificationRepo>,
    ) -> Result<(), NotiDeliverError> {
        // Parse the notification payload into `EmailPayload`
        let payload = serde_json::from_value::<EmailPayload>(notification.payload.clone())
            .map_err(|e| {
                error!("Invalid data type: {}", e);
                NotiDeliverError::JsonParseError
            })?;

        // Try parsing payload's variables
        let mut attachments: Option<Vec<Attachments>> = None;
        let mut reply: Option<ReplyTo> = None;
        if let Some(value) = payload.optionals {
            // Get attachment value if exists
            if let Some(v) = value.get("attachments") {
                if let Ok(att) = serde_json::from_value::<Vec<Attachments>>(v.clone()) {
                    attachments = Some(att);
                };
            };
            // Get reply_to value if exists
            if let Some(v) = value.get("reply_to") {
                if let Ok(rep) = serde_json::from_value::<ReplyTo>(v.clone()) {
                    reply = Some(rep);
                };
            };
        };

        // Construct the request message
        let mut message = serde_json::json!({
            "personalizations": [{
                "to": [{"email": notification.recipient}]
            }],
            "from": {
                "email": notification.sender
            },
            "subject": payload.subject,
            "content": [{
                "type": payload.content_type,
                "value": payload.content
            }]
        });

        if let Some(atts) = attachments {
            message["attachments"] = serde_json::json!(atts);
        }

        if let Some(rep) = reply {
            message["reply_to"] = serde_json::json!(rep);
        }

        // return Ok(());

        // Attempt to send the notification
        match self.try_send(&message).await {
            Ok(response) => {
                // info!("SendGrid response: {:?}", response);
                if response.status().is_success() {
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
                    // Failed to send notification
                    return Err(NotiDeliverError::RequestFailed);
                }
            }
            Err(e) => {
                error!("Can not send request: {}", e);
                return Err(NotiDeliverError::RequestError(e));
            }
        }
    }
}

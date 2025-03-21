use derive_more::Display;
use serde::{Deserialize, Serialize};

// pub struct Notification {
//     pub id: String,
//     pub user_id: String,
//     pub recipient: String,
//     pub channel: String,
//     pub template_id: Option<String>,
//     pub status: String,
//     pub created_at: String,
//     pub updated_at: String,
// }

#[derive(Debug, Deserialize, Serialize, Clone, Display)]
#[serde(rename_all = "lowercase")]
pub enum NotificationChannel {
    #[display("push")]
    Push,
    #[display("email")]
    Email,
    #[display("sms")]
    Sms,
}

#[derive(Debug, Deserialize, Serialize, Clone, Display)]
#[serde(rename_all = "lowercase")]
pub enum PushRecipientType {
    #[display("token")]
    Token,
    #[display("topic")]
    Topic,
    #[display("condition")]
    Condition,
}

#[derive(Debug, Deserialize)]
pub struct NotificationRequest {
    pub user_id: String,
    pub recipient: String,
    pub recipient_type: Option<PushRecipientType>,
    pub sender: Option<String>,
    pub channel: NotificationChannel,
    pub template_id: Option<String>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationEnQueue {
    pub notification_id: String,
    pub recipient: String,
    pub recipient_type: Option<String>,
    pub sender: Option<String>,
    pub channel: String,
    pub template_id: Option<String>,
    pub payload: serde_json::Value,
}

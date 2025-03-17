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

#[derive(Debug, Deserialize)]
pub struct NotificationRequest {
    pub user_id: String,
    pub recipient: String,
    pub channel: String,
    pub template_id: Option<String>,
    pub data: serde_json::Value,
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
    pub channel: String,
    pub template_id: String,
    pub data: serde_json::Value,
}

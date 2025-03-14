use serde::Deserialize;

pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub recipient: String,
    pub channel: String,
    pub template_id: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct NotificationRequest {
    pub user_id: String,
    pub recipient: String,
    pub channel: String,
    pub template_id: Option<String>,
    pub data: serde_json::Value,
}

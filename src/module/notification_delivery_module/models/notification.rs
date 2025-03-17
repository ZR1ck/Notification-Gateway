use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NotificationDeQueue {
    pub notification_id: String,
    pub recipient: String,
    pub channel: String,
    pub template_id: String,
    pub data: serde_json::Value,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct NotificationDeQueue {
    pub notification_id: String,
    pub recipient: String,
    pub recipient_type: Option<String>,
    pub channel: String,
    pub template_id: Option<String>,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub retry_count: u8,
}

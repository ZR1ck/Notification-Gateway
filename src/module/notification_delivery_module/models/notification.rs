use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct NotificationDeQueue {
    pub notification_id: String,
    pub recipient: String,
    pub recipient_type: String,
    pub channel: String,
    pub _template_id: String,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub retry_count: u8,
}

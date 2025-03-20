use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PushPayload {
    pub title: String,
    pub body: String,
}

// pub struct EmailPayload {
//     pub subject: String,
//     pub message: String,
//     pub variables: Option<serde_json::Value>,
// }
// pub struct SmsPayload {
//     pub message: String,
// }

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PushPayload {
    pub title: String,
    pub body: String,
}

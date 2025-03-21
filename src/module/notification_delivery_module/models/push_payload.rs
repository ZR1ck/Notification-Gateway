use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PushPayload {
    pub title: String,
    pub body: String,
}

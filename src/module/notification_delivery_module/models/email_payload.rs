use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct EmailPayload {
    pub subject: String,
    pub content: String,
    #[serde(default = "default_content_type")]
    pub content_type: String,
    pub optionals: Option<serde_json::Value>,
}

fn default_content_type() -> String {
    "text/plain".to_string()
}

// Variables options
#[derive(Debug, Deserialize, Serialize)]
pub struct Attachments {
    pub content: String,
    pub filename: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default = "default_disposition")]
    pub disposition: String,
}

fn default_disposition() -> String {
    "attachment".to_string()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReplyTo {
    pub email: String,
    pub name: String,
}

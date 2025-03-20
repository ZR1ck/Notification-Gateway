use serde_json::Value;

pub struct PushPayload;

impl Payload for PushPayload {
    fn validate_payload(payload: &Value) -> bool {
        payload.get("title").map_or(false, |v| v.is_string())
            && payload.get("body").map_or(false, |v| v.is_string())
    }
}

pub struct EmailPayload;

impl Payload for EmailPayload {
    fn validate_payload(payload: &Value) -> bool {
        payload.get("subject").map_or(false, |v| v.is_string())
            && payload.get("message").map_or(false, |v| v.is_string())
            && payload.get("variables").map_or(true, |v| v.is_object())
    }
}

// pub struct SmsPayload;

pub trait Payload {
    fn validate_payload(payload: &Value) -> bool;
}

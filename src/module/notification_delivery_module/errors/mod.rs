use std::env::VarError;

use actix_web::{HttpResponse, ResponseError};
use deadpool_redis::PoolError;
use derive_more::{Display, Error};
use redis::RedisError;

#[derive(Debug, Display, Error)]
pub enum NotiDeliverError {
    #[display("Database query failed")]
    DatabaseError(sqlx::Error),

    #[display("Env must be set")]
    MissingEnvError(VarError),

    #[display("Redis connect failed")]
    RedisConnectionError(PoolError),

    #[display("Redis pop failed")]
    RedisQueuePopError(RedisError),

    #[display("None value")]
    NoneValue,

    #[display("Cannot parse this object")]
    JsonParseError,

    #[display("Cannot send request")]
    RequestError(reqwest::Error),

    #[display("Google cloud platform authentication")]
    GCPAuthError(gcp_auth::Error),

    #[display("Request failed")]
    RequestFailed,
}

impl ResponseError for NotiDeliverError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match self {
            NotiDeliverError::DatabaseError(_) => HttpResponse::InternalServerError()
                .body(self.to_string())
                .map_into_boxed_body(),

            NotiDeliverError::RedisQueuePopError(_) => HttpResponse::InternalServerError()
                .body(self.to_string())
                .map_into_boxed_body(),

            NotiDeliverError::GCPAuthError(_) => HttpResponse::InternalServerError()
                .body(self.to_string())
                .map_into_boxed_body(),

            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}

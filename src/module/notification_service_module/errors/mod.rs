use std::env::VarError;

use actix_web::{HttpResponse, ResponseError};
use deadpool_redis::PoolError;
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum NotiSrvError {
    #[display("Database query failed")]
    DatabaseError(sqlx::Error),

    #[display("Env must be set")]
    MissingEnvError(VarError),

    #[display("Redis push failed")]
    RedisQueuePushError(PoolError),

    #[display("Invalid data field")]
    InvalidDataField,
}

impl ResponseError for NotiSrvError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match self {
            NotiSrvError::DatabaseError(_) => HttpResponse::InternalServerError()
                .body(self.to_string())
                .map_into_boxed_body(),

            NotiSrvError::RedisQueuePushError(_) => HttpResponse::InternalServerError()
                .body(self.to_string())
                .map_into_boxed_body(),

            NotiSrvError::InvalidDataField => HttpResponse::InternalServerError()
                .body(self.to_string())
                .map_into_boxed_body(),

            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}

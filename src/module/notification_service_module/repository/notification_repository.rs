use std::sync::Arc;

use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::module::notification_service_module::models::notification::NotificationRequest;

pub struct NotificationRepo {
    pool: Arc<PgPool>,
}

impl NotificationRepo {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

impl NotificationRepo {
    pub async fn insert(
        &self,
        notification_request: &NotificationRequest,
    ) -> Result<String, sqlx::Error> {
        // generate id
        let uuid = Uuid::new_v4();

        // get statement
        let stm = include_str!("../queries/insert_noti.sql");
        // bind values
        let user_id = Uuid::parse_str(&notification_request.user_id).unwrap();
        let template_id = notification_request.template_id.clone().unwrap_or_default();
        let template_id = Uuid::parse_str(&template_id).unwrap();

        let result = sqlx::query(stm)
            .bind(uuid)
            .bind(user_id)
            .bind(notification_request.recipient.clone())
            .bind(notification_request.channel.clone())
            .bind(template_id)
            .bind("pending".to_string())
            .execute(&*self.pool)
            .await?;

        info!("Query insert result: {}", result.rows_affected());

        Ok(uuid.to_string())
    }
}

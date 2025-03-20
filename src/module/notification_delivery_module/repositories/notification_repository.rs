use std::sync::Arc;

use log::info;
use sqlx::PgPool;
use uuid::Uuid;

pub struct NotificationRepo {
    pg_pool: Arc<PgPool>,
}

impl NotificationRepo {
    pub fn new(pg_pool: Arc<PgPool>) -> Self {
        Self { pg_pool }
    }

    pub async fn update_notification_status(
        &self,
        noti_id: &str,
        status: &str,
    ) -> Result<u64, sqlx::Error> {
        let stm = include_str!("../queries/update_notification_status.sql");

        let noti_id = Uuid::parse_str(noti_id).unwrap();

        let result = sqlx::query(stm)
            .bind(status)
            .bind(noti_id)
            .execute(&*self.pg_pool)
            .await?;

        let rows_affected = result.rows_affected();
        info!("Rows affected: {}", rows_affected);

        Ok(rows_affected)
    }
}

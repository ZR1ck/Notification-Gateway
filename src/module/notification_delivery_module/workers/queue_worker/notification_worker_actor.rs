use std::sync::Arc;

use actix::{Actor, AsyncContext, Context, Handler, WrapFuture};
use async_trait::async_trait;
use log::{error, info, warn};

use crate::module::notification_delivery_module::{
    errors::NotiDeliverError,
    models::notification::NotificationDeQueue,
    repositories::{notification_repository::NotificationRepo, redis_repository::RedisRepository},
};

use super::NotificationMessage;

/// Trait defining a worker responsible for sending notifications asynchronously.
#[async_trait]
pub trait NotificationWorker: Send + Sync {
    /// Sends a notification using the given repository.
    async fn send(
        &self,
        notification: &NotificationDeQueue,
        repo: Arc<NotificationRepo>,
    ) -> Result<(), NotiDeliverError>;
}

/// Actor responsible for processing notification messages using a `NotificationWorker`
///
/// Every worker MUST implement the `NotificationWorker` trait and be wrapped within a
/// `NotificationWorkerActor` to integrate seamlessly with the Actix actor system
pub struct NotificationWorkerActor {
    worker: Arc<dyn NotificationWorker>,
    noti_repo: Arc<NotificationRepo>,
    redis_repo: Arc<RedisRepository>,
}

impl NotificationWorkerActor {
    /// Creates a new `NotificationWorkerActor` instance
    pub fn new(
        worker: Arc<dyn NotificationWorker>,
        noti_repo: Arc<NotificationRepo>,
        redis_repo: Arc<RedisRepository>,
    ) -> Self {
        Self {
            worker,
            noti_repo,
            redis_repo,
        }
    }
}

impl Actor for NotificationWorkerActor {
    type Context = Context<Self>;
}

impl Handler<NotificationMessage> for NotificationWorkerActor {
    type Result = ();

    /// Handles an incoming `NotificationMessage` and processes it asynchronously
    ///
    /// This method spawns an async task within the actor's context to send the notification
    /// The result of the `send` operation is ignored, but error handling could be improved
    fn handle(&mut self, msg: NotificationMessage, ctx: &mut Self::Context) -> Self::Result {
        let worker = self.worker.clone();
        let noti_repo = self.noti_repo.clone();
        let redis_repo = self.redis_repo.clone();
        let mut notification = msg.0;
        let queue_key = msg.1;

        // Spawn an asynchronous task within the actor's context to process the notification
        ctx.spawn(
            async move {
                if let Err(e) = worker.send(&notification, noti_repo.clone()).await {
                    // Retrying failed job
                    error!("Send request error: {}", e);
                    warn!("Puting back to queue...");

                    notification.retry_count += 1;
                    warn!("Retrying job (attempt {}/3)...", notification.retry_count);

                    // Construct string valus of NotificationDequeue
                    let value = serde_json::json!(&notification);

                    if notification.retry_count < 3 {
                        if let Err(e) = redis_repo
                            .push_to_queue(&queue_key, &value.to_string())
                            .await
                        {
                            error!("Cannot be put back to queue: {}", e);
                        };
                    } else {
                        // Move job to failed queue if failed more than 3 times
                        error!("Job failed too many times, moving to failed queue...");
                        let mut failed_key = queue_key.clone();
                        failed_key.push_str("_failed");

                        if let Err(e) = redis_repo
                            .push_to_queue(&failed_key, &value.to_string())
                            .await
                        {
                            error!("Cannot push to failed queue: {}", e);
                        };
                        // Update status to "failed"
                        if let Ok(result) = noti_repo
                            .update_notification_status(&notification.notification_id, "failed")
                            .await
                        {
                            info!("Update row affected: {}", result);
                        } else {
                            error!("Update error: {}", e);
                        }
                    }
                };
            }
            .into_actor(self),
        );
    }
}

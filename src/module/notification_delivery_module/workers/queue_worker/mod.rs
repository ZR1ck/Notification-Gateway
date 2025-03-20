use std::{
    collections::HashMap,
    env,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use actix::{Actor, AsyncContext, Context, Message, Recipient, WrapFuture};
use actix_web::rt::time::sleep;
use log::{error, info, warn};
use redis::AsyncCommands;

use crate::module::notification_delivery_module::{
    errors::NotiDeliverError,
    models::notification::NotificationDeQueue,
    repositories::{notification_repository::NotificationRepo, redis_repository::RedisRepository},
};

pub mod email_worker;
pub mod notification_worker_actor;
pub mod push_worker;

/// Represents a message containing a dequeued notification
/// This message is sent to the appropriate worker for processing
#[derive(Message)]
#[rtype(result = "()")]
pub struct NotificationMessage(pub NotificationDeQueue, pub String);

/// Actor responsible for processing queued notifications
pub struct QueueWorker {
    redis_repo: Arc<RedisRepository>,
    noti_repo: Arc<NotificationRepo>,
    running: Arc<AtomicBool>, // Flag to control the worker execution loop
    workers: HashMap<String, Recipient<NotificationMessage>>, // Map of workers handling different notification channels
}

impl Actor for QueueWorker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Queue Worker started");

        let redis_repo = self.redis_repo.clone();
        let noti_repo = self.noti_repo.clone();
        let running = self.running.clone();
        let workers = self.workers.clone();

        // Spawn an async task that continuously processes notifications while the worker is running
        ctx.spawn(
            async move {
                while running.load(Ordering::Relaxed) {
                    let result = QueueWorker::process_notification(
                        redis_repo.clone(),
                        noti_repo.clone(),
                        running.clone(),
                        workers.clone(),
                    )
                    .await;

                    match result {
                        Ok(_) => warn!("Worker stopped"),
                        Err(e) => {
                            error!("Worker crashed: {}", e);
                            warn!("Restarting worker...");
                            sleep(Duration::from_secs(10)).await;
                        }
                    }
                }
            }
            .into_actor(self),
        );
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("Queue Worker stopped");
    }
}

impl QueueWorker {
    pub fn new(
        redis_repo: Arc<RedisRepository>,
        noti_repo: Arc<NotificationRepo>,
        workers: HashMap<String, Recipient<NotificationMessage>>,
    ) -> Self {
        Self {
            redis_repo,
            noti_repo,
            running: Arc::new(AtomicBool::new(true)),
            workers,
        }
    }

    /// Asynchronously processes notifications from the Redis queue.
    async fn process_notification(
        redis_repo: Arc<RedisRepository>,
        noti_repo: Arc<NotificationRepo>,
        running: Arc<AtomicBool>,
        workers: HashMap<String, Recipient<NotificationMessage>>,
    ) -> Result<(), NotiDeliverError> {
        // Establish Redis connection
        let mut conn = redis_repo.pool.get().await.map_err(|e| {
            error!("Cannot connect to Redis: {}", e);
            NotiDeliverError::RedisConnectionError(e)
        })?;

        // Fetch queue key from environment variables
        let queue_key = env::var("QUEUE_KEY").map_err(|e| {
            error!("Env key error: {}", e);
            NotiDeliverError::MissingEnvError(e)
        })?;

        while running.load(Ordering::Relaxed) {
            // Attempt to pop a job from the Redis queue
            let job: Option<String> = conn.rpop(&queue_key, None).await.map_err(|e| {
                error!("Queue pop error: {}", e);
                NotiDeliverError::RedisQueuePopError(e)
            })?;

            if let Some(job_data) = job {
                // Deserialize the notification message
                match serde_json::from_str::<NotificationDeQueue>(&job_data) {
                    Ok(notification) => {
                        // Dispatch the message to the appropriate worker
                        if let Some(worker) = workers.get(&notification.channel) {
                            worker.do_send(NotificationMessage(notification, queue_key.clone()));
                        } else {
                            error!("No worker found for channel: {}", notification.channel);
                            return Err(NotiDeliverError::NoneValue);
                        }
                    }
                    Err(e) => {
                        // Push failed value to the failed queue
                        error!("Failed to parse notification: {}", e);
                        warn!("Value will be pushed to failed queue");
                        let mut failed_key = queue_key.clone();
                        failed_key.push_str("_failed");
                        if let Err(e) = redis_repo.push_to_queue(&failed_key, &job_data).await {
                            error!("Cannot push to failed queue: {}", e);
                        };

                        // Parse failed Json for id
                        let job_json: Result<serde_json::Value, serde_json::Error> =
                            serde_json::from_str(&job_data);
                        if let Ok(json) = job_json {
                            // Attempt to update status with failed id if it is found in Json
                            if let Some(id) = json.get("notification_id").and_then(|id| id.as_str())
                            {
                                // Update status to "failed"
                                if let Ok(result) =
                                    noti_repo.update_notification_status(&id, "failed").await
                                {
                                    info!("Update row affected: {}", result);
                                } else {
                                    error!("Update error: {}", e);
                                }
                            } else {
                                error!("Job id not found in corrupted JSON");
                            }
                        } else {
                            error!("Completely invalid JSON");
                        }
                    }
                }
            } else {
                warn!("Queue is empty, retrying after 10 seconds...");
                sleep(Duration::from_secs(10)).await;
            }
        }

        Ok(())
    }
}

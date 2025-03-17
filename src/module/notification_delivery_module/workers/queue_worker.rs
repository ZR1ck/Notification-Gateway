use std::{
    env,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use actix::{Actor, AsyncContext, Context, Handler, Message, WrapFuture};
use actix_web::rt::time::sleep;
use log::{error, info, warn};
use redis::AsyncCommands;

use crate::module::notification_delivery_module::{
    errors::NotiDeliverError,
    models::notification::NotificationDeQueue,
    repositories::{notification_repository::NotificationRepo, redis_repository::RedisRepository},
};

#[derive(Message)]
#[rtype(result = "bool")]
pub struct GetWorkerStatus;

pub struct QueueWorker {
    redis_repo: Arc<RedisRepository>,
    noti_repo: Arc<NotificationRepo>,
    running: Arc<AtomicBool>,
}

impl Actor for QueueWorker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Worker started");

        let redis_repo = self.redis_repo.clone();
        let noti_repo = self.noti_repo.clone();
        let running = self.running.clone();

        ctx.spawn(
            async move {
                loop {
                    match QueueWorker::process_notification(
                        redis_repo.clone(),
                        noti_repo.clone(),
                        running.clone(),
                    )
                    .await
                    {
                        Ok(_) => {
                            warn!("Worker stopped");
                        }
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
}

impl Handler<GetWorkerStatus> for QueueWorker {
    type Result = bool;

    fn handle(&mut self, _: GetWorkerStatus, _: &mut Self::Context) -> Self::Result {
        self.get_status()
    }
}

impl QueueWorker {
    pub fn new(redis_repo: Arc<RedisRepository>, noti_repo: Arc<NotificationRepo>) -> Self {
        Self {
            redis_repo,
            noti_repo,
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    async fn process_notification(
        redis_repo: Arc<RedisRepository>,
        noti_repo: Arc<NotificationRepo>,
        running: Arc<AtomicBool>,
    ) -> Result<(), NotiDeliverError> {
        let mut conn = redis_repo.pool.get().await.map_err(|e| {
            error!("Cannot connect to redis");
            NotiDeliverError::RedisConnectionError(e)
        })?;

        let queue_key = env::var("QUEUE_KEY").map_err(|e| {
            error!("Env key error: {e}");
            NotiDeliverError::MissingEnvError(e)
        })?;

        while running.load(Ordering::Relaxed) {
            let job: Option<String> = conn.rpop(&queue_key, None).await.map_err(|e| {
                error!("Queue pop error: {}", e);
                NotiDeliverError::RedisQueuePopError(e)
            })?;

            if let Some(job_data) = job {
                if let Ok(notification) = serde_json::from_str::<NotificationDeQueue>(&job_data) {
                    let channel = notification.channel.as_str();
                    let send_result = match channel {
                        "email" => {
                            info!("Send to emai: {:?}", notification);
                            true
                        }
                        "push" => {
                            info!("Send to push: {:?}", notification);
                            true
                        }
                        "sms" => {
                            info!("Send to sms: {:?}", notification);
                            true
                        }
                        _ => false,
                    };

                    let status = if send_result { "sent" } else { "failed" };

                    let result = noti_repo
                        .update_notification_status(&notification.notification_id, status)
                        .await
                        .map_err(|e| {
                            error!("Update status error: {}", e);
                            NotiDeliverError::DatabaseError(e)
                        })?;

                    info!("Update result: {}", result);
                }
            } else {
                warn!("Empty queue");
                sleep(Duration::from_secs(30)).await;
            }
        }

        Ok(())
    }

    pub fn get_status(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

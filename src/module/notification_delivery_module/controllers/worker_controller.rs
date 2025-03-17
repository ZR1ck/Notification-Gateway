use actix::Addr;
use actix_web::{
    web::{self, ServiceConfig},
    HttpResponse, Responder,
};

use crate::module::notification_delivery_module::workers::queue_worker::{
    GetWorkerStatus, QueueWorker,
};

pub struct WorkerController;

impl WorkerController {
    pub fn routes(cfg: &mut ServiceConfig) {
        cfg.service(web::scope("/worker").route(
            "/queue/status",
            web::get().to(Self::get_queue_worker_status),
        ));
    }

    async fn get_queue_worker_status(worker: web::Data<Addr<QueueWorker>>) -> impl Responder {
        let status = worker.send(GetWorkerStatus).await.unwrap_or(false);
        let response = if status {
            "Queue worker is running"
        } else {
            "Queue worker stopped"
        };
        HttpResponse::Ok().body(response.to_string())
    }
}

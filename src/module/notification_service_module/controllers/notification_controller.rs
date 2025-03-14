use std::sync::Arc;

use actix_web::{
    web::{self, Json},
    HttpResponse, Responder,
};

use crate::module::notification_service_module::{
    models::notification::NotificationRequest, services::notification_service::NotificationService,
};

pub struct NotificationController {
    noti_service: Arc<NotificationService>,
}

impl NotificationController {
    pub fn new(noti_service: Arc<NotificationService>) -> Self {
        Self { noti_service }
    }

    pub fn routes(cfg: &mut web::ServiceConfig) {
        cfg.service(web::scope("/notification").route("/send", web::post().to(Self::send)));
    }

    async fn send(
        self_controller: web::Data<Arc<NotificationController>>,
        notification_request: Json<NotificationRequest>,
    ) -> impl Responder {
        HttpResponse::Ok()
    }
}

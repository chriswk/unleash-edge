use actix_web::{get, web, HttpResponse, Responder};

#[get("/metrics")]
async fn metrics() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json("healthy")
}

pub fn configure_backstage(cfg: &mut web::ServiceConfig) {
    cfg.service(metrics).service(health);
}

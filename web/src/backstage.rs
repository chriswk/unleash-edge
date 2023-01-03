use actix_web::{get, web, HttpResponse, Responder};

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json("healthy")
}

pub fn configure_backstage(cfg: &mut web::ServiceConfig) {
    cfg.service(health);
}

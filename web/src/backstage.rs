use crate::EdgeJsonResult;
use actix_web::web::Json;
use actix_web::{get, web, HttpResponse, Responder};
use std::sync::Arc;
use storage::{CachedData, FullState};
use types::{EdgeResult, EdgeToken};

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json("healthy")
}

#[get("/register-token")]
async fn register_token(
    token: EdgeToken,
    full_state: web::Data<Arc<FullState>>,
) -> EdgeJsonResult<()> {
    full_state.data.insert(token.token, CachedData::default());
    Ok(Json(()))
}

#[get("/tokens")]
async fn get_tokens(full_state: web::Data<Arc<FullState>>) -> EdgeJsonResult<FullState> {
    Ok(Json(full_state.as_ref().as_ref().clone()))
}

pub fn configure_backstage(cfg: &mut web::ServiceConfig) {
    cfg.service(health)
        .service(register_token)
        .service(get_tokens);
}

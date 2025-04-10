use actix_web::{get, web, HttpResponse};
use crate::auth::middleware::AuthMiddleware;

#[get("/protected")]
async fn protected() -> HttpResponse {
    HttpResponse::Ok().body("This is protected data.")
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("").wrap(AuthMiddleware).service(protected),
    );
}
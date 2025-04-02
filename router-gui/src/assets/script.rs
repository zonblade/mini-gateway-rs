use actix_web::{get, HttpResponse};

#[get("/js/{tail:.*}")]
pub async fn init() -> HttpResponse {
    HttpResponse::Ok().finish()
}
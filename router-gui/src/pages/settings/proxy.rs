use actix_web::{get, HttpResponse};

#[get("/proxy")]
pub async fn proxy() -> HttpResponse {
    HttpResponse::Ok().finish()
}
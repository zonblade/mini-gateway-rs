use actix_web::{get, HttpResponse};

#[get("/css/{tail:.*}")]
pub async fn init() -> HttpResponse {
    HttpResponse::Ok().finish()
}
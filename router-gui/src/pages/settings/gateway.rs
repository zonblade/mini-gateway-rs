use actix_web::{get, HttpResponse};

#[get("/gateway")]
pub async fn gateway() -> HttpResponse {
    HttpResponse::Ok().finish()
}
use actix_web::{get, HttpResponse};

#[get("/users")]
pub async fn users() -> HttpResponse {
    HttpResponse::Ok().finish()
}
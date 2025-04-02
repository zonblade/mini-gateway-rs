use actix_web::{get, HttpResponse};

#[get("/gwnode")]
pub async fn gwnode() -> HttpResponse {
    HttpResponse::Ok().finish()
}
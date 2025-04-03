use actix_web::{get, web, HttpResponse};

#[get("/css/{tail:.*}")]
pub async fn init(path: web::Path<String>) -> HttpResponse {
    let tail = path.into_inner();
    println!("Requested CSS file: {}", tail);
    HttpResponse::Ok().body(format!("Requested: {}", tail))
}
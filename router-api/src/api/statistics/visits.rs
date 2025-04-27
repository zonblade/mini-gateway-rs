use actix_web::{web, get, Responder};

use super::logs_broadcast::LogsBroadcaster;

// Function that creates a stream using a simple loop
#[get("/visits")]
pub async fn logs_stream(broadcaster: web::Data<LogsBroadcaster>) -> impl Responder {
    broadcaster.new_client().await
}
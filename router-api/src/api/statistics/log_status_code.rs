use actix_web::{get, web, HttpResponse, Responder};

use crate::module::temporary_log::tlog_gateway;

/// Get a proxy by ID
///
/// This endpoint returns a specific proxy configuration by its ID,
/// along with all associated proxy domains.
#[get("/status/{status}")]
pub async fn init(path: web::Path<String>) -> impl Responder {
    let status = path.into_inner();
    let status: i32 = match status.parse() {
        Ok(status) => status,
        Err(_) => {
            log::error!("Invalid status code provided");
            return HttpResponse::BadRequest().body("Invalid status code");
        }
    };

    let end = chrono::Utc::now();
    // start 30 minutes before
    let start = end - chrono::Duration::minutes(30);

    let result = tlog_gateway::get_data_time_frame_by_status_code(start, end, status);

    let result = match result {
        Ok(data) => data,
        Err(e) => {
            log::error!("Error fetching domains for proxy {}", e);
            vec![]
        }
    };

    HttpResponse::Ok().json(result)
}

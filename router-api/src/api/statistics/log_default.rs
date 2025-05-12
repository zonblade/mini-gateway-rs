use actix_web::{get, HttpResponse, Responder};

use crate::module::temporary_log::tlog_gateway;

/// Get a proxy by ID
///
/// This endpoint returns a specific proxy configuration by its ID,
/// along with all associated proxy domains.
#[get("/default")]
pub async fn init() -> impl Responder {
    let end = chrono::Utc::now();
    // start 30 minutes before
    let start = end - chrono::Duration::minutes(30);

    let result = tlog_gateway::get_data_time_frame(start, end);

    let result = match result {
        Ok(data) => data,
        Err(e) => {
            log::error!("Error fetching domains for proxy {}", e);
            vec![]
        }
    };

    HttpResponse::Ok().json(result)
}

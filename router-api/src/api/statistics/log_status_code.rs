use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::module::temporary_log::{tlog_gateway, tlog_proxy};

#[derive(Deserialize)]
struct Params {
    target: Option<String>,
}

#[get("/status/{status}")]
pub async fn init(path: web::Path<String>, query: web::Query<Params>) -> impl Responder {
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
    let start = end - chrono::Duration::minutes(120);


    let result = {
        match &query.target {
            Some(str) => {
                match str.as_str() {
                    "proxy" => tlog_proxy::get_data_time_frame_by_status_code(start, end, status),
                    "domain" => tlog_gateway::get_data_time_frame_by_status_code(start, end, status),
                    _ => tlog_gateway::get_data_time_frame_by_status_code(start, end, status),
                }
            }
            None => tlog_gateway::get_data_time_frame_by_status_code(start, end, status),
        }
    };

    let result = match result {
        Ok(data) => data,
        Err(e) => {
            log::error!("Error fetching domains for proxy {}", e);
            vec![]
        }
    };

    HttpResponse::Ok().json(result)
}
